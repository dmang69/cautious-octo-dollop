//! Userspace syscall interception library for IntentKernel Phase 1 PoC.
//!
//! Presents capability handles to the kernel gate before allowing file, network,
//! or raw resource access. Scope matching ensures tokens only authorize their
//! declared resource — structural ransomware immunity.

mod scope;

use std::collections::HashMap;

use eventscope_ebpf::bridge::{bridge_is_loaded, publish_handle, BridgeError};
use eventscope_ebpf::policy::HandleMapEntry;
use intentkernel_core::gate::{SyscallError, SyscallRequest, SyscallResult};
use intentkernel_core::handle::KernelHandle;
use intentkernel_core::IntentKernel;
use intentkernel_crypto::sign::TokenValidator;
use intentkernel_crypto::token::{ResourceScope, WireToken};
use thiserror::Error;

pub use scope::{
    parse_ip_bytes, resource_type_for_scope, scope_matches_file, scope_matches_network,
    scope_matches_raw, scope_matches_resource, FileAccess, RESOURCE_FILE, RESOURCE_NETWORK,
    RESOURCE_RAW,
};

/// Outcome of an intercepted resource request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterceptVerdict {
    Allow { resource_type: u32 },
    Deny(InterceptError),
}

impl InterceptVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Deny(_))
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InterceptError {
    #[error("no capability handle presented")]
    NoHandle,
    #[error("handle not registered with eventscope")]
    UnknownHandle,
    #[error("token scope does not match requested resource")]
    ScopeMismatch,
    #[error("capability gate denied: {0}")]
    Gate(String),
    #[error("kernel BPF bridge: {0}")]
    KernelBridge(String),
}

impl From<SyscallError> for InterceptError {
    fn from(value: SyscallError) -> Self {
        Self::Gate(value.to_string())
    }
}

/// Syscall interception facade — wraps [`IntentKernel`] and [`TokenValidator`].
pub struct EventScope {
    kernel: IntentKernel,
    validator: TokenValidator,
    handle_scopes: HashMap<u64, ResourceScope>,
}

impl EventScope {
    pub fn new(validator: TokenValidator) -> Self {
        Self {
            kernel: IntentKernel::new(),
            validator,
            handle_scopes: HashMap::new(),
        }
    }

    pub fn with_kernel(validator: TokenValidator, kernel: IntentKernel) -> Self {
        Self {
            kernel,
            validator,
            handle_scopes: HashMap::new(),
        }
    }

    pub fn kernel(&self) -> &IntentKernel {
        &self.kernel
    }

    pub fn kernel_mut(&mut self) -> &mut IntentKernel {
        &mut self.kernel
    }

    pub fn validator(&self) -> &TokenValidator {
        &self.validator
    }

    pub fn scope_for_handle(&self, handle: u64) -> Option<&ResourceScope> {
        self.handle_scopes.get(&handle)
    }

    /// Register a signed wire token and return an optimized kernel handle.
    pub fn register_token(&mut self, token: &WireToken) -> Result<KernelHandle, InterceptError> {
        let scope = token.payload.scope.clone();
        let resource_type = resource_type_for_scope(&scope);
        let handle = self
            .kernel
            .gate()
            .register_token(token, &self.validator, resource_type)
            .map_err(InterceptError::from)?;
        self.handle_scopes.insert(handle.raw, scope);
        Ok(handle)
    }

    /// Intercept a file open/read/write request.
    pub fn intercept_file_open(
        &mut self,
        path: &str,
        handle: u64,
    ) -> InterceptVerdict {
        self.intercept_file(path, FileAccess::Read, handle)
    }

    /// Intercept a file access with explicit mode.
    pub fn intercept_file(
        &mut self,
        path: &str,
        access: FileAccess,
        handle: u64,
    ) -> InterceptVerdict {
        self.intercept(
            handle,
            |scope| {
                scope_matches_resource(
                    scope,
                    Some(path),
                    access,
                    None,
                    None,
                    0,
                    None,
                )
            },
            RESOURCE_FILE,
        )
    }

    /// Intercept an outbound network connect/send.
    pub fn intercept_network(&mut self, dst: &[u8], port: u16, handle: u64) -> InterceptVerdict {
        self.intercept_network_proto(dst, port, 1, handle)
    }

    /// Intercept network access with explicit protocol (1=TCP, 2=UDP per RFC).
    pub fn intercept_network_proto(
        &mut self,
        dst: &[u8],
        port: u16,
        proto: u32,
        handle: u64,
    ) -> InterceptVerdict {
        self.intercept(
            handle,
            |scope| {
                scope_matches_resource(
                    scope,
                    None,
                    FileAccess::Read,
                    Some(dst),
                    Some(port),
                    proto,
                    None,
                )
            },
            RESOURCE_NETWORK,
        )
    }

    /// Intercept a raw action identified by byte string or context hash.
    pub fn intercept_raw(&mut self, action: &[u8], handle: u64) -> InterceptVerdict {
        self.intercept(
            handle,
            |scope| {
                scope_matches_resource(
                    scope,
                    None,
                    FileAccess::Read,
                    None,
                    None,
                    0,
                    Some(action),
                )
            },
            RESOURCE_RAW,
        )
    }

    fn intercept<F>(&mut self, handle: u64, scope_ok: F, action: u32) -> InterceptVerdict
    where
        F: FnOnce(&ResourceScope) -> bool,
    {
        if handle == 0 {
            return InterceptVerdict::Deny(InterceptError::NoHandle);
        }

        let Some(scope) = self.handle_scopes.get(&handle) else {
            return InterceptVerdict::Deny(InterceptError::UnknownHandle);
        };

        if !scope_ok(scope) {
            return InterceptVerdict::Deny(InterceptError::ScopeMismatch);
        }

        let sequence = self
            .kernel
            .sequences
            .get(&handle)
            .copied()
            .unwrap_or(0)
            .saturating_add(1);

        match self.kernel.gate().invoke(SyscallRequest {
            handle,
            sequence,
            action,
        }) {
            SyscallResult::Allowed { resource_type } => InterceptVerdict::Allow { resource_type },
            SyscallResult::Denied(err) => InterceptVerdict::Deny(err.into()),
        }
    }

    pub fn revoke_handle(&mut self, handle: u64) -> Result<(), InterceptError> {
        self.kernel
            .gate()
            .revoke_handle(handle)
            .map_err(InterceptError::from)?;
        self.handle_scopes.remove(&handle);
        Ok(())
    }

    /// Publish a registered handle to the kernel BPF `handle_map` for LSM enforcement.
    ///
    /// When the eBPF loader is active, this binds the current process PID to the handle.
    /// Without BPF (WSL2 mock mode), writes go to the in-memory bridge used by policy tests.
    pub fn publish_handle_to_kernel(&self, handle: u64) -> Result<(), InterceptError> {
        if handle == 0 {
            return Err(InterceptError::NoHandle);
        }
        let Some(scope) = self.handle_scopes.get(&handle) else {
            return Err(InterceptError::UnknownHandle);
        };
        let pid = std::process::id();
        let entry = HandleMapEntry::new(pid, handle, resource_type_for_scope(scope));
        publish_handle(entry).map_err(|e: BridgeError| InterceptError::KernelBridge(e.to_string()))
    }

    /// Returns true when a kernel or mock BPF map bridge is accepting publishes.
    pub fn kernel_bridge_active(&self) -> bool {
        bridge_is_loaded()
    }
}

#[cfg(test)]
mod tests {
    use intentkernel_crypto::sign::{KeyPair, TokenIssuer};
    use intentkernel_crypto::token::{Algorithm, FileScope, NetworkScope, ResourceScope, TrustAnchor};

    use super::*;

    fn test_scope() -> (EventScope, KeyPair) {
        let kp = KeyPair::generate(Algorithm::Ed25519).unwrap();
        let validator = intentkernel_core::gate::make_validator(&kp.public_key, Algorithm::Ed25519);
        (EventScope::new(validator), kp)
    }

    fn issue_file_token(
        issuer: &TokenIssuer,
        path: &str,
        access: u32,
        uses: u32,
    ) -> WireToken {
        issuer
            .issue_capability(
                b"ransomware-test",
                b"open_file",
                ResourceScope::File(FileScope {
                    path: path.into(),
                    access,
                    inode: None,
                }),
                TrustAnchor::UiEvent,
                60_000,
                uses,
            )
            .unwrap()
    }

    #[test]
    fn no_handle_denied() {
        let (mut es, _) = test_scope();
        let verdict = es.intercept_file_open("/etc/passwd", 0);
        assert!(verdict.is_denied());
        assert!(matches!(
            verdict,
            InterceptVerdict::Deny(InterceptError::NoHandle)
        ));
    }

    #[test]
    fn valid_handle_allowed_once() {
        let (mut es, kp) = test_scope();
        let issuer = TokenIssuer::new(kp);
        let token = issue_file_token(&issuer, "/documents/photo.jpg", 2, 1);
        let handle = es.register_token(&token).unwrap().raw;

        let first = es.intercept_file("/documents/photo.jpg", FileAccess::Write, handle);
        assert!(first.is_allowed());

        let second = es.intercept_file("/documents/photo.jpg", FileAccess::Write, handle);
        assert!(second.is_denied());
    }

    #[test]
    fn scope_mismatch_denied() {
        let (mut es, kp) = test_scope();
        let issuer = TokenIssuer::new(kp);
        let token = issue_file_token(&issuer, "/documents/photo.jpg", 2, 5);
        let handle = es.register_token(&token).unwrap().raw;

        let verdict = es.intercept_file("/etc/shadow", FileAccess::Write, handle);
        assert!(matches!(
            verdict,
            InterceptVerdict::Deny(InterceptError::ScopeMismatch)
        ));
    }

    #[test]
    fn ransomware_bulk_encryption_blocked() {
        let (mut es, kp) = test_scope();
        let issuer = TokenIssuer::new(kp);

        // User granted write to one file via UI event.
        let token = issue_file_token(&issuer, "/home/user/report.docx", 2, 1);
        let handle = es.register_token(&token).unwrap().raw;

        let targets = [
            "/home/user/report.docx",
            "/home/user/photos/vacation.jpg",
            "/home/user/.ssh/id_rsa",
            "/var/lib/mysql/ibdata1",
        ];

        let mut encrypted = 0usize;
        for (i, path) in targets.iter().enumerate() {
            let verdict = if i == 0 {
                es.intercept_file(path, FileAccess::Write, handle)
            } else {
                // Malware attempts ambient encryption without per-file user consent.
                es.intercept_file(path, FileAccess::Write, 0)
            };
            if verdict.is_allowed() {
                encrypted += 1;
            }
        }

        assert_eq!(
            encrypted, 1,
            "ransomware must not encrypt more than the single user-authorized file"
        );

        // Re-attack the authorized file after exhausting uses.
        let retry = es.intercept_file("/home/user/report.docx", FileAccess::Write, handle);
        assert!(retry.is_denied(), "exhausted capability must deny re-use");
    }

    #[test]
    fn publish_handle_to_kernel_mock_bridge() {
        let (mut es, kp) = test_scope();
        let issuer = TokenIssuer::new(kp);
        let token = issue_file_token(&issuer, "/tmp/kernel-test", 1, 1);
        let handle = es.register_token(&token).unwrap().raw;

        assert!(es.kernel_bridge_active());
        es.publish_handle_to_kernel(handle).unwrap();

        let map = eventscope_ebpf::global_bridge()
            .lock()
            .expect("bridge")
            .snapshot();
        let pid = std::process::id();
        let verdict = eventscope_ebpf::policy::evaluate_hook(
            eventscope_ebpf::policy::SyscallHook::OpenAt,
            pid,
            &map,
        );
        assert!(verdict.is_allow());
    }

    #[test]
    fn network_intercept_respects_scope() {
        let (mut es, kp) = test_scope();
        let issuer = TokenIssuer::new(kp);
        let token = issuer
            .issue_capability(
                b"app",
                b"connect",
                ResourceScope::Network(NetworkScope {
                    proto: 1,
                    dst_ip: vec![93, 184, 216, 34],
                    dst_port: 443,
                    bytes: 4096,
                }),
                TrustAnchor::UiEvent,
                30_000,
                1,
            )
            .unwrap();
        let handle = es.register_token(&token).unwrap().raw;

        let ok = es.intercept_network(&[93, 184, 216, 34], 443, handle);
        assert!(ok.is_allowed());

        let bad = es.intercept_network(&[1, 1, 1, 1], 443, handle);
        assert!(matches!(
            bad,
            InterceptVerdict::Deny(InterceptError::ScopeMismatch)
                | InterceptVerdict::Deny(InterceptError::Gate(_))
        ));
    }
}
use std::fs;
use std::sync::{Arc, Mutex};

use intentkernel_core::gate::{SyscallRequest, SyscallResult};
use intentkernel_core::IntentKernel;
use intentkernel_crypto::cbor::decode_wire_token;
use intentkernel_crypto::sign::{PublicKey, TokenValidator};
use intentkernel_crypto::token::Algorithm;
use intentkernel_util::paths::resolve_root;
use tonic::{Request, Response, Status};

use crate::intentkernel::v1::{
    capability_service_server::CapabilityService, Empty, InvokeCapabilityRequest,
    InvokeCapabilityResponse, KernelStats, RegisterTokenRequest, RegisterTokenResponse,
    RevokeHandleRequest, RevokeHandleResponse,
};

/// Shared IntentKernel state wired to the syscall gate for gRPC services.
pub struct KernelState {
    pub kernel: Arc<Mutex<IntentKernel>>,
    pub validator: Option<TokenValidator>,
}

impl KernelState {
    pub fn new() -> Self {
        Self {
            kernel: Arc::new(Mutex::new(IntentKernel::new())),
            validator: load_validator().ok(),
        }
    }

    pub fn invoke(&self, handle: u64, sequence: u64, action: u32) -> InvokeCapabilityResponse {
        let mut kernel = match self.kernel.lock() {
            Ok(k) => k,
            Err(_) => {
                return InvokeCapabilityResponse {
                    allowed: false,
                    resource_type: 0,
                    denial_reason: "kernel lock poisoned".into(),
                };
            }
        };

        match kernel.gate().invoke(SyscallRequest {
            handle,
            sequence,
            action,
        }) {
            SyscallResult::Allowed { resource_type } => InvokeCapabilityResponse {
                allowed: true,
                resource_type,
                denial_reason: String::new(),
            },
            SyscallResult::Denied(e) => InvokeCapabilityResponse {
                allowed: false,
                resource_type: 0,
                denial_reason: e.to_string(),
            },
        }
    }

    pub fn invoke_auto_sequence(&self, handle: u64, action: u32) -> InvokeCapabilityResponse {
        let seq = self
            .kernel
            .lock()
            .ok()
            .and_then(|k| k.sequences.get(&handle).copied())
            .unwrap_or(0)
            + 1;
        self.invoke(handle, seq, action)
    }
}

pub struct CapabilityServiceImpl {
    state: Arc<KernelState>,
}

impl CapabilityServiceImpl {
    pub fn new(state: Arc<KernelState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl CapabilityService for CapabilityServiceImpl {
    async fn register_token(
        &self,
        req: Request<RegisterTokenRequest>,
    ) -> Result<Response<RegisterTokenResponse>, Status> {
        let inner = req.into_inner();
        let validator = self
            .state
            .validator
            .as_ref()
            .ok_or_else(|| Status::failed_precondition("broker pubkey not configured"))?;

        let token = decode_wire_token(&inner.token_cbor)
            .map_err(|e| Status::invalid_argument(format!("invalid token CBOR: {e}")))?;

        let mut kernel = self
            .state
            .kernel
            .lock()
            .map_err(|_| Status::internal("kernel lock"))?;

        match kernel
            .gate()
            .register_token(&token, validator, inner.resource_type)
        {
            Ok(handle) => Ok(Response::new(RegisterTokenResponse {
                handle: handle.raw,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(RegisterTokenResponse {
                handle: 0,
                error: e.to_string(),
            })),
        }
    }

    async fn invoke_capability(
        &self,
        req: Request<InvokeCapabilityRequest>,
    ) -> Result<Response<InvokeCapabilityResponse>, Status> {
        let inner = req.into_inner();
        Ok(Response::new(self.state.invoke(
            inner.handle,
            inner.sequence,
            inner.action,
        )))
    }

    async fn revoke_handle(
        &self,
        req: Request<RevokeHandleRequest>,
    ) -> Result<Response<RevokeHandleResponse>, Status> {
        let handle = req.into_inner().handle;
        let mut kernel = self
            .state
            .kernel
            .lock()
            .map_err(|_| Status::internal("kernel lock"))?;

        match kernel.gate().revoke_handle(handle) {
            Ok(()) => Ok(Response::new(RevokeHandleResponse {
                ok: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(RevokeHandleResponse {
                ok: false,
                error: e.to_string(),
            })),
        }
    }

    async fn get_kernel_stats(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<KernelStats>, Status> {
        let kernel = self
            .state
            .kernel
            .lock()
            .map_err(|_| Status::internal("kernel lock"))?;
        let stats = kernel.stats();
        Ok(Response::new(KernelStats {
            active_capabilities: stats.active_capabilities as u32,
            registered_handles: stats.registered_handles as u32,
        }))
    }
}

fn load_validator() -> anyhow::Result<TokenValidator> {
    let root = resolve_root(None);
    let path = root.join("config/broker.key.json");
    let text = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
    let keyfile: serde_json::Value = serde_json::from_str(&text)?;
    let alg = match keyfile["algorithm"].as_str().unwrap_or("ed25519") {
        "ed25519" => Algorithm::Ed25519,
        "ml-dsa-87" | "mldsa87" => Algorithm::MlDsa87,
        other => anyhow::bail!("unknown algorithm {other}"),
    };
    let pk = hex::decode(
        keyfile["public_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing public_key"))?,
    )?;
    Ok(TokenValidator::new(PublicKey {
        algorithm: alg,
        bytes: pk,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use intentkernel_crypto::sign::{KeyPair, TokenIssuer};
    use intentkernel_crypto::token::{FileScope, ResourceScope, TrustAnchor};
    use intentkernel_crypto::sign::token_to_bytes;

    fn test_state_with_keypair(kp: &KeyPair) -> Arc<KernelState> {
        let validator = TokenValidator::new(PublicKey {
            algorithm: Algorithm::Ed25519,
            bytes: kp.public_key.clone(),
        });
        Arc::new(KernelState {
            kernel: Arc::new(Mutex::new(IntentKernel::new())),
            validator: Some(validator),
        })
    }

    #[test]
    fn register_invoke_revoke_round_trip() {
        let kp = KeyPair::generate(Algorithm::Ed25519).unwrap();
        let state = test_state_with_keypair(&kp);
        let issuer = TokenIssuer::new(kp);
        let token = issuer
            .issue_capability(
                b"test-subject",
                b"open_file",
                ResourceScope::File(FileScope {
                    path: "/tmp/test.txt".into(),
                    access: 1,
                    inode: None,
                }),
                TrustAnchor::UiEvent,
                60_000,
                2,
            )
            .unwrap();
        let token_cbor = token_to_bytes(&token).unwrap();

        let validator = state.validator.as_ref().unwrap();
        let mut kernel = state.kernel.lock().unwrap();
        let handle = kernel
            .gate()
            .register_token(&token, validator, 42)
            .unwrap();
        drop(kernel);

        let resp = state.invoke_auto_sequence(handle.raw, 1);
        assert!(resp.allowed);
        assert_eq!(resp.resource_type, 42);

        let mut kernel = state.kernel.lock().unwrap();
        kernel.gate().revoke_handle(handle.raw).unwrap();
        let stats = kernel.stats();
        assert_eq!(stats.registered_handles, 0);
        drop(kernel);

        let denied = state.invoke_auto_sequence(handle.raw, 1);
        assert!(!denied.allowed);
        assert!(!token_cbor.is_empty());
    }

    #[test]
    fn kernel_stats_empty() {
        let state = KernelState::new();
        let kernel = state.kernel.lock().unwrap();
        let stats = kernel.stats();
        assert_eq!(stats.active_capabilities, 0);
        assert_eq!(stats.registered_handles, 0);
    }
}
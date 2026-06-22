use intentkernel_crypto::token::WireToken;
use intentkernel_crypto::sign::{PublicKey, TokenValidator};
use thiserror::Error;

use crate::capability::CapabilityError;
use crate::handle::{HandleError, KernelHandle};
use crate::kernel::IntentKernel;

#[derive(Debug, Clone)]
pub struct SyscallRequest {
    pub handle: u64,
    pub sequence: u64,
    pub action: u32,
}

#[derive(Debug)]
pub enum SyscallResult {
    Allowed { resource_type: u32 },
    Denied(SyscallError),
}

#[derive(Debug, Error, Clone)]
pub enum SyscallError {
    #[error("capability: {0}")]
    Capability(#[from] CapabilityError),
    #[error("handle: {0}")]
    Handle(#[from] HandleError),
    #[error("token: {0}")]
    Token(String),
    #[error("replay detected")]
    Replay,
}

pub struct SyscallGate<'a> {
    kernel: &'a mut IntentKernel,
}

impl<'a> SyscallGate<'a> {
    pub fn new(kernel: &'a mut IntentKernel) -> Self {
        Self { kernel }
    }

    pub fn register_token(
        &mut self,
        token: &WireToken,
        verifier: &TokenValidator,
        resource_type: u32,
    ) -> Result<KernelHandle, SyscallError> {
        verifier
            .validate(token)
            .map_err(|e| SyscallError::Token(e.to_string()))?;
        let ttl = token.payload.exp.saturating_sub(token.payload.nbf);
        let uses = token.payload.uses.min(u16::MAX as u32) as u16;
        let id = self
            .kernel
            .capabilities
            .create(resource_type, ttl, uses)
            .map_err(SyscallError::Capability)?;
        let handle = self.kernel.handles.mint(id);
        self.kernel
            .token_bindings
            .insert(handle.raw, (id, token.header.anchor as u32));
        Ok(handle)
    }

    pub fn invoke(&mut self, req: SyscallRequest) -> SyscallResult {
        let handle = match KernelHandle::decode(req.handle) {
            Ok(h) => h,
            Err(e) => return SyscallResult::Denied(SyscallError::Handle(e)),
        };

        if let Err(e) = self.kernel.handles.verify(handle) {
            return SyscallResult::Denied(SyscallError::Handle(e));
        }

        if let Some(last_seq) = self.kernel.sequences.get(&req.handle) {
            if *last_seq >= req.sequence {
                return SyscallResult::Denied(SyscallError::Replay);
            }
        }
        self.kernel.sequences.insert(req.handle, req.sequence);

        let slot = match self.kernel.capabilities.get(handle.table_index as u16) {
            Some(c) => c.clone(),
            None => {
                return SyscallResult::Denied(SyscallError::Capability(
                    CapabilityError::Expired,
                ));
            }
        };

        match self.kernel.capabilities.validate(&slot) {
            Ok(resource_type) => SyscallResult::Allowed { resource_type },
            Err(e) => SyscallResult::Denied(SyscallError::Capability(e)),
        }
    }

    pub fn revoke_handle(&mut self, raw_handle: u64) -> Result<(), SyscallError> {
        let handle = KernelHandle::decode(raw_handle)?;
        self.kernel.handles.verify(handle)?;
        self.kernel
            .capabilities
            .revoke(handle.table_index as u16)
            .map_err(SyscallError::Capability)?;
        self.kernel.handles.invalidate(handle.table_index as u16);
        self.kernel.token_bindings.remove(&raw_handle);
        self.kernel.sequences.remove(&raw_handle);
        Ok(())
    }
}

pub fn make_validator(public_key: &[u8], algorithm: intentkernel_crypto::token::Algorithm) -> TokenValidator {
    TokenValidator::new(PublicKey {
        algorithm,
        bytes: public_key.to_vec(),
    })
}
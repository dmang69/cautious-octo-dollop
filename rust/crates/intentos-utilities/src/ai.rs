use intentos_kernel::KernelError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct AiRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiResponse {
    pub output: String,
    pub model: String,
}

#[derive(Debug, Error)]
pub enum AiError {
    #[error("kernel: {0}")]
    Kernel(#[from] KernelError),
    #[error("ai scope mismatch")]
    ScopeDenied,
}

pub struct AiGateway;

impl AiGateway {
    pub fn infer_gated(
        prompt: &str,
        handle: u64,
        kernel: &intentos_kernel::Kernel,
    ) -> Result<AiResponse, AiError> {
        let bound = kernel
            .binding_resource(handle)
            .ok_or_else(|| AiError::Kernel(KernelError::Denied("no binding".into())))?;
        if !bound.starts_with("ai:") {
            return Err(AiError::ScopeDenied);
        }
        Ok(AiResponse {
            output: format!("[stub] echo: {prompt}"),
            model: "intentos-stub-v0".into(),
        })
    }
}
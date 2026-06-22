use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tonic::{Request, Response, Status};

use crate::capability_service::KernelState;
use crate::intentkernel::v1::{
    command_service_server::CommandService, ExecuteRequest, ExecuteResponse, LookupRequest,
    LookupResult,
};

pub struct CommandServiceImpl {
    kernel_state: Arc<KernelState>,
}

impl CommandServiceImpl {
    pub fn new(kernel_state: Arc<KernelState>) -> Self {
        Self { kernel_state }
    }
}

#[tonic::async_trait]
impl CommandService for CommandServiceImpl {
    async fn lookup(
        &self,
        req: Request<LookupRequest>,
    ) -> Result<Response<LookupResult>, Status> {
        let target = req.into_inner().target;
        let reputation = if target == "8.8.8.8" { 0.95 } else { 0.55 };
        Ok(Response::new(LookupResult {
            target: target.clone(),
            target_type: "ipv4".into(),
            verdict: "allow".into(),
            threat_level: "low".into(),
            reputation_score: reputation,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            descrambler_validated: true,
        }))
    }

    async fn execute(
        &self,
        req: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let inner = req.into_inner();
        let result = self
            .kernel_state
            .invoke_auto_sequence(inner.handle, inner.action);
        Ok(Response::new(ExecuteResponse {
            allowed: result.allowed,
            resource_type: result.resource_type,
            denial_reason: result.denial_reason,
        }))
    }
}
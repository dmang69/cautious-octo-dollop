use std::sync::RwLock;

use tonic::{Request, Response, Status};

use crate::intentkernel::v1::{
    scheduler_service_server::SchedulerService, Empty, SchedulerOptimizeRequest, SchedulerPolicy,
};

pub struct SchedulerServiceImpl {
    policy: RwLock<SchedulerPolicy>,
}

impl SchedulerServiceImpl {
    pub fn new() -> Self {
        Self {
            policy: RwLock::new(SchedulerPolicy {
                time_slices_ms: vec![4.0, 8.0, 16.0, 32.0],
            }),
        }
    }
}

#[tonic::async_trait]
impl SchedulerService for SchedulerServiceImpl {
    async fn get_current_policy(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<SchedulerPolicy>, Status> {
        let p = self.policy.read().map_err(|_| Status::internal("lock"))?;
        Ok(Response::new(p.clone()))
    }

    async fn optimize_policy(
        &self,
        req: Request<SchedulerOptimizeRequest>,
    ) -> Result<Response<SchedulerPolicy>, Status> {
        let load = req.into_inner().telemetry.first().copied().unwrap_or(25.0);
        let factor = (load / 100.0).clamp(0.5, 2.0) as f64;
        let policy = SchedulerPolicy {
            time_slices_ms: vec![4.0 * factor, 8.0 * factor, 16.0 * factor, 32.0 * factor],
        };
        *self.policy.write().map_err(|_| Status::internal("lock"))? = policy.clone();
        Ok(Response::new(policy))
    }
}
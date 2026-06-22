use std::net::SocketAddr;
use std::sync::Arc;

use crate::capability_service::{CapabilityServiceImpl, KernelState};
use crate::lookup::CommandServiceImpl;
use crate::scheduler::SchedulerServiceImpl;
use crate::telemetry::TelemetryServiceImpl;
use crate::intentkernel::v1::capability_service_server::CapabilityServiceServer;
use crate::intentkernel::v1::command_service_server::CommandServiceServer;
use crate::intentkernel::v1::scheduler_service_server::SchedulerServiceServer;
use crate::intentkernel::v1::telemetry_service_server::TelemetryServiceServer;
use intentkernel_platform::stop_requested;
use tonic::transport::Server;

pub async fn run() -> anyhow::Result<()> {
    let addr: SocketAddr = std::env::var("AI_RUNTIME_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:50051".to_string())
        .parse()?;

    tracing::info!(%addr, "ai-runtime gRPC server starting");

    let kernel_state = Arc::new(KernelState::new());

    Server::builder()
        .add_service(TelemetryServiceServer::new(TelemetryServiceImpl::new()))
        .add_service(SchedulerServiceServer::new(SchedulerServiceImpl::new()))
        .add_service(CapabilityServiceServer::new(CapabilityServiceImpl::new(
            kernel_state.clone(),
        )))
        .add_service(CommandServiceServer::new(CommandServiceImpl::new(
            kernel_state,
        )))
        .serve_with_shutdown(addr, async {
            while !stop_requested() {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
            tracing::info!("shutdown requested");
        })
        .await?;

    Ok(())
}
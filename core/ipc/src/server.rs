use anyhow::Result;
use tonic::transport::Server;
use tracing::info;

use crate::proto::{
    scheduler_service_server::{SchedulerService, SchedulerServiceServer},
    SchedulerRequest, SchedulerResponse,
};

#[derive(Default)]
pub struct SchedulerServiceImpl;

#[tonic::async_trait]
impl SchedulerService for SchedulerServiceImpl {
    async fn get_recommendations(
        &self,
        _request: tonic::Request<SchedulerRequest>,
    ) -> Result<tonic::Response<SchedulerResponse>, tonic::Status> {
        Ok(tonic::Response::new(SchedulerResponse {
            recommendations: vec![],
        }))
    }

    type StreamRecommendationsStream =
        tokio_stream::wrappers::ReceiverStream<Result<crate::proto::PriorityRecommendation, tonic::Status>>;

    async fn stream_recommendations(
        &self,
        _request: tonic::Request<crate::proto::TelemetrySnapshot>,
    ) -> Result<tonic::Response<Self::StreamRecommendationsStream>, tonic::Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        drop(tx);
        Ok(tonic::Response::new(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        ))
    }
}

/// Start the gRPC server on the given address.
pub async fn serve(addr: &str) -> Result<()> {
    let addr = addr.parse()?;
    info!("gRPC server listening on {}", addr);
    Server::builder()
        .add_service(SchedulerServiceServer::new(SchedulerServiceImpl))
        .serve(addr)
        .await?;
    Ok(())
}

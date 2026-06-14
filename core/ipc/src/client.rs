use anyhow::Result;
use tonic::transport::Channel;

use crate::proto::{
    scheduler_service_client::SchedulerServiceClient, SchedulerRequest, TelemetrySnapshot,
};

pub struct AiOsClient {
    scheduler: SchedulerServiceClient<Channel>,
}

impl AiOsClient {
    /// Connect to the AI Runtime gRPC server.
    pub async fn connect(addr: &str) -> Result<Self> {
        let channel = Channel::from_shared(addr.to_string())?.connect().await?;
        Ok(Self {
            scheduler: SchedulerServiceClient::new(channel),
        })
    }

    /// Fetch scheduling recommendations for a telemetry snapshot.
    pub async fn get_recommendations(
        &mut self,
        snapshot: TelemetrySnapshot,
    ) -> Result<Vec<crate::proto::PriorityRecommendation>> {
        let request = tonic::Request::new(SchedulerRequest {
            snapshot: Some(snapshot),
        });
        let response = self.scheduler.get_recommendations(request).await?;
        Ok(response.into_inner().recommendations)
    }
}

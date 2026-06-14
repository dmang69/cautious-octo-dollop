use crate::{inference::InferenceEngine, telemetry::TelemetryCollector};
use anyhow::Result;
use tracing::{error, info};

/// Top-level context manager that wires telemetry → inference → gRPC.
pub struct ContextManager {
    telemetry: TelemetryCollector,
    inference: InferenceEngine,
}

impl ContextManager {
    pub async fn new() -> Result<Self> {
        let telemetry = TelemetryCollector::new()?;
        let inference = InferenceEngine::new()?;
        Ok(Self { telemetry, inference })
    }

    /// Main event loop — collect metrics, run inference, publish via gRPC.
    pub async fn run(&self) -> Result<()> {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_millis(500));
        loop {
            interval.tick().await;
            match self.tick().await {
                Ok(_) => {}
                Err(e) => error!("context manager tick error: {}", e),
            }
        }
    }

    async fn tick(&self) -> Result<()> {
        let snapshot = self.telemetry.collect()?;
        info!(
            cpu = snapshot.cpu_avg,
            mem_used = snapshot.memory_used_bytes,
            "telemetry snapshot"
        );
        let _recommendations = self.inference.suggest_priorities(&snapshot)?;
        Ok(())
    }
}

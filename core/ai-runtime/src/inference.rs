use crate::telemetry::TelemetrySnapshot;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A priority adjustment recommended by the inference model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityRecommendation {
    pub pid: u32,
    pub suggested_priority: i32,
    pub confidence: f32,
}

/// Wraps the ONNX Runtime session for the scheduler model.
pub struct InferenceEngine {
    // session: ort::Session  — initialized when ORT_LIB_LOCATION is set
}

impl InferenceEngine {
    pub fn new() -> Result<Self> {
        // TODO: load ONNX session from config model_dir
        Ok(Self {})
    }

    /// Run the scheduler model and return priority adjustments.
    pub fn suggest_priorities(
        &self,
        snapshot: &TelemetrySnapshot,
    ) -> Result<Vec<PriorityRecommendation>> {
        // TODO: encode snapshot as input tensor, run session, decode output
        let _ = snapshot;
        Ok(vec![])
    }
}

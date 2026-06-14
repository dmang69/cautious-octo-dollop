use anyhow::Result;
use kernel_interface::KernelInterface;
use serde::{Deserialize, Serialize};

/// A point-in-time snapshot of system telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub timestamp_ms: u64,
    pub cpu_avg: f32,
    pub cpu_per_core: Vec<f32>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub process_count: usize,
}

pub struct TelemetryCollector {
    ki: Box<dyn KernelInterface>,
}

impl TelemetryCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ki: kernel_interface::platform(),
        })
    }

    pub fn collect(&self) -> Result<TelemetrySnapshot> {
        let mem = self.ki.memory_stats()?;
        let cpu = self.ki.cpu_usage()?;
        let procs = self.ki.list_processes()?;
        let cpu_avg = if cpu.is_empty() {
            0.0
        } else {
            cpu.iter().sum::<f32>() / cpu.len() as f32
        };
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(TelemetrySnapshot {
            timestamp_ms,
            cpu_avg,
            cpu_per_core: cpu,
            memory_used_bytes: mem.used_bytes,
            memory_total_bytes: mem.total_bytes,
            process_count: procs.len(),
        })
    }
}

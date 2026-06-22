use serde::Serialize;

use crate::intentkernel::v1::{
    ExecuteResponse, InvokeCapabilityResponse, KernelStats, LookupResult, RegisterTokenResponse,
    RevokeHandleResponse, SchedulerPolicy, SystemSnapshot,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshotDto {
    pub cpu_percent: f32,
    pub mem_percent: f32,
    pub disk_io_mbps: f32,
    pub net_rx_mbps: f32,
    pub net_tx_mbps: f32,
    pub queue_depth: u32,
    pub uptime_secs: u64,
    pub process_count: u32,
    pub ipc_queued: u32,
}

impl From<SystemSnapshot> for SystemSnapshotDto {
    fn from(s: SystemSnapshot) -> Self {
        Self {
            cpu_percent: s.cpu_percent,
            mem_percent: s.mem_percent,
            disk_io_mbps: s.disk_io_mbps,
            net_rx_mbps: s.net_rx_mbps,
            net_tx_mbps: s.net_tx_mbps,
            queue_depth: s.queue_depth,
            uptime_secs: s.uptime_secs,
            process_count: s.process_count,
            ipc_queued: s.ipc_queued,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatusDto {
    pub connected: bool,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerPolicyDto {
    pub time_slices_ms: Vec<f64>,
}

impl From<SchedulerPolicy> for SchedulerPolicyDto {
    fn from(p: SchedulerPolicy) -> Self {
        Self {
            time_slices_ms: p.time_slices_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupResultDto {
    pub target: String,
    pub target_type: String,
    pub verdict: String,
    pub threat_level: String,
    pub reputation_score: f32,
    pub timestamp: u64,
    pub descrambler_validated: bool,
}

impl From<LookupResult> for LookupResultDto {
    fn from(r: LookupResult) -> Self {
        Self {
            target: r.target,
            target_type: r.target_type,
            verdict: r.verdict,
            threat_level: r.threat_level,
            reputation_score: r.reputation_score,
            timestamp: r.timestamp,
            descrambler_validated: r.descrambler_validated,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResultDto {
    pub allowed: bool,
    pub resource_type: u32,
    pub denial_reason: String,
}

impl From<ExecuteResponse> for ExecuteResultDto {
    fn from(r: ExecuteResponse) -> Self {
        Self {
            allowed: r.allowed,
            resource_type: r.resource_type,
            denial_reason: r.denial_reason,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GrpcKernelStatsDto {
    pub active_capabilities: u32,
    pub registered_handles: u32,
}

impl From<KernelStats> for GrpcKernelStatsDto {
    fn from(s: KernelStats) -> Self {
        Self {
            active_capabilities: s.active_capabilities,
            registered_handles: s.registered_handles,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokeCapabilityResultDto {
    pub allowed: bool,
    pub resource_type: u32,
    pub denial_reason: String,
}

impl From<InvokeCapabilityResponse> for InvokeCapabilityResultDto {
    fn from(r: InvokeCapabilityResponse) -> Self {
        Self {
            allowed: r.allowed,
            resource_type: r.resource_type,
            denial_reason: r.denial_reason,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeHandleResultDto {
    pub ok: bool,
    pub error: String,
}

impl From<RevokeHandleResponse> for RevokeHandleResultDto {
    fn from(r: RevokeHandleResponse) -> Self {
        Self {
            ok: r.ok,
            error: r.error,
        }
    }
}

impl RegisterTokenResponse {
    pub fn into_handle_hex(self) -> Result<String, String> {
        if !self.error.is_empty() {
            return Err(self.error);
        }
        if self.handle == 0 {
            return Err("register returned handle 0".into());
        }
        Ok(format!("0x{:016X}", self.handle))
    }
}
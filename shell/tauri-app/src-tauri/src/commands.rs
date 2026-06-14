use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub priority: i32,
    pub suggested_priority: Option<i32>,
}

#[derive(Serialize)]
pub struct TelemetrySnapshot {
    pub timestamp_ms: u64,
    pub cpu_avg: f32,
    pub cpu_per_core: Vec<f32>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub process_count: usize,
}

#[derive(Serialize)]
pub struct CommandResult {
    pub intent: String,
    pub structured_json: String,
    pub confidence: f32,
}

#[tauri::command]
pub async fn run_command(command: String, _state: State<'_, AppState>) -> Result<String, String> {
    // Execute the command in a shell and capture stdout/stderr.
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(format!("{}{}", stdout, stderr))
}

#[tauri::command]
pub async fn list_processes(_state: State<'_, AppState>) -> Result<Vec<ProcessInfo>, String> {
    // TODO: query AI Runtime via gRPC client
    Ok(vec![])
}

#[tauri::command]
pub async fn set_priority(
    pid: u32,
    priority: i32,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: call AI Runtime set_priority gRPC endpoint
    let _ = (pid, priority);
    Ok(())
}

#[tauri::command]
pub async fn get_telemetry_snapshot(
    _state: State<'_, AppState>,
) -> Result<TelemetrySnapshot, String> {
    // TODO: call TelemetryService.GetSnapshot via gRPC
    Ok(TelemetrySnapshot {
        timestamp_ms: 0,
        cpu_avg: 0.0,
        cpu_per_core: vec![],
        memory_used_bytes: 0,
        memory_total_bytes: 0,
        process_count: 0,
    })
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotPayload {
    pub timestamp_ms: u64,
    pub cpu_avg: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub process_count: usize,
}

#[tauri::command]
pub async fn get_scheduler_recommendations(
    _snapshot: SnapshotPayload,
    _state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    // TODO: forward snapshot to SchedulerService via gRPC
    Ok(vec![])
}

#[tauri::command]
pub async fn interpret_command(
    raw_command: String,
    _state: State<'_, AppState>,
) -> Result<CommandResult, String> {
    // TODO: forward to CommandInterpreterService via gRPC
    Ok(CommandResult {
        intent: raw_command,
        structured_json: "{}".into(),
        confidence: 0.0,
    })
}

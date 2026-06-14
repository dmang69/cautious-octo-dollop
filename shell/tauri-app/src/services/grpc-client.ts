import { invoke } from "@tauri-apps/api/tauri";

export interface TelemetrySnapshot {
  timestamp_ms: number;
  cpu_avg: number;
  cpu_per_core: number[];
  memory_used_bytes: number;
  memory_total_bytes: number;
  process_count: number;
}

export interface PriorityRecommendation {
  pid: number;
  suggested_priority: number;
  confidence: number;
}

/** Fetch a single telemetry snapshot from the Rust backend. */
export async function getTelemetrySnapshot(): Promise<TelemetrySnapshot> {
  return invoke<TelemetrySnapshot>("get_telemetry_snapshot");
}

/** Request scheduling recommendations from the AI Runtime. */
export async function getSchedulerRecommendations(
  snapshot: TelemetrySnapshot
): Promise<PriorityRecommendation[]> {
  return invoke<PriorityRecommendation[]>("get_scheduler_recommendations", { snapshot });
}

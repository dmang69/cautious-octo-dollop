#[cfg(test)]
mod tests {
    use kernel_interface::KernelInterface;

    fn make_platform() -> Box<dyn KernelInterface> {
        kernel_interface::platform()
    }

    #[test]
    fn test_list_processes_returns_some() {
        let ki = make_platform();
        let procs = ki.list_processes().expect("list_processes should not fail");
        // On any OS there is at least the current process.
        assert!(!procs.is_empty(), "process list should be non-empty");
    }

    #[test]
    fn test_memory_stats_non_zero_total() {
        let ki = make_platform();
        let stats = ki.memory_stats().expect("memory_stats should not fail");
        assert!(stats.total_bytes > 0, "total_bytes should be > 0");
    }

    #[test]
    fn test_cpu_usage_returns_cores() {
        let ki = make_platform();
        let cores = ki.cpu_usage().expect("cpu_usage should not fail");
        assert!(!cores.is_empty(), "cpu_usage should return at least one core");
        for &u in &cores {
            assert!((0.0..=1.0).contains(&u), "cpu usage per core must be in [0,1]");
        }
    }

    #[test]
    fn test_scheduler_suggest_priorities() {
        use ai_runtime::inference::InferenceEngine;
        use ai_runtime::telemetry::TelemetrySnapshot;

        let engine = InferenceEngine::new().unwrap();
        let snap = TelemetrySnapshot {
            timestamp_ms: 0,
            cpu_avg: 0.5,
            cpu_per_core: vec![0.5],
            memory_used_bytes: 1_000_000,
            memory_total_bytes: 8_000_000,
            process_count: 42,
        };
        let recs = engine.suggest_priorities(&snap).unwrap();
        // Stub returns empty; assert it doesn't panic.
        let _ = recs;
    }
}

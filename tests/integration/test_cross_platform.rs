#[cfg(test)]
mod tests {
    /// Verify the kernel-interface platform() function compiles and runs on the
    /// current OS without panicking.
    #[test]
    fn test_platform_factory_does_not_panic() {
        let _ki = kernel_interface::platform();
    }

    #[test]
    fn test_process_info_serialization() {
        use kernel_interface::ProcessInfo;
        let info = ProcessInfo {
            pid: 1,
            name: "init".to_string(),
            cpu_usage: 0.1,
            memory_bytes: 4096,
            priority: 0,
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: ProcessInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.pid, 1);
        assert_eq!(back.name, "init");
    }

    #[test]
    fn test_memory_stats_consistency() {
        let ki = kernel_interface::platform();
        let stats = ki.memory_stats().unwrap();
        // available_bytes must not exceed total_bytes
        assert!(
            stats.available_bytes <= stats.total_bytes,
            "available_bytes must not exceed total_bytes"
        );
        // used_bytes must not exceed total_bytes
        assert!(
            stats.used_bytes <= stats.total_bytes,
            "used_bytes must not exceed total_bytes"
        );
    }
}

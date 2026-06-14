pub mod linux;
pub mod macos;
pub mod windows;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Snapshot of a running process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub priority: i32,
}

/// System-level memory statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
}

/// Unified OS abstraction for the AI runtime.
pub trait KernelInterface: Send + Sync {
    /// Return a snapshot of all running processes.
    fn list_processes(&self) -> Result<Vec<ProcessInfo>>;

    /// Adjust the scheduling priority of a process.
    fn set_priority(&self, pid: u32, priority: i32) -> Result<()>;

    /// Return current memory statistics.
    fn memory_stats(&self) -> Result<MemoryStats>;

    /// Return CPU utilization per core (0.0–1.0).
    fn cpu_usage(&self) -> Result<Vec<f32>>;
}

/// Return the platform-specific implementation.
pub fn platform() -> Box<dyn KernelInterface> {
    #[cfg(target_os = "linux")]
    return Box::new(linux::LinuxKernel::new());

    #[cfg(target_os = "windows")]
    return Box::new(windows::WindowsKernel::new());

    #[cfg(target_os = "macos")]
    return Box::new(macos::MacOsKernel::new());

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    compile_error!("Unsupported platform");
}

use crate::{KernelInterface, MemoryStats, ProcessInfo};
use anyhow::Result;

pub struct WindowsKernel;

impl WindowsKernel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelInterface for WindowsKernel {
    fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
        // TODO: enumerate via CreateToolhelp32Snapshot / Process32First
        Ok(vec![])
    }

    fn set_priority(&self, _pid: u32, _priority: i32) -> Result<()> {
        // TODO: SetPriorityClass
        Ok(())
    }

    fn memory_stats(&self) -> Result<MemoryStats> {
        // TODO: GlobalMemoryStatusEx
        Ok(MemoryStats {
            total_bytes: 0,
            available_bytes: 0,
            used_bytes: 0,
        })
    }

    fn cpu_usage(&self) -> Result<Vec<f32>> {
        // TODO: PDH or WMI query
        Ok(vec![0.0])
    }
}

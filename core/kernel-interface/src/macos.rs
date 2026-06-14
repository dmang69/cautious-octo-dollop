use crate::{KernelInterface, MemoryStats, ProcessInfo};
use anyhow::Result;

pub struct MacOsKernel;

impl MacOsKernel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelInterface for MacOsKernel {
    fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
        // TODO: sysctl CTL_KERN / KERN_PROC
        Ok(vec![])
    }

    fn set_priority(&self, pid: u32, priority: i32) -> Result<()> {
        let result = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid, priority) };
        if result == -1 {
            return Err(anyhow::anyhow!(
                "setpriority failed for pid {}: {}",
                pid,
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }

    fn memory_stats(&self) -> Result<MemoryStats> {
        // TODO: host_statistics64 / vm_statistics64
        Ok(MemoryStats {
            total_bytes: 0,
            available_bytes: 0,
            used_bytes: 0,
        })
    }

    fn cpu_usage(&self) -> Result<Vec<f32>> {
        // TODO: host_processor_info
        Ok(vec![0.0])
    }
}

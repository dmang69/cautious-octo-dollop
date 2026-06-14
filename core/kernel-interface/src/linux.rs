use crate::{KernelInterface, MemoryStats, ProcessInfo};
use anyhow::Result;
use std::fs;

pub struct LinuxKernel;

impl LinuxKernel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelInterface for LinuxKernel {
    fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
        let mut processes = Vec::new();
        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if let Ok(pid) = name_str.parse::<u32>() {
                let comm_path = format!("/proc/{}/comm", pid);
                let proc_name = fs::read_to_string(&comm_path)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                processes.push(ProcessInfo {
                    pid,
                    name: proc_name,
                    cpu_usage: 0.0,
                    memory_bytes: 0,
                    priority: 0,
                });
            }
        }
        Ok(processes)
    }

    fn set_priority(&self, pid: u32, priority: i32) -> Result<()> {
        let result = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid, priority) };
        if result != 0 {
            anyhow::bail!("setpriority failed for pid {}: errno={}", pid, result);
        }
        Ok(())
    }

    fn memory_stats(&self) -> Result<MemoryStats> {
        let content = fs::read_to_string("/proc/meminfo")?;
        let mut total = 0u64;
        let mut available = 0u64;
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_kb(line)? * 1024;
            } else if line.starts_with("MemAvailable:") {
                available = parse_kb(line)? * 1024;
            }
        }
        Ok(MemoryStats {
            total_bytes: total,
            available_bytes: available,
            used_bytes: total.saturating_sub(available),
        })
    }

    fn cpu_usage(&self) -> Result<Vec<f32>> {
        // Read /proc/stat for per-core utilization; return placeholder for now.
        Ok(vec![0.0])
    }
}

fn parse_kb(line: &str) -> Result<u64> {
    let kb = line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("invalid /proc/meminfo line"))?
        .parse::<u64>()?;
    Ok(kb)
}

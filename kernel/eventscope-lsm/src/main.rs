//! EventScope LSM userspace daemon — coordinates BPF map writes and policy checks.
//!
//! Without root/CAP_BPF the daemon runs in mock mode using the in-memory bridge.

use std::io::{self, BufRead, Write};

use anyhow::{Context, Result};
use clap::Parser;
use eventscope_ebpf::bridge::{replace_global_bridge, MockKernelBridge};
use eventscope_ebpf::loader::{probe_loader_status, try_load_bpf};
use eventscope_ebpf::policy::{evaluate_hook, HandleMapEntry, SyscallHook};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "eventscope-lsm", about = "IntentKernel LSM + eBPF policy daemon")]
struct Args {
    /// Attempt to load and attach BPF programs (requires root/CAP_BPF).
    #[arg(long)]
    load_bpf: bool,

    /// Run with in-memory map only (default on WSL2 without BPF).
    #[arg(long, default_value_t = true)]
    mock: bool,

    /// Read JSON lines from stdin: {"op":"publish","pid":1,"handle":42,"resource_type":1}
    #[arg(long)]
    stdin_json: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum DaemonRequest {
    Publish {
        pid: u32,
        handle: u64,
        resource_type: u32,
    },
    Revoke { pid: u32 },
    Check {
        pid: u32,
        hook: String,
    },
}

#[derive(Debug, serde::Serialize)]
struct DaemonResponse {
    ok: bool,
    verdict: Option<String>,
    message: Option<String>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    if args.load_bpf {
        match try_load_bpf() {
            Ok(()) => info!("BPF LSM programs attached"),
            Err(e) => {
                warn!("BPF load failed ({e}); falling back to mock map");
                if args.mock {
                    replace_global_bridge(Box::new(MockKernelBridge::new()));
                }
            }
        }
    } else if args.mock {
        replace_global_bridge(Box::new(MockKernelBridge::new()));
        info!("mock handle_map active (no kernel enforcement)");
    }

    if args.stdin_json {
        run_stdin_loop()?;
    } else {
        print_status()?;
    }

    Ok(())
}

fn print_status() -> Result<()> {
    let status = probe_loader_status();
    println!("eventscope-lsm status: {status:?}");
    println!("Publish handles via EventScope::publish_handle_to_kernel or --stdin-json");
    Ok(())
}

fn run_stdin_loop() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.context("stdin read")?;
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_request(&line);
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }
    Ok(())
}

fn handle_request(line: &str) -> DaemonResponse {
    let req: DaemonRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return DaemonResponse {
                ok: false,
                verdict: None,
                message: Some(format!("parse error: {e}")),
            };
        }
    };

    match req {
        DaemonRequest::Publish {
            pid,
            handle,
            resource_type,
        } => {
            let entry = HandleMapEntry::new(pid, handle, resource_type);
            match eventscope_ebpf::publish_handle(entry) {
                Ok(()) => DaemonResponse {
                    ok: true,
                    verdict: None,
                    message: Some("published".into()),
                },
                Err(e) => DaemonResponse {
                    ok: false,
                    verdict: None,
                    message: Some(e.to_string()),
                },
            }
        }
        DaemonRequest::Revoke { pid } => match eventscope_ebpf::revoke_pid(pid) {
            Ok(()) => DaemonResponse {
                ok: true,
                verdict: None,
                message: Some("revoked".into()),
            },
            Err(e) => DaemonResponse {
                ok: false,
                verdict: None,
                message: Some(e.to_string()),
            },
        },
        DaemonRequest::Check { pid, hook } => {
            let hook = match hook.as_str() {
                "openat" | "file_open" => SyscallHook::OpenAt,
                "connect" | "socket_connect" => SyscallHook::Connect,
                other => {
                    return DaemonResponse {
                        ok: false,
                        verdict: None,
                        message: Some(format!("unknown hook: {other}")),
                    };
                }
            };
            let snapshot = eventscope_ebpf::global_bridge()
                .lock()
                .expect("bridge")
                .snapshot();
            let verdict = evaluate_hook(hook, pid, &snapshot);
            DaemonResponse {
                ok: true,
                verdict: Some(format!("{verdict:?}")),
                message: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_check_open_denies_without_map() {
        replace_global_bridge(Box::new(MockKernelBridge::new()));
        let resp = handle_request(r#"{"op":"check","pid":999,"hook":"openat"}"#);
        assert!(resp.ok);
        assert!(resp.verdict.unwrap().contains("Deny"));
    }

    #[test]
    fn daemon_publish_then_allow_open() {
        replace_global_bridge(Box::new(MockKernelBridge::new()));
        let publish = handle_request(
            r#"{"op":"publish","pid":100,"handle":42,"resource_type":1}"#,
        );
        assert!(publish.ok);
        let check = handle_request(r#"{"op":"check","pid":100,"hook":"openat"}"#);
        assert!(check.verdict.unwrap().contains("Allow"));
    }

    #[test]
    fn daemon_network_handle_denies_open() {
        replace_global_bridge(Box::new(MockKernelBridge::new()));
        let _ = handle_request(
            r#"{"op":"publish","pid":101,"handle":77,"resource_type":2}"#,
        );
        let check = handle_request(r#"{"op":"check","pid":101,"hook":"openat"}"#);
        assert!(check.verdict.unwrap().contains("Deny"));
    }
}
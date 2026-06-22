use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Result};

#[cfg(windows)]
use anyhow::Context;
use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(name = "ikd-verify", about = "IntentKernel deployment verifier")]
struct Cli {
    /// Run kernel / runtime readiness checks
    #[arg(long)]
    kernel_check: bool,

    /// Target OS profile: win11, win10, linux
    #[arg(long, default_value = "win11")]
    os: String,

    /// Emit JSON report
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct Check {
    id: &'static str,
    ok: bool,
    detail: String,
}

#[derive(Debug, Serialize)]
struct Report {
    os: String,
    platform: String,
    checks: Vec<Check>,
    passed: usize,
    failed: usize,
    ready: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if !cli.kernel_check {
        bail!("specify --kernel-check (e.g. ikd-verify --kernel-check --os win11)");
    }

    let root = repo_root();
    let mut checks = Vec::new();

    checks.push(file_exists(
        "wasm_parser",
        root.join("build/wasm/intent_parser.wasm"),
    ));
    checks.push(file_exists(
        "proto",
        root.join("core/ai-runtime/proto/intentkernel.proto"),
    ));
    checks.push(port_open("ai_runtime_grpc", "127.0.0.1:50051"));
    checks.push(port_open("intent_verifier", "127.0.0.1:7879"));

    match cli.os.as_str() {
        "win11" => {
            checks.push(win11_profile());
            checks.push(file_exists(
                "setup_script",
                root.join("setup-windows.ps1"),
            ));
            checks.push(file_exists(
                "install_script",
                root.join("install.ps1"),
            ));
            checks.push(iso_media_gate(&root));
            checks.push(binary_gate(&root, "intentkernel_cli", "intentkernel"));
        }
        "win10" => checks.push(Check {
            id: "os_profile",
            ok: true,
            detail: "win10 profile selected (relaxed kernel gate)".into(),
        }),
        "linux" => {
            checks.push(Check {
                id: "os_profile",
                ok: true,
                detail: "linux profile — WSL/host dev path".into(),
            });
            checks.push(iso_media_gate(&root));
            for (id, name) in [
                ("capd", "capd"),
                ("intentd", "intentd"),
                ("iksh", "iksh"),
                ("eventscope", "eventscope"),
            ] {
                checks.push(staged_binary_gate(&root, id, name));
            }
        }
        other => checks.push(Check {
            id: "os_profile",
            ok: false,
            detail: format!("unknown os profile: {other}"),
        }),
    }

    let passed = checks.iter().filter(|c| c.ok).count();
    let failed = checks.len() - passed;
    let ready = failed == 0;

    let report = Report {
        os: cli.os.clone(),
        platform: std::env::consts::OS.to_string(),
        checks,
        passed,
        failed,
        ready,
    };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("IntentKernel verify — os={} platform={}", report.os, report.platform);
        for c in &report.checks {
            let mark = if c.ok { "OK" } else { "FAIL" };
            println!("  [{mark}] {} — {}", c.id, c.detail);
        }
        println!(
            "Result: {}/{} passed — {}",
            report.passed,
            report.checks.len(),
            if report.ready { "READY" } else { "NOT READY" }
        );
    }

    if !report.ready {
        std::process::exit(1);
    }
    Ok(())
}

fn repo_root() -> PathBuf {
    if let Ok(dir) = std::env::var("INTENTKERNEL_DEV_ROOT") {
        return PathBuf::from(dir);
    }

    if let Ok(dir) = std::env::var("INTENTKERNEL_ROOT") {
        let root = PathBuf::from(dir);
        if looks_like_dev_tree(&root) {
            return root;
        }
    }

    for candidate in dev_root_candidates() {
        if looks_like_dev_tree(&candidate) {
            return candidate;
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn looks_like_dev_tree(root: &std::path::Path) -> bool {
    root.join("core/ai-runtime/proto/intentkernel.proto").is_file()
}

fn dev_root_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(windows)]
    {
        candidates.push(PathBuf::from(
            r"C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop",
        ));
        if let Ok(dir) = std::env::var("USERPROFILE") {
            let profile = PathBuf::from(dir);
            candidates.push(profile.join("Documents/GitHub/cautious-octo-dollop"));
            candidates.push(profile.join("CLionProjects/cautious-octo-dollop"));
            candidates.push(profile.join("cautious-octo-dollop"));
        }
        candidates.push(PathBuf::from(
            r"C:\Users\Dizzle\CLionProjects\cautious-octo-dollop",
        ));
        candidates.push(PathBuf::from(r"C:\Users\Dizzle\cautious-octo-dollop"));
        candidates.push(PathBuf::from(r"D:\cautious-octo-dollop"));
        if let Ok(dir) = std::env::var("INTENTKERNEL_ISO_ROOT") {
            candidates.push(PathBuf::from(dir));
        }
        candidates.push(PathBuf::from(r"C:\Users\Dizzle\IntentKernelISO"));
    }

    if let Ok(dir) = std::env::var("HOME") {
        let home = PathBuf::from(&dir);
        candidates.push(home.join("cautious-octo-dollop"));
        candidates.push(home.join("IntentKernelISO"));
    }

    candidates
}

fn default_iso_root() -> PathBuf {
    if let Ok(dir) = std::env::var("INTENTKERNEL_ISO_ROOT") {
        return PathBuf::from(dir);
    }

    #[cfg(windows)]
    {
        return PathBuf::from(r"C:\Users\Dizzle\IntentKernelISO");
    }

    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("IntentKernelISO"))
            .unwrap_or_else(|| PathBuf::from("IntentKernelISO"))
    }
}

fn staged_tree_root(iso: &std::path::Path) -> Option<PathBuf> {
    let nested = iso.join("IntentKernel");
    if nested.join("bin").is_dir() {
        return Some(nested);
    }
    if iso.join("bin").is_dir() {
        return Some(iso.to_path_buf());
    }
    None
}

fn iso_media_gate(_root: &PathBuf) -> Check {
    let iso = default_iso_root();

    if !iso.is_dir() {
        return Check {
            id: "iso_media",
            ok: true,
            detail: format!(
                "optional ISO root not present ({})",
                iso.display()
            ),
        };
    }

    let has_installer = iso.join("install.ps1").is_file();
    let has_zip = iso
        .read_dir()
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().is_some_and(|ext| ext == "zip"));
    let staged = staged_tree_root(&iso);
    let has_staged = staged.as_ref().is_some_and(|tree| {
        tree.join("bin/ai-runtime").is_file()
            || tree.join("bin/ai-runtime.exe").is_file()
    });
    let has_autorun = iso.join("live/intentkernel/autorun.sh").is_file();

    let ok = has_installer || has_zip || has_staged;
    Check {
        id: "iso_media",
        ok,
        detail: if ok {
            format!(
                "ISO media ready at {} (installer={has_installer}, zip={has_zip}, staged={has_staged}, autorun={has_autorun})",
                iso.display()
            )
        } else {
            #[cfg(windows)]
            let hint = "run scripts\\stage-iso.ps1";
            #[cfg(not(windows))]
            let hint = "run scripts/stage-iso.sh";
            format!(
                "ISO root exists but empty: {} — {hint}",
                iso.display()
            )
        },
    }
}

fn staged_binary_gate(root: &PathBuf, id: &'static str, name: &str) -> Check {
    let iso = default_iso_root();
    if let Some(tree) = staged_tree_root(&iso) {
        let unix = tree.join(format!("bin/{name}"));
        let win = tree.join(format!("bin/{name}.exe"));
        if unix.is_file() {
            return Check {
                id,
                ok: true,
                detail: format!("staged {}", unix.display()),
            };
        }
        if win.is_file() {
            return Check {
                id,
                ok: true,
                detail: format!("staged {}", win.display()),
            };
        }
        return Check {
            id,
            ok: false,
            detail: format!(
                "missing staged bin/{name} under {}",
                tree.display()
            ),
        };
    }

    binary_gate(root, id, name)
}

fn binary_gate(root: &PathBuf, id: &'static str, name: &str) -> Check {
    let candidates = [
        root.join(format!("target/release/{name}")),
        root.join(format!("target/release/{name}.exe")),
        root.join(format!("bin/{name}")),
        root.join(format!("bin/{name}.exe")),
    ];
    for path in candidates {
        if path.is_file() {
            return Check {
                id,
                ok: true,
                detail: format!("found {}", path.display()),
            };
        }
    }
    Check {
        id,
        ok: false,
        detail: format!("missing {name} binary under {}", root.display()),
    }
}

fn file_exists(id: &'static str, path: PathBuf) -> Check {
    let ok = path.is_file();
    Check {
        id,
        ok,
        detail: if ok {
            format!("found {}", path.display())
        } else {
            format!("missing {}", path.display())
        },
    }
}

fn port_open(id: &'static str, addr: &str) -> Check {
    let ok = TcpStream::connect_timeout(
        &addr.parse().unwrap_or_else(|_| panic!("bad addr {addr}")),
        Duration::from_millis(800),
    )
    .is_ok();
    Check {
        id,
        ok,
        detail: if ok {
            format!("{addr} reachable")
        } else {
            format!("{addr} not listening")
        },
    }
}

fn win11_profile() -> Check {
    #[cfg(windows)]
    {
        use std::process::Command;
        let out = Command::new("cmd")
            .args(["/C", "ver"])
            .output()
            .context("ver")
            .ok();
        let text = out
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();
        let build = text
            .split('.')
            .filter_map(|p| p.trim().parse::<u32>().ok())
            .nth(2)
            .unwrap_or(0);
        let ok = build >= 22000;
        return Check {
            id: "win11_build",
            ok,
            detail: if ok {
                format!("Windows build {build} (11+)")
            } else {
                format!("Windows build {build} — need >= 22000 for win11")
            },
        };
    }

    #[cfg(not(windows))]
    {
        Check {
            id: "win11_build",
            ok: true,
            detail: "skipped on non-Windows host (run on win11 for full gate)".into(),
        }
    }
}
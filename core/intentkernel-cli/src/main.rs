use std::fs;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "intentkernel", about = "Intent Kernel AI OS CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Intent broker daemon (orchestrates AI OS health)
    Intentd {
        #[command(subcommand)]
        action: IntentdCommands,
    },
    /// Write first-run configuration
    Configure {
        #[arg(long)]
        runtime_addr: Option<String>,
        #[arg(long)]
        verifier_addr: Option<String>,
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// Show configured paths and service reachability
    Status {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// Print installed version
    Version {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct IntentKernelConfig {
    version: String,
    runtime_addr: String,
    verifier_addr: String,
    dashboard: String,
}

impl Default for IntentKernelConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            runtime_addr: "127.0.0.1:50051".into(),
            verifier_addr: "127.0.0.1:7879".into(),
            dashboard: "share/dashboard/index.html".into(),
        }
    }
}

#[derive(Subcommand, Debug)]
enum IntentdCommands {
    /// Verify services and run HTTP health API on 127.0.0.1:8780
    Start {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// Print unified AI OS status (JSON)
    Status {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// Probe ai-runtime and intent-verifier (exit 1 if not ready)
    Health {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Intentd { action } => match action {
            IntentdCommands::Start { install_root } => intentd::run_start_blocking(install_root),
            IntentdCommands::Status { install_root } => {
                let status = intentd::collect_status(install_root)?;
                println!("{}", serde_json::to_string_pretty(&status)?);
                Ok(())
            }
            IntentdCommands::Health { install_root } => {
                let status = intentd::collect_status(install_root)?;
                println!("{}", serde_json::to_string_pretty(&status)?);
                if !status.ready {
                    eprintln!(
                        "not ready — start missing services, then run `intentd start` (broker http://127.0.0.1:8780/health)"
                    );
                    std::process::exit(1);
                }
                Ok(())
            }
        },
        Commands::Configure {
            runtime_addr,
            verifier_addr,
            install_root,
        } => configure(runtime_addr, verifier_addr, install_root),
        Commands::Status { install_root } => status(install_root),
        Commands::Version { install_root } => show_version(install_root),
    }
}

fn configure(
    runtime_addr: Option<String>,
    verifier_addr: Option<String>,
    install_root: Option<PathBuf>,
) -> Result<()> {
    let root = resolve_root(install_root);
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir).with_context(|| format!("create {}", config_dir.display()))?;

    let mut config = load_config(&root).unwrap_or_default();
    if let Some(addr) = runtime_addr {
        config.runtime_addr = addr;
    }
    if let Some(addr) = verifier_addr {
        config.verifier_addr = addr;
    }

    let path = config_dir.join("intentkernel.toml");
    let body = toml::to_string_pretty(&config)?;
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;

    println!("Configured Intent Kernel AI OS");
    println!("  root:     {}", root.display());
    println!("  config:   {}", path.display());
    println!("  runtime:  {}", config.runtime_addr);
    println!("  verifier: {}", config.verifier_addr);
    Ok(())
}

fn status(install_root: Option<PathBuf>) -> Result<()> {
    let root = resolve_root(install_root);
    let config = load_config(&root).unwrap_or_default();
    let version = read_version(&root);

    println!("Intent Kernel status");
    println!("  root:     {}", root.display());
    println!("  version:  {version}");
    println!("  runtime:  {} ({})", config.runtime_addr, reach(&config.runtime_addr));
    println!(
        "  verifier: {} ({})",
        config.verifier_addr,
        reach(&config.verifier_addr)
    );
    Ok(())
}

fn show_version(install_root: Option<PathBuf>) -> Result<()> {
    let root = resolve_root(install_root);
    println!("{}", read_version(&root));
    Ok(())
}

fn resolve_root(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }
    if let Ok(dir) = std::env::var("INTENTKERNEL_ROOT") {
        return PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("INTENTKERNEL_DEV_ROOT") {
        return PathBuf::from(dir);
    }
    #[cfg(windows)]
    {
        for candidate in [
            r"C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop",
            r"D:\intentkernel",
            r"C:\Users\Dizzle\CLionProjects\cautious-octo-dollop",
        ] {
            let path = PathBuf::from(candidate);
            if path.is_dir() {
                return path;
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn load_config(root: &Path) -> Result<IntentKernelConfig> {
    let path = root.join("config/intentkernel.toml");
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(toml::from_str(&text)?)
}

fn read_version(root: &Path) -> String {
    fs::read_to_string(root.join("VERSION"))
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "0.1.0".into())
}

fn reach(addr: &str) -> &'static str {
    let Ok(sock_addr) = addr.parse() else {
        return "invalid";
    };
    if TcpStream::connect_timeout(&sock_addr, Duration::from_millis(800)).is_ok() {
        "up"
    } else {
        "down"
    }
}
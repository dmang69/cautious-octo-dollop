mod capability_service;
mod lookup;
mod scheduler;
mod server;
mod telemetry;

pub mod intentkernel {
    pub mod v1 {
        tonic::include_proto!("intentkernel.v1");
    }
}

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

const SERVICE_NAME: &str = "IntentKernelRuntime";
const SERVICE_DISPLAY: &str = "Intent Kernel AI Runtime";

#[derive(Parser, Debug)]
#[command(name = "ai-runtime", about = "Intent Kernel AI runtime (gRPC)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Open the Intent Kernel monitoring dashboard
    #[arg(long)]
    dashboard: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Register the Windows background service
    Install,
    /// Remove the Windows background service
    Uninstall,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return match command {
            Commands::Install => install_service(),
            Commands::Uninstall => uninstall_service(),
        };
    }

    if cli.dashboard {
        return open_dashboard();
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    intentkernel_platform::run_as_service(SERVICE_NAME, run_server)
}

fn run_server() -> Result<()> {
    let rt = tokio::runtime::Runtime::new().context("create tokio runtime")?;
    rt.block_on(server::run())
}

fn install_service() -> Result<()> {
    let exe = std::env::current_exe().context("resolve current executable")?;
    intentkernel_platform::install_service(SERVICE_NAME, SERVICE_DISPLAY, &exe)?;
    println!("Installed Windows service: {SERVICE_NAME}");
    Ok(())
}

fn uninstall_service() -> Result<()> {
    intentkernel_platform::uninstall_service(SERVICE_NAME)?;
    println!("Removed Windows service: {SERVICE_NAME}");
    Ok(())
}

fn open_dashboard() -> Result<()> {
    let root = install_root();
    let dashboard_exe = root.join("bin").join("intentkernel-dashboard.exe");
    let html = root.join("share").join("dashboard").join("index.html");

    if dashboard_exe.is_file() {
        Command::new(dashboard_exe)
            .spawn()
            .context("launch intentkernel-dashboard.exe")?;
        return Ok(());
    }

    if html.is_file() {
        open_path(&html)?;
        return Ok(());
    }

    open_path("https://docs.intentkernel.ai")?;
    println!("Dashboard assets not found under {}", root.display());
    println!("Opened documentation — install share/dashboard or build the Tauri shell.");
    Ok(())
}

fn install_root() -> PathBuf {
    if let Ok(dir) = std::env::var("INTENTKERNEL_ROOT") {
        return PathBuf::from(dir);
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|b| b.parent().unwrap_or(b).to_path_buf()))
        .unwrap_or_else(default_install_root)
}

fn default_install_root() -> PathBuf {
    PathBuf::from(r"D:\intentkernel")
}

fn open_path(path: impl AsRef<std::ffi::OsStr>) -> Result<()> {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.as_ref().to_string_lossy()])
            .spawn()
            .context("open path via start")?;
    }

    #[cfg(not(windows))]
    {
        let path = path.as_ref();
        if path.to_string_lossy().starts_with("http") {
            if Command::new("xdg-open").arg(path).spawn().is_err() {
                println!("Open in browser: {}", path.to_string_lossy());
            }
        } else {
            println!("Dashboard: {}", path.to_string_lossy());
        }
    }

    Ok(())
}
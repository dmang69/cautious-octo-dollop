use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};
use intentd::{collect_status, run_start_blocking, INTENTD_HTTP_ADDR};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "intentd", about = "IntentKernel Intent Broker orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Verify child services and run the HTTP health broker
    Start {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// Print unified AI OS status as JSON
    Status {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// Probe core services; exit non-zero when not ready
    Health {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("intentd error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start { install_root } => run_start_blocking(install_root),
        Commands::Status { install_root } => {
            let status = collect_status(install_root)?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        Commands::Health { install_root } => {
            let status = collect_status(install_root)?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            if !status.ready {
                eprintln!(
                    "intentd health: not ready (need ai-runtime + intent-verifier; broker at {INTENTD_HTTP_ADDR})"
                );
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
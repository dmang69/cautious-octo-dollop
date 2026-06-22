//! Standalone BPF loader binary (requires `--features bpf`).

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use eventscope_ebpf::loader::{default_bpf_object_path, probe_loader_status, try_load_bpf, LoaderStatus};

#[derive(Parser, Debug)]
#[command(name = "eventscope-bpf-loader", about = "Load EventScope eBPF LSM programs")]
struct Args {
    /// Path to compiled `eventscope.bpf.o`
    #[arg(long)]
    object: Option<PathBuf>,

    /// Print loader status and exit without attaching
    #[arg(long)]
    probe: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.probe {
        match probe_loader_status() {
            LoaderStatus::Ready => println!("eventscope-bpf: ready"),
            LoaderStatus::ObjectMissing(p) => {
                println!("eventscope-bpf: object missing at {}", p.display());
            }
            LoaderStatus::FeatureDisabled => {
                println!("eventscope-bpf: built without `bpf` feature");
            }
            LoaderStatus::LoadFailed(msg) => {
                println!("eventscope-bpf: load failed: {msg}");
            }
        }
        return Ok(());
    }

    let object = args.object.unwrap_or_else(default_bpf_object_path);
    std::env::set_var("EVENTSCOPE_BPF_OBJ", object.display().to_string());
    try_load_bpf()?;
    println!("eventscope-bpf: LSM programs attached, handle_map live");
    Ok(())
}
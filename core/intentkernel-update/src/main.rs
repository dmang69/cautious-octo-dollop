use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

#[cfg(windows)]
use anyhow::Context;
use clap::Parser;
#[cfg(windows)]
use serde::Deserialize;
use tracing_subscriber::EnvFilter;

const DEFAULT_RELEASES_URL: &str = "https://releases.intentkernel.ai";

#[derive(Parser, Debug)]
#[command(name = "intentkernel-update", about = "Intent Kernel automatic updater")]
struct Cli {
    /// Install root (defaults to INTENTKERNEL_ROOT or Program Files)
    #[arg(long)]
    root: Option<PathBuf>,

    /// Releases base URL
    #[arg(long, default_value = DEFAULT_RELEASES_URL)]
    releases_url: String,

    /// Check for updates only; do not download or install
    #[arg(long)]
    check: bool,
}

#[cfg(windows)]
#[derive(Debug, Deserialize)]
struct ReleaseManifest {
    version: String,
    #[serde(default)]
    package: Option<String>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let cli = Cli::parse();
    let root = install_root(cli.root.or_else(|| {
        std::env::var("INTENTKERNEL_ROOT")
            .ok()
            .map(PathBuf::from)
    }));
    let current = read_installed_version(&root)?;
    tracing::info!(installed = %current, root = %root.display(), "checking for updates");

    #[cfg(windows)]
    {
        return run_windows_update(&cli, &root, &current);
    }

    #[cfg(not(windows))]
    {
        let _ = current;
        bail!("intentkernel-update runs on installed Windows deployments")
    }
}

#[cfg(windows)]
fn run_windows_update(cli: &Cli, root: &Path, current: &str) -> Result<()> {
    let manifest = fetch_manifest(&cli.releases_url)?;
    if manifest.version == current {
        tracing::info!(version = %current, "already up to date");
        return Ok(());
    }

    tracing::info!(
        installed = %current,
        latest = %manifest.version,
        "update available"
    );

    if cli.check {
        println!("Update available: {} -> {}", current, manifest.version);
        return Ok(());
    }

    let package = manifest
        .package
        .unwrap_or_else(|| format!("intentkernel-{}-windows-x86_64.zip", manifest.version));
    let url = format!("{}/{}", cli.releases_url.trim_end_matches('/'), package);
    let temp = std::env::temp_dir().join(&package);

    tracing::info!(%url, "downloading package");
    let bytes = reqwest::blocking::get(&url)
        .with_context(|| format!("download {url}"))?
        .bytes()
        .context("read release package")?;
    fs::write(&temp, &bytes).with_context(|| format!("write {}", temp.display()))?;

    let staging = root.join("update-staging");
    if staging.exists() {
        fs::remove_dir_all(&staging).ok();
    }
    fs::create_dir_all(&staging)?;

    let status = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                temp.display(),
                staging.display()
            ),
        ])
        .status()
        .context("expand release archive")?;

    if !status.success() {
        bail!("failed to extract update package");
    }

    copy_tree(&staging, root)?;
    write_version(root, &manifest.version)?;
    fs::remove_dir_all(&staging).ok();
    fs::remove_file(&temp).ok();

    tracing::info!(version = %manifest.version, "update installed");
    println!("Updated Intent Kernel AI OS to {}", manifest.version);
    Ok(())
}

#[cfg(windows)]
fn fetch_manifest(base_url: &str) -> Result<ReleaseManifest> {
    let url = format!("{}/manifest.json", base_url.trim_end_matches('/'));
    let text = reqwest::blocking::get(&url)
        .with_context(|| format!("fetch {url}"))?
        .text()
        .context("read manifest")?;
    serde_json::from_str(&text).context("parse manifest.json")
}

fn install_root(explicit: Option<PathBuf>) -> PathBuf {
    explicit.unwrap_or_else(|| {
        std::env::var("INTENTKERNEL_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(r"D:\intentkernel"))
    })
}

fn read_installed_version(root: &Path) -> Result<String> {
    let version_file = root.join("VERSION");
    if version_file.is_file() {
        return Ok(fs::read_to_string(version_file)?.trim().to_string());
    }
    Ok("0.0.0".into())
}

#[cfg(windows)]
fn write_version(root: &Path, version: &str) -> Result<()> {
    fs::write(root.join("VERSION"), format!("{version}\n"))?;
    Ok(())
}

#[cfg(windows)]
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        bail!("staging path missing: {}", from.display());
    }
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        let dest = to.join(entry.file_name());
        if path.is_dir() {
            fs::create_dir_all(&dest)?;
            copy_tree(&path, &dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}
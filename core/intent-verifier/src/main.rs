mod server;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};

const TOKEN_MAGIC: &[u8] = b"IKTK";
const TOKEN_TRAILER: &[u8] = b"KTIK";
const TOKEN_VERSION: u16 = 1;

const SERVICE_NAME: &str = "IntentKernelVerifier";
const SERVICE_DISPLAY: &str = "Intent Kernel Verifier";

#[derive(Parser, Debug)]
#[command(name = "intent-verifier", about = "Intent Kernel IKTK verifier")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to an IKTK token file to verify once and exit
    #[arg(value_name = "TOKEN_FILE")]
    token_file: Option<PathBuf>,
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

    if let Some(path) = cli.token_file {
        let bytes = std::fs::read(&path)?;
        let summary = verify_bytes(&bytes)?;
        println!("OK: {summary}");
        return Ok(());
    }

    fmt().with_env_filter(EnvFilter::new("info")).init();

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

pub(crate) fn verify_bytes(bytes: &[u8]) -> Result<String> {
    if bytes.len() >= 14 && &bytes[0..4] == TOKEN_MAGIC {
        return verify_legacy_iktk(bytes);
    }
    verify_rfc_intent(bytes)
}

fn verify_legacy_iktk(bytes: &[u8]) -> Result<String> {
    let version = u16::from_be_bytes([bytes[4], bytes[5]]);
    if version != TOKEN_VERSION {
        anyhow::bail!("unsupported version {version}");
    }
    let payload_len = u32::from_be_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
    let end = 10 + payload_len + 4;
    if bytes.len() < end {
        anyhow::bail!("truncated payload");
    }
    if &bytes[10 + payload_len..end] != TOKEN_TRAILER {
        anyhow::bail!("bad trailer");
    }
    let payload = std::str::from_utf8(&bytes[10..10 + payload_len])?;
    let value: serde_json::Value = serde_json::from_str(payload)?;
    let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
    let subject = value
        .get("subject")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    Ok(format!("legacy id={id} subject={subject}"))
}

fn verify_rfc_intent(bytes: &[u8]) -> Result<String> {
    use intentkernel_crypto::sign::{PublicKey, TokenValidator};
    use intentkernel_crypto::token::Algorithm;
    use intentkernel_util::paths::resolve_root;

    let root = resolve_root(None);
    let key_path = root.join("config/broker.key.json");
    let keyfile: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&key_path)
        .with_context(|| format!("read broker key {}", key_path.display()))?)?;
    let alg = match keyfile["algorithm"].as_str().unwrap_or("ed25519") {
        "ed25519" => Algorithm::Ed25519,
        "ml-dsa-87" | "mldsa87" => Algorithm::MlDsa87,
        other => anyhow::bail!("unknown algorithm {other}"),
    };
    let pk = hex::decode(keyfile["public_key"].as_str().context("public_key")?)?;
    let validator = TokenValidator::new(PublicKey {
        algorithm: alg,
        bytes: pk,
    });
    let token = validator
        .validate_bytes(bytes)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(format!(
        "rfc-intent jti={} sub={} uses={} exp={}",
        hex::encode(&token.payload.jti),
        hex::encode(&token.payload.sub),
        token.payload.uses,
        token.payload.exp
    ))
}
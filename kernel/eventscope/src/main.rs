//! Thin CLI for testing eventscope interception (`eventscope check --handle 0x... --path /foo`).

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use eventscope::{EventScope, FileAccess, InterceptVerdict};
use intentkernel_crypto::cbor::decode_wire_token;
use intentkernel_crypto::sign::{PublicKey, TokenValidator};
use intentkernel_crypto::token::Algorithm;
use intentkernel_util::paths::resolve_root;

#[derive(Parser, Debug)]
#[command(name = "eventscope", about = "IntentKernel syscall interception tester")]
struct Cli {
    #[arg(long)]
    install_root: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Register a signed token and print its kernel handle.
    Register {
        /// Path to CBOR-encoded wire token.
        token: PathBuf,
    },
    /// Check whether a handle authorizes a resource request.
    Check {
        /// Kernel handle (hex, e.g. 0x00010001A3B2).
        #[arg(long)]
        handle: Option<String>,

        /// Register token inline before check (alternative to --handle).
        #[arg(long)]
        token: Option<PathBuf>,

        /// File path to test.
        #[arg(long)]
        path: Option<String>,

        /// File access: read, write, rw.
        #[arg(long, default_value = "read")]
        access: String,

        /// Network destination host (optionally host:port).
        #[arg(long)]
        network: Option<String>,

        /// Raw action string for raw-scope tokens.
        #[arg(long)]
        action: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = resolve_root(cli.install_root);
    let validator = load_validator(&root)?;
    let mut scope = EventScope::new(validator);

    match cli.command {
        Command::Register { token } => {
            let handle = register_token(&mut scope, &token)?;
            println!("handle=0x{:016X}", handle.raw);
        }
        Command::Check {
            handle,
            token,
            path,
            access,
            network,
            action,
        } => {
            let handle_raw = if let Some(token_path) = token {
                register_token(&mut scope, &token_path)?.raw
            } else if let Some(h) = handle {
                parse_handle(&h)?
            } else {
                anyhow::bail!("check requires --handle or --token");
            };

            let verdict = if let Some(path) = path {
                let mode = parse_access(&access)?;
                scope.intercept_file(&path, mode, handle_raw)
            } else if let Some(net) = network {
                let (host, port) = parse_network(&net)?;
                let ip = eventscope::parse_ip_bytes(&host)
                    .with_context(|| format!("invalid network address {host}"))?;
                scope.intercept_network(&ip, port, handle_raw)
            } else if let Some(action) = action {
                scope.intercept_raw(action.as_bytes(), handle_raw)
            } else {
                anyhow::bail!("check requires --path, --network, or --action");
            };

            print_verdict(verdict);
        }
    }

    Ok(())
}

fn register_token(
    scope: &mut EventScope,
    path: &PathBuf,
) -> Result<intentkernel_core::handle::KernelHandle> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let token = decode_wire_token(&bytes)?;
    scope
        .register_token(&token)
        .map_err(|e| anyhow::anyhow!("register failed: {e}"))
}

fn parse_handle(s: &str) -> Result<u64> {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).context("invalid handle hex")
}

fn parse_access(s: &str) -> Result<FileAccess> {
    match s.to_ascii_lowercase().as_str() {
        "read" | "r" => Ok(FileAccess::Read),
        "write" | "w" => Ok(FileAccess::Write),
        "rw" | "readwrite" => Ok(FileAccess::ReadWrite),
        other => anyhow::bail!("unknown access mode {other}"),
    }
}

fn parse_network(s: &str) -> Result<(String, u16)> {
    if let Some((host, port)) = s.rsplit_once(':') {
        if !host.is_empty() && !host.contains(':') {
            return Ok((host.to_string(), port.parse()?));
        }
    }
    Ok((s.to_string(), 443))
}

fn print_verdict(verdict: InterceptVerdict) {
    match verdict {
        InterceptVerdict::Allow { resource_type } => {
            println!("ALLOW resource_type={resource_type}");
            std::process::exit(0);
        }
        InterceptVerdict::Deny(err) => {
            println!("DENY {err}");
            std::process::exit(1);
        }
    }
}

fn load_validator(root: &PathBuf) -> Result<TokenValidator> {
    let path = root.join("config/broker.key.json");
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read broker key {}", path.display()))?;
    let keyfile: serde_json::Value = serde_json::from_str(&text)?;
    let alg = match keyfile["algorithm"].as_str().unwrap_or("ed25519") {
        "ed25519" => Algorithm::Ed25519,
        "ml-dsa-87" | "mldsa87" => Algorithm::MlDsa87,
        other => anyhow::bail!("unknown algorithm {other}"),
    };
    let pk = hex::decode(keyfile["public_key"].as_str().context("public_key")?)?;
    Ok(TokenValidator::new(PublicKey {
        algorithm: alg,
        bytes: pk,
    }))
}
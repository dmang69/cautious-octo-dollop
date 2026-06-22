use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use intentkernel_core::gate::{SyscallRequest, SyscallResult};
use intentkernel_core::IntentKernel;
use intentkernel_crypto::cbor::decode_wire_token;
use intentkernel_crypto::sign::{PublicKey, TokenValidator};
use intentkernel_crypto::token::Algorithm;
use intentkernel_util::config::{load_config, read_version};
use intentkernel_util::paths::resolve_root;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(name = "iksh", about = "IntentKernel interactive shell")]
struct Cli {
    #[arg(long)]
    install_root: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct ShellStatus {
    version: String,
    root: String,
    runtime_addr: String,
    verifier_addr: String,
    active_capabilities: usize,
    registered_handles: usize,
}

struct ShellState {
    root: PathBuf,
    kernel: IntentKernel,
    validator: Option<TokenValidator>,
    handles: Vec<u64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = resolve_root(cli.install_root);
    let mut shell = ShellState {
        root: root.clone(),
        kernel: IntentKernel::new(),
        validator: load_validator(&root).ok(),
        handles: Vec::new(),
    };

    println!("IntentKernel Shell (iksh) — zero ambient authority");
    println!("Type `help` for commands. root={}", root.display());

    let stdin = io::stdin();
    loop {
        print!("iksh> ");
        io::stdout().flush()?;
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match dispatch(&mut shell, line) {
            Ok(Some(msg)) => println!("{msg}"),
            Ok(None) => {}
            Err(e) => eprintln!("error: {e:#}"),
        }
    }
    Ok(())
}

fn dispatch(shell: &mut ShellState, line: &str) -> Result<Option<String>> {
    let mut parts = line.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    match cmd {
        "help" => {
            print_help();
            Ok(None)
        }
        "exit" | "quit" => std::process::exit(0),
        "status" => Ok(Some(status_line(shell)?)),
        "stats" => {
            let s = shell.kernel.stats();
            Ok(Some(format!(
                "capabilities={} handles={}",
                s.active_capabilities, s.registered_handles
            )))
        }
        "register" => {
            let path = parts
                .next()
                .context("usage: register <token-file> [resource-type]")?;
            let resource_type = parts.next().unwrap_or("1").parse::<u32>()?;
            let bytes = fs::read(path).with_context(|| format!("read {path}"))?;
            let token = decode_wire_token(&bytes)?;
            let validator = shell
                .validator
                .as_ref()
                .context("no broker pubkey — run capd init")?;
            let handle = shell
                .kernel
                .gate()
                .register_token(&token, validator, resource_type)?;
            shell.handles.push(handle.raw);
            Ok(Some(format!(
                "registered handle=0x{:016X} type={resource_type}",
                handle.raw
            )))
        }
        "invoke" => {
            let handle_str = parts.next().context("usage: invoke <handle-hex> [action]")?;
            let action = parts.next().unwrap_or("0").parse::<u32>()?;
            let handle = u64::from_str_radix(handle_str.trim_start_matches("0x"), 16)
                .context("invalid handle hex")?;
            let seq = shell.kernel.sequences.get(&handle).copied().unwrap_or(0) + 1;
            match shell.kernel.gate().invoke(SyscallRequest {
                handle,
                sequence: seq,
                action,
            }) {
                SyscallResult::Allowed { resource_type } => Ok(Some(format!(
                    "ALLOWED resource_type={resource_type}"
                ))),
                SyscallResult::Denied(e) => Ok(Some(format!("DENIED: {e}"))),
            }
        }
        "revoke" => {
            let handle_str = parts.next().context("usage: revoke <handle-hex>")?;
            let handle = u64::from_str_radix(handle_str.trim_start_matches("0x"), 16)?;
            shell.kernel.gate().revoke_handle(handle)?;
            shell.handles.retain(|h| *h != handle);
            Ok(Some(format!("revoked handle=0x{handle:016X}")))
        }
        "handles" => {
            if shell.handles.is_empty() {
                Ok(Some("no active handles".into()))
            } else {
                let list: Vec<String> = shell
                    .handles
                    .iter()
                    .map(|h| format!("0x{h:016X}"))
                    .collect();
                Ok(Some(list.join("\n")))
            }
        }
        "verify" => {
            let path = parts.next().context("usage: verify <token-file>")?;
            let bytes = fs::read(path)?;
            let validator = shell
                .validator
                .as_ref()
                .context("no broker pubkey")?;
            let token = validator.validate_bytes(&bytes)?;
            Ok(Some(format!(
                "OK jti={} uses={} exp={}",
                hex::encode(&token.payload.jti),
                token.payload.uses,
                token.payload.exp
            )))
        }
        other => Ok(Some(format!("unknown command: {other} (try help)"))),
    }
}

fn status_line(shell: &ShellState) -> Result<String> {
    let config = load_config(&shell.root).unwrap_or_default();
    let stats = shell.kernel.stats();
    let status = ShellStatus {
        version: read_version(&shell.root),
        root: shell.root.display().to_string(),
        runtime_addr: config.runtime_addr,
        verifier_addr: config.verifier_addr,
        active_capabilities: stats.active_capabilities,
        registered_handles: stats.registered_handles,
    };
    Ok(serde_json::to_string_pretty(&status)?)
}

fn load_validator(root: &PathBuf) -> Result<TokenValidator> {
    let path = root.join("config/broker.key.json");
    let text = fs::read_to_string(&path)?;
    let keyfile: serde_json::Value = serde_json::from_str(&text)?;
    let alg = match keyfile["algorithm"].as_str().unwrap_or("ed25519") {
        "ed25519" => Algorithm::Ed25519,
        "ml-dsa-87" | "mldsa87" => Algorithm::MlDsa87,
        other => anyhow::bail!("unknown algorithm {other}"),
    };
    let pk = hex::decode(keyfile["public_key"].as_str().context("public_key")?)?;
    Ok(TokenValidator::new(PublicKey { algorithm: alg, bytes: pk }))
}

fn print_help() {
    println!(
        r#"Commands:
  help                         Show this help
  status                       JSON status (version, services, kernel stats)
  stats                        Kernel capability/handle counts
  register <file> [type]       Register signed token, get kernel handle
  invoke <handle> [action]     Invoke syscall with handle
  revoke <handle>              Revoke handle and capability
  handles                      List active handles this session
  verify <file>                Verify token signature and TTL
  exit                         Quit"#
    );
}
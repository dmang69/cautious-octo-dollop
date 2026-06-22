use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use intentkernel_crypto::sign::{KeyPair, Signer, TokenIssuer, token_to_bytes};
use intentkernel_crypto::token::{
    Algorithm, FileScope, NetworkScope, ResourceScope, TrustAnchor,
};
use intentkernel_util::paths::resolve_root;
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "capd", about = "IntentKernel capability broker daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a new broker keypair
    Init {
        #[arg(long, default_value = "ed25519")]
        algorithm: String,
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// Issue a capability token
    Issue {
        #[arg(long)]
        subject: String,
        #[arg(long, default_value = "open_file")]
        action: String,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        host: Option<String>,
        #[arg(long, default_value_t = 443)]
        port: u16,
        #[arg(long, default_value = "ui-event")]
        anchor: String,
        #[arg(long, default_value_t = 60_000)]
        ttl_ms: u64,
        #[arg(long, default_value_t = 1)]
        uses: u32,
        #[arg(long)]
        install_root: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Show broker public key
    Pubkey {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct BrokerKeyFile {
    algorithm: String,
    public_key: String,
    secret_key: String,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Init { algorithm, install_root } => init_broker(algorithm, install_root),
        Commands::Issue {
            subject,
            action,
            path,
            host,
            port,
            anchor,
            ttl_ms,
            uses,
            install_root,
            out,
        } => issue_token(
            subject,
            action,
            path,
            host,
            port,
            parse_anchor(&anchor)?,
            ttl_ms,
            uses,
            install_root,
            out,
        ),
        Commands::Pubkey { install_root } => show_pubkey(install_root),
    }
}

fn broker_key_path(root: &PathBuf) -> PathBuf {
    root.join("config/broker.key.json")
}

fn init_broker(algorithm: String, install_root: Option<PathBuf>) -> Result<()> {
    let root = resolve_root(install_root);
    let alg = parse_algorithm(&algorithm)?;
    let kp = KeyPair::generate(alg)?;
    let keyfile = BrokerKeyFile {
        algorithm: algorithm.clone(),
        public_key: hex::encode(kp.public_key()),
        secret_key: hex::encode(kp.secret_key_bytes()),
    };
    let path = broker_key_path(&root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(&keyfile)?)
        .with_context(|| format!("write {}", path.display()))?;
    println!("Broker initialized");
    println!("  root:   {}", root.display());
    println!("  key:    {}", path.display());
    println!("  alg:    {algorithm}");
    println!("  pubkey: {}", keyfile.public_key);
    Ok(())
}

fn load_broker(root: &PathBuf) -> Result<KeyPair> {
    let path = broker_key_path(root);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read {} — run `capd init` first", path.display()))?;
    let keyfile: BrokerKeyFile = serde_json::from_str(&text)?;
    let alg = parse_algorithm(&keyfile.algorithm)?;
    let secret = hex::decode(&keyfile.secret_key)?;
    KeyPair::from_secret(alg, secret)
}

fn issue_token(
    subject: String,
    action: String,
    path: Option<String>,
    host: Option<String>,
    port: u16,
    anchor: TrustAnchor,
    ttl_ms: u64,
    uses: u32,
    install_root: Option<PathBuf>,
    out: Option<PathBuf>,
) -> Result<()> {
    let root = resolve_root(install_root);
    let kp = load_broker(&root)?;
    let issuer = TokenIssuer::new(kp);

    let scope = if let Some(p) = path {
        ResourceScope::File(FileScope {
            path: p,
            access: 3,
            inode: None,
        })
    } else if let Some(h) = host {
        let ip = if h.parse::<std::net::IpAddr>().is_ok() {
            h.parse::<std::net::IpAddr>()?.to_string()
        } else {
            h
        };
        ResourceScope::Network(NetworkScope {
            proto: 1,
            dst_ip: ip.as_bytes().to_vec(),
            dst_port: port,
            bytes: 1_048_576,
        })
    } else {
        anyhow::bail!("specify --path or --host for scope");
    };

    let token = issuer.issue_capability(
        subject.as_bytes(),
        action.as_bytes(),
        scope,
        anchor,
        ttl_ms,
        uses,
    )?;
    let bytes = token_to_bytes(&token)?;

    if let Some(out_path) = out {
        fs::write(&out_path, &bytes).with_context(|| format!("write {}", out_path.display()))?;
        println!("Token written to {}", out_path.display());
    } else {
        println!("{}", hex::encode(&bytes));
    }
    Ok(())
}

fn show_pubkey(install_root: Option<PathBuf>) -> Result<()> {
    let root = resolve_root(install_root);
    let path = broker_key_path(&root);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let keyfile: BrokerKeyFile = serde_json::from_str(&text)?;
    println!("algorithm: {}", keyfile.algorithm);
    println!("public_key: {}", keyfile.public_key);
    Ok(())
}

fn parse_anchor(s: &str) -> Result<TrustAnchor> {
    match s.to_lowercase().as_str() {
        "none" => Ok(TrustAnchor::None),
        "ui-event" | "ui_event" => Ok(TrustAnchor::UiEvent),
        "biometric" => Ok(TrustAnchor::Biometric),
        "hardware" => Ok(TrustAnchor::Hardware),
        "federated" => Ok(TrustAnchor::Federated),
        other => anyhow::bail!("unknown anchor: {other}"),
    }
}

fn parse_algorithm(s: &str) -> Result<Algorithm> {
    match s.to_lowercase().as_str() {
        "ed25519" => Ok(Algorithm::Ed25519),
        "ml-dsa-87" | "mldsa87" => Ok(Algorithm::MlDsa87),
        "ml-dsa-65" | "mldsa65" => Ok(Algorithm::MlDsa65),
        other => anyhow::bail!("unknown algorithm: {other}"),
    }
}
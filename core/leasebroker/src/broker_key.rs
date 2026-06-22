use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use intentkernel_crypto::sign::KeyPair;
use intentkernel_crypto::token::Algorithm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BrokerKeyFile {
    pub algorithm: String,
    pub public_key: String,
    pub secret_key: String,
}

pub fn broker_key_path(root: &Path) -> PathBuf {
    root.join("config/broker.key.json")
}

pub fn load_broker_key(root: &Path) -> Result<KeyPair> {
    let path = broker_key_path(root);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read {} — run `capd init` first", path.display()))?;
    let keyfile: BrokerKeyFile = serde_json::from_str(&text)?;
    let alg = parse_algorithm(&keyfile.algorithm)?;
    let secret = hex::decode(&keyfile.secret_key)?;
    KeyPair::from_secret(alg, secret)
}

fn parse_algorithm(s: &str) -> Result<Algorithm> {
    match s.to_lowercase().as_str() {
        "ed25519" => Ok(Algorithm::Ed25519),
        "ml-dsa-87" | "mldsa87" => Ok(Algorithm::MlDsa87),
        "ml-dsa-65" | "mldsa65" => Ok(Algorithm::MlDsa65),
        other => anyhow::bail!("unknown algorithm: {other}"),
    }
}
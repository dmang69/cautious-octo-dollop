use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentKernelConfig {
    pub version: String,
    pub runtime_addr: String,
    pub verifier_addr: String,
    pub dashboard: String,
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

pub fn load_config(root: &Path) -> Result<IntentKernelConfig> {
    let path = root.join("config/intentkernel.toml");
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(toml::from_str(&text)?)
}

pub fn save_config(root: &Path, config: &IntentKernelConfig) -> Result<()> {
    let dir = root.join("config");
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let path = dir.join("intentkernel.toml");
    let body = toml::to_string_pretty(config)?;
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn read_version(root: &Path) -> String {
    fs::read_to_string(root.join("VERSION"))
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "0.1.0".into())
}
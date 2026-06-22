use std::fs;
use std::net::TcpStream;
use std::time::Duration;

use intentkernel_util::config::{load_config, read_version};
use intentkernel_util::paths::resolve_root;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BrokerKeyStatusDto {
    pub present: bool,
    pub algorithm: Option<String>,
    pub public_key_preview: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceHealthDto {
    pub id: String,
    pub addr: String,
    pub reachable: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct OsHealthDto {
    pub version: String,
    pub root: String,
    pub broker: BrokerKeyStatusDto,
    pub runtime: ServiceHealthDto,
    pub verifier: ServiceHealthDto,
    pub healthy: bool,
}

#[tauri::command]
pub fn os_health() -> Result<OsHealthDto, String> {
    let root = resolve_root(None);
    let config = load_config(&root).unwrap_or_default();

    let broker = broker_key_status(&root);
    let runtime = check_port("ai_runtime", &config.runtime_addr);
    let verifier = check_port("intent_verifier", &config.verifier_addr);

    let healthy = broker.present && runtime.reachable && verifier.reachable;

    Ok(OsHealthDto {
        version: read_version(&root),
        root: root.display().to_string(),
        broker,
        runtime,
        verifier,
        healthy,
    })
}

fn broker_key_status(root: &std::path::Path) -> BrokerKeyStatusDto {
    let path = root.join("config/broker.key.json");
    if !path.is_file() {
        return BrokerKeyStatusDto {
            present: false,
            algorithm: None,
            public_key_preview: None,
        };
    }

    match fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(keyfile) => {
                let algorithm = keyfile["algorithm"]
                    .as_str()
                    .map(|s| s.to_string());
                let preview = keyfile["public_key"]
                    .as_str()
                    .map(|pk| truncate_hex(pk, 16));
                BrokerKeyStatusDto {
                    present: keyfile["public_key"].as_str().is_some(),
                    algorithm,
                    public_key_preview: preview,
                }
            }
            Err(_) => BrokerKeyStatusDto {
                present: false,
                algorithm: None,
                public_key_preview: None,
            },
        },
        Err(_) => BrokerKeyStatusDto {
            present: false,
            algorithm: None,
            public_key_preview: None,
        },
    }
}

fn check_port(id: &str, addr: &str) -> ServiceHealthDto {
    let reachable = TcpStream::connect_timeout(
        &addr
            .parse()
            .unwrap_or_else(|_| panic!("invalid address {addr}")),
        Duration::from_millis(800),
    )
    .is_ok();

    ServiceHealthDto {
        id: id.into(),
        addr: addr.into(),
        reachable,
        detail: if reachable {
            format!("{addr} reachable")
        } else {
            format!("{addr} not listening")
        },
    }
}

fn truncate_hex(value: &str, max_chars: usize) -> String {
    if value.len() <= max_chars {
        value.to_string()
    } else {
        format!("{}…", &value[..max_chars])
    }
}
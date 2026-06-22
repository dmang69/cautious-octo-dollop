use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::{extract::State, routing::get, Json, Router};
use intentkernel_util::config::{load_config, read_version};
use intentkernel_util::paths::resolve_root;
use serde::Serialize;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{info, warn};

pub const INTENTD_HTTP_ADDR: &str = "127.0.0.1:8780";
const PROBE_TIMEOUT: Duration = Duration::from_millis(800);
const MONITOR_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct AppState {
    pub install_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceHealth {
    pub id: &'static str,
    pub addr: String,
    pub status: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntentdHealth {
    pub addr: String,
    pub status: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OsStatus {
    pub version: String,
    pub root: String,
    pub intentd: IntentdHealth,
    pub services: Vec<ServiceHealth>,
    pub ready: bool,
}

pub fn collect_status(install_root: Option<PathBuf>) -> Result<OsStatus> {
    let root = resolve_root(install_root);
    let config = load_config(&root).unwrap_or_default();
    let version = read_version(&root);

    let services = vec![
        probe_service("ai_runtime", &config.runtime_addr),
        probe_service("intent_verifier", &config.verifier_addr),
    ];

    let intentd = probe_intentd_http();
    let ready = services.iter().all(|s| s.status == "up");

    Ok(OsStatus {
        version,
        root: root.display().to_string(),
        intentd,
        services,
        ready,
    })
}

pub fn port_reachable(addr: &str) -> bool {
    let Ok(sock_addr) = addr.parse() else {
        return false;
    };
    TcpStream::connect_timeout(&sock_addr, PROBE_TIMEOUT).is_ok()
}

pub fn probe_service(id: &'static str, addr: &str) -> ServiceHealth {
    let ok = port_reachable(addr);
    ServiceHealth {
        id,
        addr: addr.to_string(),
        status: if ok { "up" } else { "down" },
        detail: if ok {
            format!("{addr} reachable")
        } else {
            format!("{addr} not listening")
        },
    }
}

pub fn probe_intentd_http() -> IntentdHealth {
    let ok = port_reachable(INTENTD_HTTP_ADDR);
    IntentdHealth {
        addr: INTENTD_HTTP_ADDR.into(),
        status: if ok { "up" } else { "down" },
        detail: if ok {
            format!("{INTENTD_HTTP_ADDR} listening")
        } else {
            format!("{INTENTD_HTTP_ADDR} not listening — run `intentd start`")
        },
    }
}

pub fn print_awareness(status: &OsStatus) {
    for service in &status.services {
        if service.status == "up" {
            info!(
                "verified {} at {} (already running, not spawned)",
                service.id, service.addr
            );
        } else {
            warn!(
                "missing {} at {} — start manually (e.g. cargo run -p {})",
                service.id,
                service.addr,
                match service.id {
                    "ai_runtime" => "ai-runtime",
                    "intent_verifier" => "intent-verifier",
                    other => other,
                }
            );
        }
    }
}

pub async fn run_server(install_root: Option<PathBuf>) -> Result<()> {
    let state = Arc::new(AppState { install_root: install_root.clone() });

    let monitor_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut ticker = interval(MONITOR_INTERVAL);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            match collect_status(monitor_state.install_root.clone()) {
                Ok(status) => {
                    if status.ready {
                        info!("monitor: all core services up");
                    } else {
                        let down: Vec<_> = status
                            .services
                            .iter()
                            .filter(|s| s.status != "up")
                            .map(|s| s.id)
                            .collect();
                        warn!("monitor: services down: {}", down.join(", "));
                    }
                }
                Err(err) => warn!("monitor: status probe failed: {err:#}"),
            }
        }
    });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/", get(health_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(INTENTD_HTTP_ADDR)
        .await
        .with_context(|| format!("bind HTTP health API on {INTENTD_HTTP_ADDR}"))?;

    info!("intentd health API listening on http://{INTENTD_HTTP_ADDR}/health");
    axum::serve(listener, app)
        .await
        .context("serve intentd health API")?;
    Ok(())
}

pub fn run_start_blocking(install_root: Option<PathBuf>) -> Result<()> {
    let status = collect_status(install_root.clone())?;
    println!("{}", serde_json::to_string_pretty(&status)?);
    print_awareness(&status);

    let rt = tokio::runtime::Runtime::new().context("create tokio runtime")?;
    rt.block_on(run_server(install_root))
}

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<OsStatus> {
    match collect_status(state.install_root.clone()) {
        Ok(status) => Json(status),
        Err(err) => Json(OsStatus {
            version: "unknown".into(),
            root: ".".into(),
            intentd: IntentdHealth {
                addr: INTENTD_HTTP_ADDR.into(),
                status: "up",
                detail: format!("status probe error: {err:#}"),
            },
            services: vec![],
            ready: false,
        }),
    }
}
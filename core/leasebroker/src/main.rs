use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use intentkernel_crypto::sign::{token_to_bytes, TokenIssuer};
use intentkernel_util::paths::resolve_root;
use leasebroker::{
    load_broker_key, parse_jti_hex, LeaseBroker, LeaseSummary, LEASEBROKER_HTTP_ADDR,
};
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "leasebroker", about = "IntentKernel lease renewal broker")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the lease broker daemon (1s tick loop + HTTP API)
    Run {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// List tracked leases as JSON
    List {
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
    /// Renew a lease by JTI (hex)
    Renew {
        jti: String,
        #[arg(long)]
        install_root: Option<PathBuf>,
    },
}

#[derive(Clone)]
struct AppState {
    broker: Arc<Mutex<LeaseBroker>>,
}

#[derive(Debug, Serialize)]
struct LeasesResponse {
    leases: Vec<LeaseSummary>,
}

#[derive(Debug, Serialize)]
struct RenewResponse {
    jti: String,
    exp: u64,
    state: String,
    renewal_count: u32,
    token_hex: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("leasebroker error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { install_root } => {
            let rt = tokio::runtime::Runtime::new().context("create tokio runtime")?;
            rt.block_on(run_daemon(install_root))
        }
        Commands::List { install_root } => list_leases(install_root),
        Commands::Renew { jti, install_root } => renew_lease(install_root, &jti),
    }
}

fn build_broker(install_root: Option<PathBuf>) -> Result<LeaseBroker> {
    let root = resolve_root(install_root);
    let kp = load_broker_key(&root)?;
    Ok(LeaseBroker::new(TokenIssuer::new(kp)))
}

fn list_leases(install_root: Option<PathBuf>) -> Result<()> {
    let broker = build_broker(install_root)?;
    let leases = broker.list();
    println!("{}", serde_json::to_string_pretty(&LeasesResponse { leases })?);
    Ok(())
}

fn renew_lease(install_root: Option<PathBuf>, jti_hex: &str) -> Result<()> {
    let mut broker = build_broker(install_root)?;
    let jti = parse_jti_hex(jti_hex)?;
    let token = broker.renew(&jti)?;
    let bytes = token_to_bytes(&token)?;
    let summary = broker
        .list()
        .into_iter()
        .find(|l| l.jti == jti_hex)
        .ok_or_else(|| anyhow::anyhow!("renewed lease not found in registry"))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&RenewResponse {
            jti: summary.jti,
            exp: summary.exp,
            state: summary.state,
            renewal_count: summary.renewal_count,
            token_hex: hex::encode(bytes),
        })?
    );
    Ok(())
}

async fn run_daemon(install_root: Option<PathBuf>) -> Result<()> {
    let broker = build_broker(install_root)?;
    let shared = Arc::new(Mutex::new(broker));
    let tick_broker = Arc::clone(&shared);

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let mut broker = tick_broker.lock().await;
            for action in broker.tick() {
                match action {
                    leasebroker::TickAction::EnteredRenewing { jti } => {
                        info!(
                            "lease {} entered RENEWING (heartbeat pending)",
                            hex::encode(&jti)
                        );
                    }
                    leasebroker::TickAction::Expired { jti } => {
                        warn!("lease {} EXPIRED — halt execution", hex::encode(&jti));
                    }
                }
            }
        }
    });

    let app = Router::new()
        .route("/leases", get(list_handler))
        .route("/renew/:jti", post(renew_handler))
        .with_state(AppState {
            broker: Arc::clone(&shared),
        });

    let listener = tokio::net::TcpListener::bind(LEASEBROKER_HTTP_ADDR)
        .await
        .with_context(|| format!("bind HTTP API on {LEASEBROKER_HTTP_ADDR}"))?;

    info!("leasebroker listening on http://{LEASEBROKER_HTTP_ADDR}");
    info!("  GET  /leases");
    info!("  POST /renew/{{jti}}");
    info!("tick interval: 1s");

    axum::serve(listener, app)
        .await
        .context("serve leasebroker HTTP API")?;
    Ok(())
}

async fn list_handler(State(state): State<AppState>) -> Json<LeasesResponse> {
    let broker = state.broker.lock().await;
    Json(LeasesResponse {
        leases: broker.list(),
    })
}

async fn renew_handler(
    State(state): State<AppState>,
    Path(jti): Path<String>,
) -> Result<Json<RenewResponse>, (StatusCode, Json<ErrorResponse>)> {
    let jti_bytes = parse_jti_hex(&jti).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let mut broker = state.broker.lock().await;
    let token = broker.renew(&jti_bytes).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let bytes = token_to_bytes(&token).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let summary = broker
        .list()
        .into_iter()
        .find(|l| l.jti == jti)
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "renewed lease missing from registry".into(),
                }),
            )
        })?;

    Ok(Json(RenewResponse {
        jti: summary.jti,
        exp: summary.exp,
        state: summary.state,
        renewal_count: summary.renewal_count,
        token_hex: hex::encode(bytes),
    }))
}
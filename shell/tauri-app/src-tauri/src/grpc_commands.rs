use std::fs;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, State};
use tokio::time::timeout;
use tonic::Request;

use crate::dto::{
    ConnectionStatusDto, ExecuteResultDto, GrpcKernelStatsDto, InvokeCapabilityResultDto,
    LookupResultDto, RevokeHandleResultDto, SchedulerPolicyDto, SystemSnapshotDto,
};
use crate::grpc::GrpcClientState;
use crate::intentkernel::v1::{
    Empty, ExecuteRequest, InvokeCapabilityRequest, LookupRequest, RegisterTokenRequest,
    RevokeHandleRequest, SchedulerOptimizeRequest, StreamMetricsRequest,
};

const RPC_TIMEOUT: Duration = Duration::from_secs(5);

#[tauri::command]
pub async fn grpc_connect(
    state: State<'_, Arc<GrpcClientState>>,
    endpoint: Option<String>,
) -> Result<ConnectionStatusDto, String> {
    timeout(RPC_TIMEOUT, state.connect(endpoint))
        .await
        .map_err(|_| "connect timed out after 5s".into())??;

    Ok(ConnectionStatusDto {
        connected: true,
        endpoint: state.endpoint.read().await.clone(),
    })
}

#[tauri::command]
pub async fn grpc_disconnect(
    state: State<'_, Arc<GrpcClientState>>,
) -> Result<ConnectionStatusDto, String> {
    state.disconnect().await;
    Ok(ConnectionStatusDto {
        connected: false,
        endpoint: state.endpoint.read().await.clone(),
    })
}

#[tauri::command]
pub async fn grpc_connection_status(
    state: State<'_, Arc<GrpcClientState>>,
) -> Result<ConnectionStatusDto, String> {
    Ok(ConnectionStatusDto {
        connected: state.is_connected().await,
        endpoint: state.endpoint.read().await.clone(),
    })
}

#[tauri::command]
pub async fn grpc_get_system_snapshot(
    state: State<'_, Arc<GrpcClientState>>,
) -> Result<SystemSnapshotDto, String> {
    let clients = state.clients().await?;
    let mut client = clients.telemetry.clone();

    let resp = timeout(RPC_TIMEOUT, client.get_system_snapshot(Request::new(Empty {})))
        .await
        .map_err(|_| "get_system_snapshot timed out after 5s".into())?
        .map_err(|e| format!("get_system_snapshot failed: {e}"))?
        .into_inner();

    Ok(resp.into())
}

#[tauri::command]
pub async fn grpc_start_metrics_stream(
    app: AppHandle,
    state: State<'_, Arc<GrpcClientState>>,
    interval_ms: u32,
) -> Result<(), String> {
    if let Some(cancel) = state.stream_cancel.write().await.take() {
        let _ = cancel.send(());
    }

    let clients = state.clients().await?;
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    *state.stream_cancel.write().await = Some(stop_tx);

    let mut client = clients.telemetry.clone();
    let req = StreamMetricsRequest {
        interval_ms: interval_ms.max(250),
    };

    let mut stream = timeout(
        RPC_TIMEOUT,
        client.stream_metrics(Request::new(req)),
    )
    .await
    .map_err(|_| "stream_metrics timed out after 5s".into())?
    .map_err(|e| format!("stream_metrics failed: {e}"))?
    .into_inner();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                item = stream.message() => {
                    match item {
                        Ok(Some(snapshot)) => {
                            let dto: SystemSnapshotDto = snapshot.into();
                            let _ = app.emit("metrics-snapshot", dto);
                        }
                        Ok(None) => break,
                        Err(e) => {
                            let _ = app.emit("metrics-error", format!("stream error: {e}"));
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn grpc_stop_metrics_stream(
    state: State<'_, Arc<GrpcClientState>>,
) -> Result<(), String> {
    if let Some(cancel) = state.stream_cancel.write().await.take() {
        let _ = cancel.send(());
    }
    Ok(())
}

#[tauri::command]
pub async fn grpc_get_scheduler_policy(
    state: State<'_, Arc<GrpcClientState>>,
) -> Result<SchedulerPolicyDto, String> {
    let clients = state.clients().await?;
    let mut client = clients.scheduler.clone();

    let resp = timeout(RPC_TIMEOUT, client.get_current_policy(Request::new(Empty {})))
        .await
        .map_err(|_| "get_current_policy timed out after 5s".into())?
        .map_err(|e| format!("get_current_policy failed: {e}"))?
        .into_inner();

    Ok(resp.into())
}

#[tauri::command]
pub async fn grpc_optimize_scheduler_policy(
    state: State<'_, Arc<GrpcClientState>>,
    telemetry: Vec<f32>,
) -> Result<SchedulerPolicyDto, String> {
    let clients = state.clients().await?;
    let mut client = clients.scheduler.clone();

    let resp = timeout(
        RPC_TIMEOUT,
        client.optimize_policy(Request::new(SchedulerOptimizeRequest { telemetry })),
    )
    .await
    .map_err(|_| "optimize_policy timed out after 5s".into())?
    .map_err(|e| format!("optimize_policy failed: {e}"))?
    .into_inner();

    Ok(resp.into())
}

#[tauri::command]
pub async fn grpc_lookup(
    state: State<'_, Arc<GrpcClientState>>,
    target: String,
) -> Result<LookupResultDto, String> {
    let clients = state.clients().await?;
    let mut client = clients.command.clone();

    let resp = timeout(
        RPC_TIMEOUT,
        client.lookup(Request::new(LookupRequest { target })),
    )
    .await
    .map_err(|_| "lookup timed out after 5s".into())?
    .map_err(|e| format!("lookup failed: {e}"))?
    .into_inner();

    Ok(resp.into())
}

#[tauri::command]
pub async fn grpc_execute(
    state: State<'_, Arc<GrpcClientState>>,
    handle: u64,
    action: u32,
) -> Result<ExecuteResultDto, String> {
    let clients = state.clients().await?;
    let mut client = clients.command.clone();

    let resp = timeout(
        RPC_TIMEOUT,
        client.execute(Request::new(ExecuteRequest { handle, action })),
    )
    .await
    .map_err(|_| "execute timed out after 5s".into())?
    .map_err(|e| format!("execute failed: {e}"))?
    .into_inner();

    Ok(resp.into())
}

fn parse_handle_hex(handle: &str) -> Result<u64, String> {
    u64::from_str_radix(handle.trim_start_matches("0x"), 16)
        .map_err(|e| format!("invalid handle hex: {e}"))
}

fn load_token_cbor(token_path: Option<String>, token_bytes: Option<Vec<u8>>) -> Result<Vec<u8>, String> {
    match (token_path, token_bytes) {
        (Some(path), None) => fs::read(&path).map_err(|e| format!("read token file: {e}")),
        (None, Some(bytes)) if !bytes.is_empty() => Ok(bytes),
        (Some(_), Some(_)) => Err("provide token_path or token_bytes, not both".into()),
        _ => Err("token_path or token_bytes required".into()),
    }
}

#[tauri::command]
pub async fn grpc_register_token(
    state: State<'_, Arc<GrpcClientState>>,
    token_path: Option<String>,
    token_bytes: Option<Vec<u8>>,
    resource_type: u32,
) -> Result<String, String> {
    let token_cbor = load_token_cbor(token_path, token_bytes)?;
    let clients = state.clients().await?;
    let mut client = clients.capability.clone();

    let resp = timeout(
        RPC_TIMEOUT,
        client.register_token(Request::new(RegisterTokenRequest {
            token_cbor,
            resource_type,
        })),
    )
    .await
    .map_err(|_| "register_token timed out after 5s".into())?
    .map_err(|e| format!("register_token failed: {e}"))?
    .into_inner();

    let handle_hex = resp.into_handle_hex()?;
    let handle = parse_handle_hex(&handle_hex)?;
    state.track_handle(handle);
    Ok(handle_hex)
}

#[tauri::command]
pub async fn grpc_invoke_capability(
    state: State<'_, Arc<GrpcClientState>>,
    handle: String,
    action: u32,
) -> Result<InvokeCapabilityResultDto, String> {
    let handle_raw = parse_handle_hex(&handle)?;
    let sequence = state.next_sequence(handle_raw);
    let clients = state.clients().await?;
    let mut client = clients.capability.clone();

    let resp = timeout(
        RPC_TIMEOUT,
        client.invoke_capability(Request::new(InvokeCapabilityRequest {
            handle: handle_raw,
            sequence,
            action,
        })),
    )
    .await
    .map_err(|_| "invoke_capability timed out after 5s".into())?
    .map_err(|e| format!("invoke_capability failed: {e}"))?
    .into_inner();

    Ok(resp.into())
}

#[tauri::command]
pub async fn grpc_revoke_handle(
    state: State<'_, Arc<GrpcClientState>>,
    handle: String,
) -> Result<RevokeHandleResultDto, String> {
    let handle_raw = parse_handle_hex(&handle)?;
    let clients = state.clients().await?;
    let mut client = clients.capability.clone();

    let resp = timeout(
        RPC_TIMEOUT,
        client.revoke_handle(Request::new(RevokeHandleRequest {
            handle: handle_raw,
        })),
    )
    .await
    .map_err(|_| "revoke_handle timed out after 5s".into())?
    .map_err(|e| format!("revoke_handle failed: {e}"))?
    .into_inner();

    if resp.ok {
        state.untrack_handle(handle_raw);
    }

    Ok(resp.into())
}

#[tauri::command]
pub async fn grpc_get_kernel_stats(
    state: State<'_, Arc<GrpcClientState>>,
) -> Result<GrpcKernelStatsDto, String> {
    let clients = state.clients().await?;
    let mut client = clients.capability.clone();

    let resp = timeout(RPC_TIMEOUT, client.get_kernel_stats(Request::new(Empty {})))
        .await
        .map_err(|_| "get_kernel_stats timed out after 5s".into())?
        .map_err(|e| format!("get_kernel_stats failed: {e}"))?
        .into_inner();

    Ok(resp.into())
}
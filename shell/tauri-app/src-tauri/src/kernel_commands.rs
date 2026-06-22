use std::fs;
use std::sync::Mutex;



use intentkernel_core::gate::{SyscallRequest, SyscallResult};
use intentkernel_core::IntentKernel;
use intentkernel_crypto::cbor::decode_wire_token;
use intentkernel_crypto::sign::{PublicKey, TokenValidator};
use intentkernel_crypto::token::Algorithm;
use intentkernel_util::paths::resolve_root;
use serde::Serialize;
use tauri::State;

pub struct KernelState {
    pub kernel: Mutex<IntentKernel>,
    pub handles: Mutex<Vec<u64>>,
}

impl KernelState {
    pub fn new() -> Self {
        Self {
            kernel: Mutex::new(IntentKernel::new()),
            handles: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct KernelStatsDto {
    pub active_capabilities: usize,
    pub registered_handles: usize,
    pub session_handles: usize,
}

#[tauri::command]
pub fn kernel_stats(state: State<KernelState>) -> Result<KernelStatsDto, String> {
    let kernel = state.kernel.lock().map_err(|e| e.to_string())?;
    let handles = state.handles.lock().map_err(|e| e.to_string())?;
    let stats = kernel.stats();
    Ok(KernelStatsDto {
        active_capabilities: stats.active_capabilities,
        registered_handles: stats.registered_handles,
        session_handles: handles.len(),
    })
}

#[tauri::command]
pub fn kernel_register_token(
    state: State<KernelState>,
    token_path: String,
    resource_type: Option<u32>,
) -> Result<String, String> {
    let bytes = fs::read(&token_path).map_err(|e| e.to_string())?;
    let token = decode_wire_token(&bytes).map_err(|e| e.to_string())?;
    let validator = load_validator().map_err(|e| e.to_string())?;
    let mut kernel = state.kernel.lock().map_err(|e| e.to_string())?;
    let handle = kernel
        .gate()
        .register_token(&token, &validator, resource_type.unwrap_or(1))
        .map_err(|e| e.to_string())?;
    state
        .handles
        .lock()
        .map_err(|e| e.to_string())?
        .push(handle.raw);
    Ok(format!("0x{:016X}", handle.raw))
}

#[tauri::command]
pub fn kernel_invoke(
    state: State<KernelState>,
    handle_hex: String,
    action: Option<u32>,
) -> Result<String, String> {
    let handle = u64::from_str_radix(handle_hex.trim_start_matches("0x"), 16)
        .map_err(|e| e.to_string())?;
    let mut kernel = state.kernel.lock().map_err(|e| e.to_string())?;
    let seq = kernel.sequences.get(&handle).copied().unwrap_or(0) + 1;
    match kernel.gate().invoke(SyscallRequest {
        handle,
        sequence: seq,
        action: action.unwrap_or(0),
    }) {
        SyscallResult::Allowed { resource_type } => Ok(format!("ALLOWED type={resource_type}")),
        SyscallResult::Denied(e) => Ok(format!("DENIED: {e}")),
    }
}

#[tauri::command]
pub fn kernel_verify_token(token_path: String) -> Result<String, String> {
    let bytes = fs::read(&token_path).map_err(|e| e.to_string())?;
    let validator = load_validator().map_err(|e| e.to_string())?;
    let token = validator.validate_bytes(&bytes).map_err(|e| e.to_string())?;
    Ok(format!(
        "OK uses={} exp={}",
        token.payload.uses, token.payload.exp
    ))
}

fn load_validator() -> anyhow::Result<TokenValidator> {
    let root = resolve_root(None);
    let path = root.join("config/broker.key.json");
    let text = fs::read_to_string(&path)?;
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


use std::sync::Mutex;

/// Shared application state accessible from all Tauri commands.
#[derive(Default)]
pub struct AppState {
    /// gRPC endpoint for the AI Runtime daemon.
    pub grpc_endpoint: Mutex<String>,
}

mod dto;
mod grpc;
mod grpc_commands;
mod health_commands;
mod kernel_commands;

pub mod intentkernel {
    pub mod v1 {
        tonic::include_proto!("intentkernel.v1");
    }
}

use std::sync::Arc;

use grpc::GrpcClientState;
use kernel_commands::KernelState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let grpc_state = Arc::new(GrpcClientState::new("http://127.0.0.1:50051"));
    let kernel_state = KernelState::new();

    tauri::Builder::default()
        .manage(grpc_state)
        .manage(kernel_state)
        .invoke_handler(tauri::generate_handler![
            grpc_commands::grpc_connect,
            grpc_commands::grpc_disconnect,
            grpc_commands::grpc_connection_status,
            grpc_commands::grpc_get_system_snapshot,
            grpc_commands::grpc_start_metrics_stream,
            grpc_commands::grpc_stop_metrics_stream,
            grpc_commands::grpc_get_scheduler_policy,
            grpc_commands::grpc_optimize_scheduler_policy,
            grpc_commands::grpc_lookup,
            grpc_commands::grpc_execute,
            grpc_commands::grpc_register_token,
            grpc_commands::grpc_invoke_capability,
            grpc_commands::grpc_revoke_handle,
            grpc_commands::grpc_get_kernel_stats,
            kernel_commands::kernel_stats,
            kernel_commands::kernel_register_token,
            kernel_commands::kernel_invoke,
            kernel_commands::kernel_verify_token,
            health_commands::os_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
pub mod server;
pub mod client;

// Re-export generated protobuf types (tonic-build output)
pub mod proto {
    tonic::include_proto!("ai_os");
}

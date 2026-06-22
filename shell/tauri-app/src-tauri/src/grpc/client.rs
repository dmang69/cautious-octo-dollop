use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::intentkernel::v1::{
    capability_service_client::CapabilityServiceClient,
    command_service_client::CommandServiceClient,
    scheduler_service_client::SchedulerServiceClient,
    telemetry_service_client::TelemetryServiceClient,
};

pub struct GrpcClients {
    pub telemetry: TelemetryServiceClient<Channel>,
    pub scheduler: SchedulerServiceClient<Channel>,
    pub command: CommandServiceClient<Channel>,
    pub capability: CapabilityServiceClient<Channel>,
}

pub struct GrpcClientState {
    pub endpoint: RwLock<String>,
    pub clients: RwLock<Option<Arc<GrpcClients>>>,
    pub stream_cancel: RwLock<Option<tokio::sync::oneshot::Sender<()>>>,
    pub handle_sequences: Mutex<HashMap<u64, u64>>,
}

impl GrpcClientState {
    pub fn new(default_endpoint: &str) -> Self {
        Self {
            endpoint: RwLock::new(default_endpoint.to_string()),
            clients: RwLock::new(None),
            stream_cancel: RwLock::new(None),
            handle_sequences: Mutex::new(HashMap::new()),
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.clients.read().await.is_some()
    }

    pub async fn connect(&self, endpoint: Option<String>) -> Result<(), String> {
        let addr = match endpoint {
            Some(e) => e,
            None => self.endpoint.read().await.clone(),
        };

        let channel = Channel::from_shared(addr.clone())
            .map_err(|e| format!("invalid endpoint: {e}"))?
            .connect()
            .await
            .map_err(|e| format!("connect failed: {e}"))?;

        let clients = Arc::new(GrpcClients {
            telemetry: TelemetryServiceClient::new(channel.clone()),
            scheduler: SchedulerServiceClient::new(channel.clone()),
            command: CommandServiceClient::new(channel.clone()),
            capability: CapabilityServiceClient::new(channel),
        });

        *self.endpoint.write().await = addr;
        *self.clients.write().await = Some(clients);
        Ok(())
    }

    pub async fn disconnect(&self) {
        if let Some(cancel) = self.stream_cancel.write().await.take() {
            let _ = cancel.send(());
        }
        *self.clients.write().await = None;
        self.handle_sequences.lock().unwrap().clear();
    }

    pub fn next_sequence(&self, handle: u64) -> u64 {
        let mut sequences = self.handle_sequences.lock().unwrap();
        let seq = sequences.get(&handle).copied().unwrap_or(0) + 1;
        sequences.insert(handle, seq);
        seq
    }

    pub fn track_handle(&self, handle: u64) {
        self.handle_sequences.lock().unwrap().entry(handle).or_insert(0);
    }

    pub fn untrack_handle(&self, handle: u64) {
        self.handle_sequences.lock().unwrap().remove(&handle);
    }

    pub async fn clients(&self) -> Result<Arc<GrpcClients>, String> {
        self.clients
            .read()
            .await
            .clone()
            .ok_or_else(|| "not connected to gRPC endpoint".into())
    }
}
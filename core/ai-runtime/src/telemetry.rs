use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};

use crate::intentkernel::v1::{
    telemetry_service_server::TelemetryService, Empty, StreamMetricsRequest, SystemSnapshot,
};

static START: AtomicU64 = AtomicU64::new(0);

fn sample() -> SystemSnapshot {
    let mut rng = rand::thread_rng();
    SystemSnapshot {
        cpu_percent: 12.0 + rng.gen::<f32>() * 35.0,
        mem_percent: 28.0 + rng.gen::<f32>() * 40.0,
        disk_io_mbps: rng.gen::<f32>() * 12.0,
        net_rx_mbps: rng.gen::<f32>() * 8.0,
        net_tx_mbps: rng.gen::<f32>() * 4.0,
        queue_depth: rng.gen_range(0..32),
        uptime_secs: START.load(Ordering::Relaxed),
        process_count: rng.gen_range(40..220),
        ipc_queued: rng.gen_range(0..8),
    }
}

pub struct TelemetryServiceImpl;

impl TelemetryServiceImpl {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        START.store(now, Ordering::Relaxed);
        Self
    }
}

#[tonic::async_trait]
impl TelemetryService for TelemetryServiceImpl {
    async fn get_system_snapshot(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<SystemSnapshot>, Status> {
        Ok(Response::new(sample()))
    }

    type StreamMetricsStream =
        tokio_stream::wrappers::ReceiverStream<Result<SystemSnapshot, Status>>;

    async fn stream_metrics(
        &self,
        req: Request<StreamMetricsRequest>,
    ) -> Result<Response<Self::StreamMetricsStream>, Status> {
        let ms = req.into_inner().interval_ms.max(250) as u64;
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        tokio::spawn(async move {
            loop {
                if tx.send(Ok(sample())).await.is_err() {
                    break;
                }
                sleep(Duration::from_millis(ms)).await;
            }
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
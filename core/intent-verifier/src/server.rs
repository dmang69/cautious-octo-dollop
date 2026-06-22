use std::net::SocketAddr;

use anyhow::Result;
use intentkernel_platform::stop_requested;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::verify_bytes;

pub async fn run() -> Result<()> {
    let addr: SocketAddr = std::env::var("INTENT_VERIFIER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:7879".to_string())
        .parse()?;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "intent-verifier listening (IKTK wire format)");

    loop {
        if stop_requested() {
            tracing::info!("shutdown requested");
            break;
        }

        tokio::select! {
            accept = listener.accept() => {
                let (mut socket, peer) = accept?;
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 65536];
                    match socket.read(&mut buf).await {
                        Ok(0) => {}
                        Ok(n) => {
                            let response = match verify_bytes(&buf[..n]) {
                                Ok(summary) => format!(r#"{{"ok":true,"summary":"{summary}"}}"#),
                                Err(err) => format!(r#"{{"ok":false,"error":"{err}"}}"#),
                            };
                            let _ = socket.write_all(response.as_bytes()).await;
                        }
                        Err(err) => tracing::warn!(%peer, %err, "read failed"),
                    }
                });
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(250)) => {}
        }
    }

    Ok(())
}
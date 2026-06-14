#[cfg(test)]
mod tests {
    use ipc::client::AiOsClient;
    use ipc::proto::TelemetrySnapshot;

    /// Verify the gRPC server starts and accepts a connection.
    #[tokio::test]
    async fn test_grpc_server_starts() {
        let addr = "127.0.0.1:50099";
        let server = tokio::spawn(ipc::server::serve(addr));
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let result = AiOsClient::connect(&format!("http://{}", addr)).await;
        assert!(result.is_ok(), "gRPC client should connect: {:?}", result);

        server.abort();
    }

    /// Verify get_recommendations returns a valid (possibly empty) response.
    #[tokio::test]
    async fn test_get_recommendations_empty_snapshot() {
        let addr = "127.0.0.1:50098";
        let _server = tokio::spawn(ipc::server::serve(addr));
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let mut client = AiOsClient::connect(&format!("http://{}", addr))
            .await
            .expect("connect");
        let snapshot = TelemetrySnapshot {
            timestamp_ms: 0,
            cpu_avg: 0.0,
            cpu_cores: vec![],
            memory_used_bytes: 0,
            memory_total_bytes: 0,
            process_count: 0,
        };
        let recs = client.get_recommendations(snapshot).await.expect("rpc");
        assert!(recs.is_empty());
    }
}

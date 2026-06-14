# ai-runtime

The AI Context Manager daemon. It collects system telemetry, runs ONNX inference models, and exposes results over gRPC.

## Architecture

```
main.rs
 └─ ContextManager          (context_manager.rs)
     ├─ TelemetryCollector  (telemetry.rs)
     └─ InferenceEngine     (inference.rs)
```

## Configuration

Edit `config.toml` to change gRPC address, model paths, and telemetry interval.

## Running

```bash
cargo run --release --bin ai-runtime -- --config config.toml
```

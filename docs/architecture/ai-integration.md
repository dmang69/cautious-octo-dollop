# AI Integration

## Models

| Model | Purpose | Format | Location |
|-------|---------|--------|----------|
| `scheduler_v1` | RL-trained process priority optimizer | ONNX | `models/pretrained/` |
| `command_interpreter_v1` | NLP command-to-intent mapper | ONNX | `models/pretrained/` |

## Inference Pipeline

```
TelemetrySnapshot ──► encode as float32 tensor
                  ──► ort::Session::run()
                  ──► decode output tensor
                  ──► Vec<PriorityRecommendation>
```

The inference engine runs synchronously within a Tokio blocking task to avoid blocking the async executor.

## ONNX Runtime

ONNX Runtime is loaded dynamically via the `ort` crate. Set `ORT_LIB_LOCATION` to the directory containing `libonnxruntime.so` (Linux), `onnxruntime.dll` (Windows), or `libonnxruntime.dylib` (macOS).

## Sandboxing

All model inference runs within a WASM sandbox using `wasmtime` (planned for v1.2). Until then, model files are integrity-verified on load using SHA-256 hashes stored in `models/pretrained/model_registry.json`.

## Training

See [`guides/model-training.md`](../guides/model-training.md) for the full training pipeline.

# gRPC API Reference

See [`core/ipc/proto/ai_os_services.proto`](../../core/ipc/proto/ai_os_services.proto) for the full Protobuf definitions.

## Services

### SchedulerService

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `GetRecommendations` | `SchedulerRequest` | `SchedulerResponse` | One-shot priority recommendations |
| `StreamRecommendations` | `TelemetrySnapshot` | `stream PriorityRecommendation` | Streaming recommendations |

### CommandInterpreterService

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `Interpret` | `CommandRequest` | `CommandResponse` | Convert raw command to structured intent |

### TelemetryService

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `StreamTelemetry` | `TelemetryRequest` | `stream TelemetrySnapshot` | Live telemetry stream |
| `GetSnapshot` | `TelemetryRequest` | `TelemetrySnapshot` | Single telemetry snapshot |

## Default Endpoint

`http://127.0.0.1:50051`

Configure via `core/ai-runtime/config.toml` → `[grpc]`.

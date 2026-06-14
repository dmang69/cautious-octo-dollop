# System Overview

## Architecture

The AI OS is a four-layer stack that integrates AI inference directly into OS scheduling and user interaction.

```
┌──────────────────────────────────────────────────┐
│  shell/tauri-app   (Cross-platform Desktop UI)    │
│  React + Tauri     Terminal / ProcessTable / Metrics│
└──────────────────────────┬───────────────────────┘
                           │ Tauri IPC / gRPC
┌──────────────────────────▼───────────────────────┐
│  core/ipc          (gRPC Services)                │
│  SchedulerService / CommandInterpreter / Telemetry│
└──────────────────────────┬───────────────────────┘
                           │ Rust traits
┌──────────────────────────▼───────────────────────┐
│  core/ai-runtime   (AI Context Manager Daemon)    │
│  TelemetryCollector → InferenceEngine → gRPC      │
└──────────────────────────┬───────────────────────┘
                           │ KernelInterface trait
┌──────────────────────────▼───────────────────────┐
│  core/kernel-interface  (Platform Abstraction)    │
│  Linux / Windows / macOS syscall implementations  │
└──────────────────────────────────────────────────┘
```

## Data Flow

1. `TelemetryCollector` polls the OS via `KernelInterface` every 500 ms.
2. Telemetry is fed to `InferenceEngine` which runs the ONNX scheduler model.
3. Priority recommendations are published over gRPC (`SchedulerService`).
4. The Tauri shell subscribes to gRPC streams and displays live data.
5. When a user types a natural-language command, the `CommandInterpreterService` converts it to a structured intent before execution.

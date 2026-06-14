# Building from Source

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | 20+ | [https://nodejs.org](https://nodejs.org) |
| ONNX Runtime | 1.16 | See below |

### Install ONNX Runtime (Linux)

```bash
wget https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-x64-1.16.0.tgz
tar -xzf onnxruntime-linux-x64-1.16.0.tgz
export ORT_LIB_LOCATION=$(pwd)/onnxruntime-linux-x64-1.16.0/lib
```

## Building the Core Runtime

```bash
cd core/ai-runtime
cargo build --release
```

The binary is produced at `core/ai-runtime/target/release/ai-runtime`.

## Building the Shell

```bash
cd shell/tauri-app
npm install
npm run tauri:build
```

Installers are placed in `shell/tauri-app/src-tauri/target/release/bundle/`.

## Running Tests

```bash
# Integration tests
cargo test --workspace

# Benchmarks
cargo bench --workspace
```

## Quick Start (all in one)

```bash
./scripts/setup-dev.sh
./scripts/build-all.sh
```

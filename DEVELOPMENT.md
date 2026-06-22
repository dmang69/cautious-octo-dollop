# Development Guide

## Primary path: IntentOS (`rust/`)

```bash
cd rust
cargo run -p intentos --release
cargo test
```

## Legacy IKRL stack (repo root workspace)

```bash
cargo build --release
cargo run -p capd -- init --algorithm ed25519
cargo run -p ai-runtime &
cargo run -p intent-verifier &
cargo run -p intentd -- start
```

## ISO staging (Linux/WSL)

```bash
./scripts/stage-iso.sh
./scripts/build-iso.sh
```

## Tauri shell

```bash
cd shell/tauri-app
npm install
npm run build
# Desktop: npm run tauri dev  (requires libdbus on Linux)
```

## Next development targets

1. `ransomware-demo` crate under `rust/crates/` — VFS denial without per-file handles
2. Wire Tauri shell fully to remote `CapabilityService` when ai-runtime is up
3. Migrate legacy crates into `rust/crates/ikrl-*` naming
4. PQC feature flag for `intentos-kernel` crypto (ML-DSA-87 behind `--features pqc`)
5. Host filesystem eventscope bridge (optional, behind feature flag)

## Broker key setup

```bash
cp config/broker.key.json.example config/broker.key.json
cargo run -p capd -- init --algorithm ed25519
```

Never commit `config/broker.key.json`.
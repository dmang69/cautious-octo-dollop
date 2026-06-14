# Security Model

## Threat Model

The AI OS daemon runs with elevated privileges to adjust process priorities. The attack surface is minimized through:

1. **Principle of Least Privilege** — the daemon runs as a dedicated low-privilege user (`aios` / `_aios`).
2. **Seccomp / App Sandbox** — only a whitelist of syscalls is permitted (see platform-specific install scripts).
3. **No Network Access** — the inference engine has no outbound network capability.
4. **Model Integrity** — ONNX models are verified against SHA-256 hashes in `model_registry.json` before loading.
5. **mTLS for gRPC** — all gRPC communication uses mutual TLS with short-lived certificates (planned for v1.2).

## Privilege Boundaries

| Component | Privilege | Reason |
|-----------|-----------|--------|
| `ai-runtime` daemon | `aios` user | Needs `setpriority` on other processes |
| Tauri shell | current user | UI only — no kernel interaction |
| gRPC server | loopback only | No remote access by default |

## Supply Chain

- ONNX Runtime is verified via official release checksums.
- All Rust dependencies are audited with `cargo audit` in CI.
- Python training dependencies are pinned in `requirements.txt`.

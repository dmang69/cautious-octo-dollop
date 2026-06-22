# IntentOS Rust Reference Runtime

The primary runnable path in this repository is the **in-process IntentOS reference runtime** built from three crates plus one entry binary.

## Components

| # | Component | Crate / Binary | Role |
|---|-----------|----------------|------|
| 1 | Utilities | `intentos-utilities` | In-memory VFS, AI stub gateway |
| 2 | Shell | `intentos-shell` | Interactive REPL and command dispatch |
| 3 | Kernel | `intentos-kernel` | Policy, tokens, capability table, leases |
| — | Entry point | `intentos` | Boots all three in one process |

## Build and run

```bash
cd rust
cargo run -p intentos --release
```

## Shell commands

| Command | Description |
|---------|-------------|
| `status` | JSON session status (handles, stats, VFS size) |
| `flow <action> <resource> [anchor]` | Evaluate intent, mint token, register handle |
| `ls` | List in-memory VFS paths |
| `cat <path>` | Read gated file (requires active handle) |
| `write <path> <text>` | Write gated file (requires active handle) |
| `ai infer <prompt>` | Stub AI response after kernel authorization |
| `lease list\|tick\|grant\|renew` | Lease tracking demo |

## Example session

```
intentos> flow vfs:read /notes.txt ui
flow ok handle=0x000000000001XXXX jti=... exp=...

intentos> cat /notes.txt
event-scoped authority demo

intentos> status
{ "subject": "intentos-session", "handle": "0x...", ... }
```

## Tests

```bash
cargo test -p intentos-kernel
cargo test -p intentos
```

`crates/intentos/tests/ground_up.rs` verifies the intentos dependency graph does **not** pull in legacy IKRL daemon crates (`capd`, `intentd`, `eventscope`, etc.).

## Cryptography note

`intentos-kernel/src/crypto.rs` uses **Ed25519 + SHA3-384** for development signing. This exercises token flow but is **not** production post-quantum cryptography.

## Relationship to legacy IKRL

The legacy multi-process stack (`capd`, `intentd`, `leasebroker`, `eventscope`, `ai-runtime`, etc.) lives outside this workspace under the repository root. It is experimental context — not the main runnable path documented here.
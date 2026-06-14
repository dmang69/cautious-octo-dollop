# kernel-interface

Platform abstraction layer providing a unified API over OS-specific system calls for Linux, Windows, and macOS.

## Overview

This crate exposes a single `KernelInterface` trait implemented per-platform so the rest of the AI OS stack can remain OS-agnostic.

## Supported Platforms

| Platform | Module       | Status  |
|----------|-------------|---------|
| Linux    | `linux.rs`  | Active  |
| Windows  | `windows.rs`| Active  |
| macOS    | `macos.rs`  | Active  |

## Usage

```rust
use kernel_interface::KernelInterface;

let ki = kernel_interface::platform();
let procs = ki.list_processes()?;
```

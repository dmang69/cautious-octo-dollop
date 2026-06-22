//! IntentKernel EventScope Phase 2 — eBPF LSM hooks and userspace loader.
//!
//! Default build compiles policy + mock bridge only. Enable `--features bpf` for aya.

pub mod bridge;
pub mod policy;

#[path = "../userspace/loader.rs"]
pub mod loader;

pub use bridge::{
    bridge_is_loaded, global_bridge, publish_handle, replace_global_bridge, revoke_pid,
    BridgeError, KernelBridge, MockKernelBridge,
};
pub use policy::{
    evaluate_hook, HandleMapEntry, HandleMapLookup, PolicyDenyReason, PolicyVerdict, SyscallHook,
    HANDLE_MAP_NAME, PROG_FILE_OPEN, PROG_SOCKET_CONNECT, RESOURCE_FILE, RESOURCE_NETWORK,
    RESOURCE_RAW,
};
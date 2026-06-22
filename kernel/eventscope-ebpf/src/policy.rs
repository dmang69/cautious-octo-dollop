//! Shared policy logic used by eBPF programs (mirrored in `bpf/eventscope.bpf.c`),
//! the LSM daemon, and userspace mock tests.

use thiserror::Error;

/// Resource type identifiers — must match `eventscope.bpf.c` and userspace EventScope.
pub const RESOURCE_FILE: u32 = 1;
pub const RESOURCE_NETWORK: u32 = 2;
pub const RESOURCE_RAW: u32 = 3;

/// BPF map name shared between kernel program and userspace loader.
pub const HANDLE_MAP_NAME: &str = "handle_map";

/// eBPF program section names (LSM hooks).
pub const PROG_FILE_OPEN: &str = "eventscope_file_open";
pub const PROG_SOCKET_CONNECT: &str = "eventscope_socket_connect";

/// Userspace-published capability binding for a process.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandleMapEntry {
    pub handle: u64,
    pub pid: u32,
    pub resource_type: u32,
    pub valid: u8,
    pub _pad: [u8; 3],
}

impl HandleMapEntry {
    pub fn new(pid: u32, handle: u64, resource_type: u32) -> Self {
        Self {
            handle,
            pid,
            resource_type,
            valid: 1,
            _pad: [0; 3],
        }
    }

    pub fn revoked(pid: u32) -> Self {
        Self {
            handle: 0,
            pid,
            resource_type: 0,
            valid: 0,
            _pad: [0; 3],
        }
    }

    pub fn is_active(&self) -> bool {
        self.valid != 0 && self.handle != 0
    }
}

/// Syscall surfaces hooked in Phase 2 PoC (openat → file_open LSM, connect → socket_connect LSM).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallHook {
    OpenAt,
    Connect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyVerdict {
    Allow,
    Deny(PolicyDenyReason),
}

impl PolicyVerdict {
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    pub fn is_deny(&self) -> bool {
        matches!(self, Self::Deny(_))
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDenyReason {
    #[error("no capability handle in BPF map for pid")]
    NoHandle,
    #[error("handle resource type does not permit this syscall")]
    WrongResourceType,
    #[error("revoked or invalid map entry")]
    InvalidEntry,
}

/// Trait abstracting the BPF `handle_map` for mock tests and live loaders.
pub trait HandleMapLookup {
    fn lookup(&self, pid: u32) -> Option<HandleMapEntry>;
}

impl HandleMapLookup for std::collections::HashMap<u32, HandleMapEntry> {
    fn lookup(&self, pid: u32) -> Option<HandleMapEntry> {
        self.get(&pid).copied()
    }
}

/// Evaluate LSM policy for a hooked syscall using the current map snapshot.
pub fn evaluate_hook<M: HandleMapLookup>(
    hook: SyscallHook,
    pid: u32,
    map: &M,
) -> PolicyVerdict {
    let Some(entry) = map.lookup(pid) else {
        return PolicyVerdict::Deny(PolicyDenyReason::NoHandle);
    };

    if !entry.is_active() {
        return PolicyVerdict::Deny(PolicyDenyReason::InvalidEntry);
    }

    let required = match hook {
        SyscallHook::OpenAt => RESOURCE_FILE,
        SyscallHook::Connect => RESOURCE_NETWORK,
    };

    if entry.resource_type != required {
        return PolicyVerdict::Deny(PolicyDenyReason::WrongResourceType);
    }

    PolicyVerdict::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_requires_file_handle() {
        let mut map = std::collections::HashMap::new();
        map.insert(100, HandleMapEntry::new(100, 42, RESOURCE_FILE));
        assert!(evaluate_hook(SyscallHook::OpenAt, 100, &map).is_allow());
        assert!(evaluate_hook(SyscallHook::Connect, 100, &map).is_deny());
    }

    #[test]
    fn connect_requires_network_handle() {
        let mut map = std::collections::HashMap::new();
        map.insert(200, HandleMapEntry::new(200, 99, RESOURCE_NETWORK));
        assert!(evaluate_hook(SyscallHook::Connect, 200, &map).is_allow());
        assert!(evaluate_hook(SyscallHook::OpenAt, 200, &map).is_deny());
    }
}
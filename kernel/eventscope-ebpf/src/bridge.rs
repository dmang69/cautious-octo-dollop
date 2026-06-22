//! Bridge between userspace EventScope and the kernel BPF map.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use thiserror::Error;

use crate::policy::HandleMapEntry;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("kernel BPF loader is not active (map unavailable)")]
    NotLoaded,
    #[error("invalid handle or map entry")]
    InvalidEntry,
    #[error("BPF operation failed: {0}")]
    Bpf(String),
}

/// Writes capability handles into the BPF map (or an in-memory mock).
pub trait KernelBridge: Send {
    fn is_loaded(&self) -> bool;
    fn publish(&mut self, entry: HandleMapEntry) -> Result<(), BridgeError>;
    fn revoke(&mut self, pid: u32) -> Result<(), BridgeError>;
    fn snapshot(&self) -> HashMap<u32, HandleMapEntry>;
}

/// In-memory stand-in for `handle_map` — used in tests and when BPF cannot load.
#[derive(Debug, Default)]
pub struct MockKernelBridge {
    map: HashMap<u32, HandleMapEntry>,
    loaded: bool,
}

impl MockKernelBridge {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            loaded: true,
        }
    }

    pub fn inactive() -> Self {
        Self {
            map: HashMap::new(),
            loaded: false,
        }
    }
}

impl KernelBridge for MockKernelBridge {
    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn publish(&mut self, entry: HandleMapEntry) -> Result<(), BridgeError> {
        if !self.loaded {
            return Err(BridgeError::NotLoaded);
        }
        if entry.pid == 0 || entry.handle == 0 {
            return Err(BridgeError::InvalidEntry);
        }
        self.map.insert(entry.pid, entry);
        Ok(())
    }

    fn revoke(&mut self, pid: u32) -> Result<(), BridgeError> {
        if !self.loaded {
            return Err(BridgeError::NotLoaded);
        }
        self.map.insert(pid, HandleMapEntry::revoked(pid));
        Ok(())
    }

    fn snapshot(&self) -> HashMap<u32, HandleMapEntry> {
        self.map.clone()
    }
}

static GLOBAL_BRIDGE: OnceLock<Mutex<Box<dyn KernelBridge>>> = OnceLock::new();

fn default_bridge() -> Box<dyn KernelBridge> {
    Box::new(MockKernelBridge::new())
}

/// Access the process-global bridge (mock by default; replaced when BPF loader starts).
pub fn global_bridge() -> &'static Mutex<Box<dyn KernelBridge>> {
    GLOBAL_BRIDGE.get_or_init(|| Mutex::new(default_bridge()))
}

/// Replace the global bridge after initialization (used by loader + tests).
pub fn replace_global_bridge(bridge: Box<dyn KernelBridge>) {
    if let Some(slot) = GLOBAL_BRIDGE.get() {
        *slot.lock().expect("bridge lock") = bridge;
    } else {
        let _ = GLOBAL_BRIDGE.set(Mutex::new(bridge));
    }
}

pub fn publish_handle(entry: HandleMapEntry) -> Result<(), BridgeError> {
    global_bridge()
        .lock()
        .expect("bridge lock")
        .publish(entry)
}

pub fn revoke_pid(pid: u32) -> Result<(), BridgeError> {
    global_bridge()
        .lock()
        .expect("bridge lock")
        .revoke(pid)
}

pub fn bridge_is_loaded() -> bool {
    global_bridge()
        .lock()
        .expect("bridge lock")
        .is_loaded()
}
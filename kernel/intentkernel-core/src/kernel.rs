use std::collections::HashMap;

use crate::capability::CapabilityTable;
use crate::gate::SyscallGate;
use crate::handle::HandleRegistry;

/// IntentKernel microkernel state — capability table + handle registry + syscall gate.
pub struct IntentKernel {
    pub capabilities: CapabilityTable,
    pub handles: HandleRegistry,
    pub token_bindings: HashMap<u64, (u16, u32)>,
    pub sequences: HashMap<u64, u64>,
}

impl Default for IntentKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentKernel {
    pub fn new() -> Self {
        Self {
            capabilities: CapabilityTable::new(),
            handles: HandleRegistry::new(),
            token_bindings: HashMap::new(),
            sequences: HashMap::new(),
        }
    }

    pub fn gate(&mut self) -> SyscallGate<'_> {
        SyscallGate::new(self)
    }

    pub fn stats(&self) -> KernelStats {
        KernelStats {
            active_capabilities: self.capabilities.active_count(),
            registered_handles: self.token_bindings.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KernelStats {
    pub active_capabilities: usize,
    pub registered_handles: usize,
}
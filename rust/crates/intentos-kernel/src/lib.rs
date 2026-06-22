pub mod crypto;
pub mod lease;
pub mod policy;
pub mod token;

use std::collections::HashMap;

use rand::RngCore;
use thiserror::Error;

use crate::crypto::DevKeyPair;
use crate::lease::LeaseRegistry;
use crate::policy::{evaluate, Intent, PolicyDecision};
use crate::token::{mint_token, CapabilityToken, TokenType};

pub const CAP_TABLE_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy)]
struct CapabilitySlot {
    key: [u8; 32],
    expires: u64,
    resource_type: u32,
    resource: u64,
    uses: u16,
}

impl Default for CapabilitySlot {
    fn default() -> Self {
        Self {
            key: [0; 32],
            expires: 0,
            resource_type: 0,
            resource: 0,
            uses: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KernelHandle {
    pub raw: u64,
    pub table_index: u32,
}

#[derive(Debug, Error, Clone)]
pub enum KernelError {
    #[error("capability denied: {0}")]
    Denied(String),
    #[error("table full")]
    TableFull,
}

pub struct Kernel {
    broker: DevKeyPair,
    slots: Vec<CapabilitySlot>,
    generations: Vec<u16>,
    bindings: HashMap<u64, (u32, String)>,
    sequences: HashMap<u64, u64>,
    pub leases: LeaseRegistry,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            broker: DevKeyPair::generate(),
            slots: vec![CapabilitySlot::default(); CAP_TABLE_SIZE],
            generations: vec![0; CAP_TABLE_SIZE],
            bindings: HashMap::new(),
            sequences: HashMap::new(),
            leases: LeaseRegistry::default(),
        }
    }

    pub fn with_broker(broker: DevKeyPair) -> Self {
        let mut k = Self::new();
        k.broker = broker;
        k
    }

    pub fn broker_public_key(&self) -> Vec<u8> {
        self.broker.public_key_bytes().to_vec()
    }

    pub fn now_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    pub fn evaluate_intent(&self, intent: &Intent) -> Result<PolicyDecision, KernelError> {
        evaluate(intent).map_err(|e| KernelError::Denied(e.to_string()))
    }

    pub fn mint_for_intent(&self, subject: &str, intent: &Intent) -> Result<CapabilityToken, KernelError> {
        let decision = self.evaluate_intent(intent)?;
        Ok(mint_token(
            &self.broker,
            subject,
            intent,
            &decision,
            Self::now_ms(),
        ))
    }

    pub fn register_token(&mut self, token: &CapabilityToken) -> Result<KernelHandle, KernelError> {
        token
            .verify(&self.broker.public_key_bytes(), Self::now_ms())
            .map_err(|e| KernelError::Denied(e.to_string()))?;

        if token.payload.typ == TokenType::Lease {
            self.leases
                .register(token)
                .map_err(KernelError::Denied)?;
        }

        let ttl = token.payload.exp.saturating_sub(token.payload.nbf);
        let uses = token.payload.uses.min(u16::MAX as u32) as u16;
        let id = self.alloc_slot(token.payload.resource_type, ttl, uses)?;
        let handle = self.mint_handle(id);
        self.bindings
            .insert(handle.raw, (id, token.payload.resource.clone()));
        Ok(handle)
    }

    pub fn invoke(&mut self, handle: u64, sequence: u64, _action: u32) -> Result<u32, KernelError> {
        let decoded = decode_handle(handle)?;
        if let Some(last) = self.sequences.get(&handle) {
            if *last >= sequence {
                return Err(KernelError::Denied("replay detected".into()));
            }
        }
        self.sequences.insert(handle, sequence);

        let idx = decoded.table_index as usize;
        let now = Self::now_ms();
        if self.slots[idx].expires < now || self.slots[idx].uses == 0 {
            return Err(KernelError::Denied("capability expired".into()));
        }

        let resource_type = self.slots[idx].resource_type;
        self.slots[idx].uses = self.slots[idx].uses.saturating_sub(1);
        if self.slots[idx].uses == 0 {
            self.slots[idx].expires = 0;
        }
        Ok(resource_type)
    }

    pub fn binding_resource(&self, handle: u64) -> Option<String> {
        self.bindings.get(&handle).map(|(_, r)| r.clone())
    }

    pub fn tick_leases(&mut self) {
        self.leases.tick(Self::now_ms());
    }

    pub fn stats(&self) -> KernelStats {
        let now = Self::now_ms();
        let active = self
            .slots
            .iter()
            .filter(|s| s.expires >= now && s.uses > 0)
            .count();
        KernelStats {
            active_capabilities: active,
            registered_handles: self.bindings.len(),
            active_leases: self.leases.list().len(),
        }
    }

    fn alloc_slot(&mut self, resource_type: u32, ttl_ms: u64, uses: u16) -> Result<u32, KernelError> {
        let now = Self::now_ms();
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.expires < now || slot.uses == 0 {
                let mut key = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                *slot = CapabilitySlot {
                    key,
                    expires: now + ttl_ms,
                    resource_type,
                    resource: 0,
                    uses,
                };
                return Ok(i as u32);
            }
        }
        Err(KernelError::TableFull)
    }

    fn mint_handle(&mut self, table_index: u32) -> KernelHandle {
        let idx = table_index as usize;
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        let gen = self.generations[idx];
        let checksum = (table_index ^ (gen as u32) << 16).wrapping_mul(0x9E37_79B9) as u16;
        let raw = ((table_index as u64) << 32) | ((gen as u64) << 16) | (checksum as u64);
        KernelHandle {
            raw,
            table_index,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct KernelStats {
    pub active_capabilities: usize,
    pub registered_handles: usize,
    pub active_leases: usize,
}

fn decode_handle(raw: u64) -> Result<KernelHandle, KernelError> {
    let table_index = (raw >> 32) as u32;
    let generation = ((raw >> 16) & 0xFFFF) as u16;
    let checksum = (raw & 0xFFFF) as u16;
    let expected = (table_index ^ (generation as u32) << 16).wrapping_mul(0x9E37_79B9) as u16;
    if expected != checksum {
        return Err(KernelError::Denied("invalid handle checksum".into()));
    }
    if table_index as usize >= CAP_TABLE_SIZE {
        return Err(KernelError::Denied("invalid handle index".into()));
    }
    Ok(KernelHandle { raw, table_index })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::TrustAnchor;

    #[test]
    fn mint_register_invoke() {
        let mut k = Kernel::new();
        let intent = Intent {
            action: "vfs:read".into(),
            resource: "/notes.txt".into(),
            anchor: TrustAnchor::UiEvent,
        };
        let token = k.mint_for_intent("session", &intent).unwrap();
        let handle = k.register_token(&token).unwrap();
        assert!(k.invoke(handle.raw, 1, 0).is_ok());
        assert!(k.invoke(handle.raw, 2, 0).is_err());
    }
}
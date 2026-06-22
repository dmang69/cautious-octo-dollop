use intentkernel_util::time::{ct_eq, now_ms};
use rand::RngCore;
use thiserror::Error;

pub const CAP_TABLE_SIZE: usize = 65536;
pub const CAP_KEY_SIZE: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct Capability {
    pub key: [u8; CAP_KEY_SIZE],
    pub expires: u64,
    pub resource_type: u32,
    pub uses: u16,
    pub id: u16,
}

impl Default for Capability {
    fn default() -> Self {
        Self {
            key: [0; CAP_KEY_SIZE],
            expires: 0,
            resource_type: 0,
            uses: 0,
            id: 0,
        }
    }
}

impl Capability {
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires < now
    }

    pub fn is_empty(&self, now: u64) -> bool {
        self.is_expired(now) || self.uses == 0
    }
}

#[derive(Debug, Error, Clone)]
pub enum CapabilityError {
    #[error("capability table full")]
    TableFull,
    #[error("invalid capability id")]
    InvalidId,
    #[error("capability expired")]
    Expired,
    #[error("capability exhausted")]
    Exhausted,
    #[error("capability key mismatch")]
    KeyMismatch,
}

pub struct CapabilityTable {
    slots: Vec<Capability>,
}

impl Default for CapabilityTable {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityTable {
    pub fn new() -> Self {
        Self {
            slots: vec![Capability::default(); CAP_TABLE_SIZE],
        }
    }

    pub fn create(&mut self, resource_type: u32, ttl_ms: u64, uses: u16) -> Result<u16, CapabilityError> {
        let now = now_ms();
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_empty(now) {
                let mut key = [0u8; CAP_KEY_SIZE];
                rand::thread_rng().fill_bytes(&mut key);
                *slot = Capability {
                    key,
                    expires: now + ttl_ms,
                    resource_type,
                    uses,
                    id: i as u16,
                };
                return Ok(i as u16);
            }
        }
        Err(CapabilityError::TableFull)
    }

    pub fn validate(&mut self, presented: &Capability) -> Result<u32, CapabilityError> {
        let now = now_ms();
        if presented.expires < now {
            return Err(CapabilityError::Expired);
        }
        if presented.uses == 0 {
            return Err(CapabilityError::Exhausted);
        }
        let id = presented.id as usize;
        if id >= CAP_TABLE_SIZE {
            return Err(CapabilityError::InvalidId);
        }
        let slot = &mut self.slots[id];
        if !ct_eq(&presented.key, &slot.key) {
            return Err(CapabilityError::KeyMismatch);
        }
        if slot.uses == 0 || slot.expires < now {
            return Err(CapabilityError::Expired);
        }
        slot.uses = slot.uses.saturating_sub(1);
        if slot.uses == 0 {
            slot.expires = 0;
        }
        Ok(slot.resource_type)
    }

    pub fn revoke(&mut self, id: u16) -> Result<(), CapabilityError> {
        let idx = id as usize;
        if idx >= CAP_TABLE_SIZE {
            return Err(CapabilityError::InvalidId);
        }
        let slot = &mut self.slots[idx];
        slot.expires = 0;
        slot.uses = 0;
        slot.key = [0; CAP_KEY_SIZE];
        Ok(())
    }

    pub fn get(&self, id: u16) -> Option<&Capability> {
        let idx = id as usize;
        if idx >= CAP_TABLE_SIZE {
            return None;
        }
        let slot = &self.slots[idx];
        if slot.is_empty(now_ms()) {
            None
        } else {
            Some(slot)
        }
    }

    pub fn active_count(&self) -> usize {
        let now = now_ms();
        self.slots.iter().filter(|s| !s.is_empty(now)).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_validate_consume() {
        let mut table = CapabilityTable::new();
        let id = table.create(7, 5000, 2).unwrap();
        let cap = table.get(id).unwrap().clone();
        assert_eq!(table.validate(&cap).unwrap(), 7);
        let cap2 = table.get(id).unwrap().clone();
        assert_eq!(table.validate(&cap2).unwrap(), 7);
        let cap3 = table.get(id);
        assert!(cap3.is_none() || cap3.unwrap().uses == 0);
    }
}
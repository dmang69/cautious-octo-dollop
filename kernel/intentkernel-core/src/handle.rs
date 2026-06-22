use thiserror::Error;

use crate::capability::{Capability, CAP_TABLE_SIZE};

/// 64-bit kernel handle per RFC-INTENT-001 §6.0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelHandle {
    pub raw: u64,
    pub table_index: u32,
    pub generation: u16,
}

#[derive(Debug, Error, Clone)]
pub enum HandleError {
    #[error("invalid handle checksum")]
    BadChecksum,
    #[error("handle generation mismatch")]
    StaleGeneration,
    #[error("handle index out of range")]
    InvalidIndex,
}

impl KernelHandle {
    pub fn encode(table_index: u32, generation: u16) -> Self {
        let checksum = Self::compute_checksum(table_index, generation);
        let raw = ((table_index as u64) << 32)
            | ((generation as u64) << 16)
            | (checksum as u64);
        Self {
            raw,
            table_index,
            generation,
        }
    }

    pub fn decode(raw: u64) -> Result<Self, HandleError> {
        let table_index = (raw >> 32) as u32;
        let generation = ((raw >> 16) & 0xFFFF) as u16;
        let checksum = (raw & 0xFFFF) as u16;
        if Self::compute_checksum(table_index, generation) != checksum {
            return Err(HandleError::BadChecksum);
        }
        if table_index as usize >= CAP_TABLE_SIZE {
            return Err(HandleError::InvalidIndex);
        }
        Ok(Self {
            raw,
            table_index,
            generation,
        })
    }

    fn compute_checksum(table_index: u32, generation: u16) -> u16 {
        let mut v = table_index ^ (generation as u32) << 16;
        v = v.wrapping_mul(0x9E37_79B9);
        (v ^ (v >> 16)) as u16
    }
}

pub struct HandleRegistry {
    generations: Vec<u16>,
}

impl Default for HandleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandleRegistry {
    pub fn new() -> Self {
        Self {
            generations: vec![0; CAP_TABLE_SIZE],
        }
    }

    pub fn mint(&mut self, table_index: u16) -> KernelHandle {
        let idx = table_index as usize;
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        KernelHandle::encode(table_index as u32, self.generations[idx])
    }

    pub fn verify(&self, handle: KernelHandle) -> Result<(), HandleError> {
        let idx = handle.table_index as usize;
        if idx >= CAP_TABLE_SIZE {
            return Err(HandleError::InvalidIndex);
        }
        if self.generations[idx] != handle.generation {
            return Err(HandleError::StaleGeneration);
        }
        Ok(())
    }

    pub fn invalidate(&mut self, table_index: u16) {
        let idx = table_index as usize;
        if idx < CAP_TABLE_SIZE {
            self.generations[idx] = self.generations[idx].wrapping_add(1);
        }
    }
}

pub fn capability_from_slot(slot: &Capability) -> Capability {
    slot.clone()
}
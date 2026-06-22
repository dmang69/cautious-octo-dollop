use serde::{Deserialize, Serialize};

use crate::token::{CapabilityToken, LeaseState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseRecord {
    pub jti: String,
    pub subject: String,
    pub resource: String,
    pub exp: u64,
    pub nbf: u64,
    pub state: LeaseState,
    pub renewal_count: u32,
}

#[derive(Default)]
pub struct LeaseRegistry {
    leases: Vec<LeaseRecord>,
}

impl LeaseRegistry {
    pub fn register(&mut self, token: &CapabilityToken) -> Result<(), String> {
        if token.payload.typ != crate::token::TokenType::Lease {
            return Err("not a lease token".into());
        }
        if token.payload.state != LeaseState::Granted {
            return Err("lease not granted".into());
        }
        self.leases.retain(|l| l.jti != token.payload.jti);
        self.leases.push(LeaseRecord {
            jti: token.payload.jti.clone(),
            subject: token.payload.sub.clone(),
            resource: token.payload.resource.clone(),
            exp: token.payload.exp,
            nbf: token.payload.nbf,
            state: token.payload.state,
            renewal_count: 0,
        });
        Ok(())
    }

    pub fn tick(&mut self, now_ms: u64) {
        for lease in &mut self.leases {
            if lease.state == LeaseState::Revoked || lease.state == LeaseState::Expired {
                continue;
            }
            let ttl = lease.exp.saturating_sub(lease.nbf);
            let elapsed = now_ms.saturating_sub(lease.nbf);
            if ttl > 0 && elapsed * 100 >= ttl * 80 && lease.state == LeaseState::Granted {
                lease.state = LeaseState::Renewing;
            }
            if now_ms >= lease.exp {
                lease.state = LeaseState::Expired;
            }
        }
    }

    pub fn renew(&mut self, jti: &str, new_exp: u64) -> Result<(), String> {
        let lease = self
            .leases
            .iter_mut()
            .find(|l| l.jti == jti)
            .ok_or_else(|| "lease not found".to_string())?;
        lease.exp = new_exp;
        lease.state = LeaseState::Granted;
        lease.renewal_count += 1;
        Ok(())
    }

    pub fn list(&self) -> &[LeaseRecord] {
        &self.leases
    }
}
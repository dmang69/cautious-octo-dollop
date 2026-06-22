use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::crypto::{context_hash, DevKeyPair};
use crate::policy::{Intent, PolicyDecision, TrustAnchor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    Capability = 1,
    Lease = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseState {
    Requested = 0,
    Granted = 1,
    Renewing = 2,
    Expired = 3,
    Revoked = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPayload {
    pub iss: Vec<u8>,
    pub sub: String,
    pub ctx: Vec<u8>,
    pub resource: String,
    pub exp: u64,
    pub nbf: u64,
    pub uses: u32,
    pub resource_type: u32,
    pub typ: TokenType,
    pub state: LeaseState,
    pub anchor: TrustAnchor,
    pub jti: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub payload: TokenPayload,
    pub signature: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("token expired")]
    Expired,
    #[error("invalid signature")]
    BadSignature,
    #[error("encoding error: {0}")]
    Encoding(String),
}

impl CapabilityToken {
    pub fn signing_bytes(&self) -> Result<Vec<u8>, TokenError> {
        serde_json::to_vec(&self.payload).map_err(|e| TokenError::Encoding(e.to_string()))
    }

    pub fn verify(&self, public_key: &[u8], now_ms: u64) -> Result<(), TokenError> {
        if now_ms < self.payload.nbf || now_ms >= self.payload.exp {
            return Err(TokenError::Expired);
        }
        let body = self.signing_bytes()?;
        if !DevKeyPair::verify(public_key, &body, &self.signature) {
            return Err(TokenError::BadSignature);
        }
        Ok(())
    }
}

pub fn mint_token(
    broker: &DevKeyPair,
    subject: &str,
    intent: &Intent,
    decision: &PolicyDecision,
    now_ms: u64,
) -> CapabilityToken {
    let typ = if intent.action.starts_with("lease:") {
        TokenType::Lease
    } else {
        TokenType::Capability
    };
    let state = if typ == TokenType::Lease {
        LeaseState::Granted
    } else {
        LeaseState::Granted
    };
    let payload = TokenPayload {
        iss: broker.issuer_id(),
        sub: subject.into(),
        ctx: context_hash(&intent.action).to_vec(),
        resource: intent.resource.clone(),
        exp: now_ms + decision.ttl_ms,
        nbf: now_ms,
        uses: decision.max_uses,
        resource_type: decision.resource_type,
        typ,
        state,
        anchor: intent.anchor,
        jti: Uuid::new_v4().to_string(),
    };
    let body = serde_json::to_vec(&payload).expect("payload serializes");
    let signature = broker.sign(&body);
    CapabilityToken { payload, signature }
}
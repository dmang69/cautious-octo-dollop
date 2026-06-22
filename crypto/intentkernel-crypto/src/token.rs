use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: u32 = 1;
pub const ML_DSA_87_SIG_LEN: usize = 4595;
pub const ED25519_SIG_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum TokenType {
    Capability = 1,
    Lease = 2,
    Delegation = 3,
    Revocation = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum Algorithm {
    MlDsa87 = 1,
    Ed25519 = 2,
    MlDsa65 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum TrustAnchor {
    None = 0,
    UiEvent = 1,
    Biometric = 2,
    Hardware = 3,
    Federated = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum LeaseState {
    Requested = 0,
    Granted = 1,
    Renewing = 2,
    Expired = 3,
    Revoked = 4,
    Suspended = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScope {
    pub path: String,
    pub access: u32,
    pub inode: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkScope {
    pub proto: u32,
    pub dst_ip: Vec<u8>,
    pub dst_port: u16,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResourceScope {
    File(FileScope),
    Network(NetworkScope),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHeader {
    pub ver: u32,
    pub typ: TokenType,
    pub alg: Algorithm,
    pub anchor: TrustAnchor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPayload {
    pub iss: Vec<u8>,
    pub sub: Vec<u8>,
    pub ctx: Vec<u8>,
    pub scope: ResourceScope,
    pub exp: u64,
    pub nbf: u64,
    pub uses: u32,
    pub state: LeaseState,
    pub jti: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct WireToken {
    pub header: TokenHeader,
    pub payload: TokenPayload,
    pub signature: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("unsupported protocol version {0}")]
    BadVersion(u32),
    #[error("token expired")]
    Expired,
    #[error("token not yet valid")]
    NotYetValid,
    #[error("insufficient trust anchor")]
    InsufficientAnchor,
    #[error("lease not granted")]
    LeaseNotGranted,
    #[error("invalid signature")]
    BadSignature,
    #[error("encoding error: {0}")]
    Encoding(String),
}

impl TokenHeader {
    pub fn new(typ: TokenType, alg: Algorithm, anchor: TrustAnchor) -> Self {
        Self {
            ver: PROTOCOL_VERSION,
            typ,
            alg,
            anchor,
        }
    }
}

impl TokenPayload {
    pub fn is_valid_at(&self, now_ms: u64) -> Result<(), TokenError> {
        if now_ms < self.nbf {
            return Err(TokenError::NotYetValid);
        }
        if now_ms >= self.exp {
            return Err(TokenError::Expired);
        }
        Ok(())
    }
}

impl WireToken {
    pub fn signing_bytes(&self) -> Result<Vec<u8>, TokenError> {
        crate::cbor::encode_signed_body(&self.header, &self.payload)
            .map_err(|e| TokenError::Encoding(e.to_string()))
    }

    pub fn validate_anchor_for_scope(&self) -> Result<(), TokenError> {
        match &self.payload.scope {
            ResourceScope::File(_) | ResourceScope::Network(_) => {
                if (self.header.anchor as u32) < (TrustAnchor::UiEvent as u32) {
                    return Err(TokenError::InsufficientAnchor);
                }
            }
            ResourceScope::Raw(_) => {}
        }
        if self.header.typ == TokenType::Lease
            && self.payload.state != LeaseState::Granted
        {
            return Err(TokenError::LeaseNotGranted);
        }
        Ok(())
    }
}
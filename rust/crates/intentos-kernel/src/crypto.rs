//! Development-oriented signing (Ed25519 + SHA3-384).
//! Not production post-quantum cryptography.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha3::{Digest, Sha3_384};

#[derive(Clone)]
pub struct DevKeyPair {
    signing: SigningKey,
}

impl DevKeyPair {
    pub fn generate() -> Self {
        Self {
            signing: SigningKey::generate(&mut OsRng),
        }
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            signing: SigningKey::from_bytes(&seed),
        }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing.verifying_key().to_bytes()
    }

    pub fn issuer_id(&self) -> Vec<u8> {
        hash384(&self.public_key_bytes()).to_vec()
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing.sign(message).to_bytes().to_vec()
    }

    pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        let Ok(pk): Result<[u8; 32], _> = public_key.try_into() else {
            return false;
        };
        let Ok(sig): Result<[u8; 64], _> = signature.try_into() else {
            return false;
        };
        VerifyingKey::from_bytes(&pk)
            .and_then(|vk| vk.verify(message, &ed25519_dalek::Signature::from_bytes(&sig)))
            .is_ok()
    }
}

pub fn hash384(data: &[u8]) -> [u8; 48] {
    let mut h = Sha3_384::new();
    h.update(data);
    h.finalize().into()
}

pub fn context_hash(action: &str) -> [u8; 48] {
    hash384(action.as_bytes())
}
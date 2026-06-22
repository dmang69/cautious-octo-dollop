use anyhow::{bail, Result};
use intentkernel_util::time::now_ms;
use uuid::Uuid;

use crate::cbor::{decode_wire_token, encode_wire_token};
use crate::hash::context_hash;
use crate::token::{
    Algorithm, LeaseState, ResourceScope, TokenError, TokenHeader, TokenPayload, TokenType,
    TrustAnchor, WireToken, PROTOCOL_VERSION,
};

#[derive(Clone)]
pub struct KeyPair {
    pub algorithm: Algorithm,
    pub public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

pub trait Signer {
    fn algorithm(&self) -> Algorithm;
    fn public_key(&self) -> &[u8];
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>>;
}

pub trait Verifier {
    fn algorithm(&self) -> Algorithm;
    fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool>;
}

impl KeyPair {
    pub fn generate(algorithm: Algorithm) -> Result<Self> {
        match algorithm {
            Algorithm::Ed25519 => generate_ed25519(),
            Algorithm::MlDsa87 => generate_mldsa87(),
            Algorithm::MlDsa65 => bail!("ML-DSA-65 not yet implemented"),
        }
    }

    pub fn from_secret(algorithm: Algorithm, secret_key: Vec<u8>) -> Result<Self> {
        match algorithm {
            Algorithm::Ed25519 => ed25519_from_secret(secret_key),
            Algorithm::MlDsa87 => mldsa87_from_secret(secret_key),
            Algorithm::MlDsa65 => bail!("ML-DSA-65 not yet implemented"),
        }
    }

    pub fn issuer_id(&self) -> Vec<u8> {
        crate::hash::sha384(self.public_key()).to_vec()
    }

    pub fn secret_key_bytes(&self) -> &[u8] {
        &self.secret_key
    }
}

impl Signer for KeyPair {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        match self.algorithm {
            Algorithm::Ed25519 => sign_ed25519(&self.secret_key, message),
            Algorithm::MlDsa87 => sign_mldsa87(&self.secret_key, message),
            Algorithm::MlDsa65 => bail!("ML-DSA-65 not yet implemented"),
        }
    }
}

#[derive(Clone)]
pub struct PublicKey {
    pub algorithm: Algorithm,
    pub bytes: Vec<u8>,
}

impl Verifier for PublicKey {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        match self.algorithm {
            Algorithm::Ed25519 => verify_ed25519(&self.bytes, message, signature),
            Algorithm::MlDsa87 => verify_mldsa87(&self.bytes, message, signature),
            Algorithm::MlDsa65 => bail!("ML-DSA-65 not yet implemented"),
        }
    }
}

pub struct TokenIssuer {
    pub keypair: KeyPair,
}

impl TokenIssuer {
    pub fn new(keypair: KeyPair) -> Self {
        Self { keypair }
    }

    pub fn issue_capability(
        &self,
        subject: &[u8],
        action: &[u8],
        scope: ResourceScope,
        anchor: TrustAnchor,
        ttl_ms: u64,
        max_uses: u32,
    ) -> Result<WireToken> {
        let now = now_ms();
        let header = TokenHeader::new(TokenType::Capability, self.keypair.algorithm(), anchor);
        let payload = TokenPayload {
            iss: self.keypair.issuer_id(),
            sub: subject.to_vec(),
            ctx: context_hash(action).to_vec(),
            scope,
            exp: now + ttl_ms,
            nbf: now,
            uses: max_uses,
            state: LeaseState::Granted,
            jti: Uuid::new_v4().as_bytes().to_vec(),
        };
        self.sign_token(header, payload)
    }

    pub fn sign_token(&self, header: TokenHeader, payload: TokenPayload) -> Result<WireToken> {
        if header.ver != PROTOCOL_VERSION {
            bail!("unsupported version");
        }
        let mut token = WireToken {
            header,
            payload,
            signature: Vec::new(),
        };
        let body = token.signing_bytes().map_err(|e| anyhow::anyhow!("{e}"))?;
        token.signature = self.keypair.sign(&body)?;
        Ok(token)
    }
}

pub struct TokenValidator {
    pub verifier: PublicKey,
}

impl TokenValidator {
    pub fn new(verifier: PublicKey) -> Self {
        Self { verifier }
    }

    pub fn validate(&self, token: &WireToken) -> Result<(), TokenError> {
        if token.header.ver != PROTOCOL_VERSION {
            return Err(TokenError::BadVersion(token.header.ver));
        }
        if token.header.alg != self.verifier.algorithm() {
            return Err(TokenError::BadSignature);
        }
        let body = token.signing_bytes()?;
        if !self
            .verifier
            .verify(&body, &token.signature)
            .map_err(|_| TokenError::BadSignature)?
        {
            return Err(TokenError::BadSignature);
        }
        token.payload.is_valid_at(now_ms())?;
        token.validate_anchor_for_scope()?;
        Ok(())
    }

    pub fn validate_bytes(&self, bytes: &[u8]) -> Result<WireToken, TokenError> {
        let token = decode_wire_token(bytes).map_err(|e| TokenError::Encoding(e.to_string()))?;
        self.validate(&token)?;
        Ok(token)
    }
}

pub fn token_to_bytes(token: &WireToken) -> Result<Vec<u8>> {
    encode_wire_token(token)
}

// --- Ed25519 (debug / dev) ---

#[cfg(feature = "ed25519")]
fn generate_ed25519() -> Result<KeyPair> {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    let signing = SigningKey::generate(&mut OsRng);
    Ok(KeyPair {
        algorithm: Algorithm::Ed25519,
        public_key: signing.verifying_key().to_bytes().to_vec(),
        secret_key: signing.to_bytes().to_vec(),
    })
}

#[cfg(not(feature = "ed25519"))]
fn generate_ed25519() -> Result<KeyPair> {
    bail!("ed25519 feature disabled")
}

#[cfg(feature = "ed25519")]
fn ed25519_from_secret(secret_key: Vec<u8>) -> Result<KeyPair> {
    use ed25519_dalek::SigningKey;
    let arr: [u8; 32] = secret_key
        .try_into()
        .map_err(|_| anyhow::anyhow!("ed25519 secret must be 32 bytes"))?;
    let signing = SigningKey::from_bytes(&arr);
    Ok(KeyPair {
        algorithm: Algorithm::Ed25519,
        public_key: signing.verifying_key().to_bytes().to_vec(),
        secret_key: signing.to_bytes().to_vec(),
    })
}

#[cfg(not(feature = "ed25519"))]
fn ed25519_from_secret(_: Vec<u8>) -> Result<KeyPair> {
    bail!("ed25519 feature disabled")
}

#[cfg(feature = "ed25519")]
fn sign_ed25519(secret: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    use ed25519_dalek::{Signer as _, SigningKey};
    let arr: [u8; 32] = secret
        .try_into()
        .map_err(|_| anyhow::anyhow!("bad ed25519 secret"))?;
    let key = SigningKey::from_bytes(&arr);
    Ok(key.sign(message).to_bytes().to_vec())
}

#[cfg(not(feature = "ed25519"))]
fn sign_ed25519(_: &[u8], _: &[u8]) -> Result<Vec<u8>> {
    bail!("ed25519 feature disabled")
}

#[cfg(feature = "ed25519")]
fn verify_ed25519(public: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    use ed25519_dalek::{Signature, Verifier as _, VerifyingKey};
    let pk: [u8; 32] = public
        .try_into()
        .map_err(|_| anyhow::anyhow!("bad ed25519 public key"))?;
    let sig: [u8; 64] = signature
        .try_into()
        .map_err(|_| anyhow::anyhow!("bad ed25519 signature"))?;
    let vk = VerifyingKey::from_bytes(&pk)?;
    Ok(vk.verify(message, &Signature::from_bytes(&sig)).is_ok())
}

#[cfg(not(feature = "ed25519"))]
fn verify_ed25519(_: &[u8], _: &[u8], _: &[u8]) -> Result<bool> {
    bail!("ed25519 feature disabled")
}

// --- ML-DSA-87 (production PQC) ---

#[cfg(feature = "pqc")]
fn generate_mldsa87() -> Result<KeyPair> {
    use pqcrypto_mldsa::mldsa87;
    use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _};
    let (pk, sk) = mldsa87::keypair();
    Ok(KeyPair {
        algorithm: Algorithm::MlDsa87,
        public_key: pk.as_bytes().to_vec(),
        secret_key: sk.as_bytes().to_vec(),
    })
}

#[cfg(not(feature = "pqc"))]
fn generate_mldsa87() -> Result<KeyPair> {
    bail!("enable the `pqc` feature for ML-DSA-87")
}

#[cfg(feature = "pqc")]
fn mldsa87_from_secret(secret_key: Vec<u8>) -> Result<KeyPair> {
    use pqcrypto_mldsa::mldsa87;
    use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _};
    let sk = mldsa87::SecretKey::from_bytes(&secret_key)
        .map_err(|_| anyhow::anyhow!("invalid ML-DSA-87 secret key"))?;
    let pk = mldsa87::PublicKey::from_secret_key(&sk);
    Ok(KeyPair {
        algorithm: Algorithm::MlDsa87,
        public_key: pk.as_bytes().to_vec(),
        secret_key: sk.as_bytes().to_vec(),
    })
}

#[cfg(not(feature = "pqc"))]
fn mldsa87_from_secret(_: Vec<u8>) -> Result<KeyPair> {
    bail!("enable the `pqc` feature for ML-DSA-87")
}

#[cfg(feature = "pqc")]
fn sign_mldsa87(secret: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    use pqcrypto_mldsa::mldsa87;
    use pqcrypto_traits::sign::{DetachedSignature as _, SecretKey as _};
    let sk = mldsa87::SecretKey::from_bytes(secret)
        .map_err(|_| anyhow::anyhow!("invalid ML-DSA-87 secret key"))?;
    let sig = mldsa87::detached_sign(message, &sk);
    Ok(sig.as_bytes().to_vec())
}

#[cfg(not(feature = "pqc"))]
fn sign_mldsa87(_: &[u8], _: &[u8]) -> Result<Vec<u8>> {
    bail!("enable the `pqc` feature for ML-DSA-87")
}

#[cfg(feature = "pqc")]
fn verify_mldsa87(public: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    use pqcrypto_mldsa::mldsa87;
    use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _};
    let pk = mldsa87::PublicKey::from_bytes(public)
        .map_err(|_| anyhow::anyhow!("invalid ML-DSA-87 public key"))?;
    let sig = mldsa87::DetachedSignature::from_bytes(signature)
        .map_err(|_| anyhow::anyhow!("invalid ML-DSA-87 signature"))?;
    Ok(mldsa87::verify_detached_signature(&sig, message, &pk).is_ok())
}

#[cfg(not(feature = "pqc"))]
fn verify_mldsa87(_: &[u8], _: &[u8], _: &[u8]) -> Result<bool> {
    bail!("enable the `pqc` feature for ML-DSA-87")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::FileScope;

    #[test]
    fn round_trip_ed25519_token() {
        let kp = KeyPair::generate(Algorithm::Ed25519).unwrap();
        let issuer = TokenIssuer::new(kp.clone());
        let token = issuer
            .issue_capability(
                b"app-hash",
                b"open_file",
                ResourceScope::File(FileScope {
                    path: "/data/secret.txt".into(),
                    access: 1,
                    inode: Some(42),
                }),
                TrustAnchor::UiEvent,
                60_000,
                1,
            )
            .unwrap();
        let bytes = token_to_bytes(&token).unwrap();
        let validator = TokenValidator::new(PublicKey {
            algorithm: Algorithm::Ed25519,
            bytes: kp.public_key.clone(),
        });
        let decoded = validator.validate_bytes(&bytes).unwrap();
        assert_eq!(decoded.payload.uses, 1);
    }
}
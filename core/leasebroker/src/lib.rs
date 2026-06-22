mod broker_key;

use std::collections::HashMap;

use anyhow::{bail, Result};
use intentkernel_crypto::sign::TokenIssuer;
use intentkernel_crypto::token::{LeaseState, TokenType, WireToken};
use intentkernel_util::time::now_ms;
use serde::Serialize;

pub use broker_key::{broker_key_path, load_broker_key};

pub const LEASEBROKER_HTTP_ADDR: &str = "127.0.0.1:8781";

/// RFC-INTENT-001 lease state machine threshold: renew at 80% TTL elapsed.
pub const RENEW_THRESHOLD_PERCENT: u64 = 80;

/// Tracked lease metadata per RFC-INTENT-001.
#[derive(Debug, Clone, Serialize)]
pub struct LeaseRecord {
    pub jti: Vec<u8>,
    pub subject: Vec<u8>,
    pub exp: u64,
    pub nbf: u64,
    pub state: LeaseState,
    pub renewal_count: u32,
    pub heartbeat_pending: bool,
    #[serde(skip)]
    token: WireToken,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeaseSummary {
    pub jti: String,
    pub subject: String,
    pub exp: u64,
    pub nbf: u64,
    pub state: String,
    pub renewal_count: u32,
    pub heartbeat_pending: bool,
}

impl From<&LeaseRecord> for LeaseSummary {
    fn from(record: &LeaseRecord) -> Self {
        Self {
            jti: hex::encode(&record.jti),
            subject: String::from_utf8_lossy(&record.subject).into_owned(),
            exp: record.exp,
            nbf: record.nbf,
            state: lease_state_name(record.state).into(),
            renewal_count: record.renewal_count,
            heartbeat_pending: record.heartbeat_pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TickAction {
    EnteredRenewing { jti: Vec<u8> },
    Expired { jti: Vec<u8> },
}

/// In-memory registry of active leases keyed by hex(jti).
#[derive(Debug, Default)]
pub struct LeaseRegistry {
    leases: HashMap<String, LeaseRecord>,
}

impl LeaseRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, record: LeaseRecord) {
        let key = hex::encode(&record.jti);
        self.leases.insert(key, record);
    }

    pub fn get(&self, jti: &[u8]) -> Option<&LeaseRecord> {
        self.leases.get(&hex::encode(jti))
    }

    pub fn get_mut(&mut self, jti: &[u8]) -> Option<&mut LeaseRecord> {
        self.leases.get_mut(&hex::encode(jti))
    }

    pub fn list(&self) -> Vec<LeaseSummary> {
        let mut out: Vec<_> = self.leases.values().map(LeaseSummary::from).collect();
        out.sort_by(|a, b| a.jti.cmp(&b.jti));
        out
    }

    pub fn len(&self) -> usize {
        self.leases.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leases.is_empty()
    }
}

/// Background lease renewal broker per RFC-INTENT-001.
pub struct LeaseBroker {
    pub registry: LeaseRegistry,
    issuer: TokenIssuer,
}

impl LeaseBroker {
    pub fn new(issuer: TokenIssuer) -> Self {
        Self {
            registry: LeaseRegistry::new(),
            issuer,
        }
    }

    /// Register a lease token. Only `typ=LEASE` with `state=GRANTED` is accepted.
    pub fn register_lease(&mut self, token: WireToken) -> Result<()> {
        if token.header.typ != TokenType::Lease {
            bail!("register_lease: token type must be LEASE");
        }
        if token.payload.state != LeaseState::Granted {
            bail!("register_lease: lease state must be GRANTED");
        }

        let record = LeaseRecord {
            jti: token.payload.jti.clone(),
            subject: token.payload.sub.clone(),
            exp: token.payload.exp,
            nbf: token.payload.nbf,
            state: LeaseState::Granted,
            renewal_count: 0,
            heartbeat_pending: false,
            token,
        };
        self.registry.insert(record);
        Ok(())
    }

    /// Scan all leases and apply RFC state transitions.
    ///
    /// - At 80% TTL elapsed: `GRANTED` → `RENEWING`, heartbeat pending
    /// - At expiry (`now >= exp`): → `EXPIRED`, halt execution
    pub fn tick_at(&mut self, now: u64) -> Vec<TickAction> {
        let mut actions = Vec::new();
        for record in self.registry.leases.values_mut() {
            if matches!(
                record.state,
                LeaseState::Expired | LeaseState::Revoked | LeaseState::Suspended
            ) {
                continue;
            }

            if now >= record.exp {
                record.state = LeaseState::Expired;
                record.heartbeat_pending = false;
                record.token.payload.state = LeaseState::Expired;
                actions.push(TickAction::Expired {
                    jti: record.jti.clone(),
                });
                continue;
            }

            if record.state == LeaseState::Granted && should_enter_renewing(record, now) {
                record.state = LeaseState::Renewing;
                record.heartbeat_pending = true;
                record.token.payload.state = LeaseState::Renewing;
                actions.push(TickAction::EnteredRenewing {
                    jti: record.jti.clone(),
                });
            }
        }
        actions
    }

    pub fn tick(&mut self) -> Vec<TickAction> {
        self.tick_at(now_ms())
    }

    /// Re-sign the lease token with extended `exp`, transition back to `GRANTED`.
    pub fn renew_at(&mut self, jti: &[u8], now: u64) -> Result<WireToken> {
        let record = self
            .registry
            .get_mut(jti)
            .ok_or_else(|| anyhow::anyhow!("lease not found: {}", hex::encode(jti)))?;

        match record.state {
            LeaseState::Revoked => bail!("cannot renew revoked lease"),
            LeaseState::Expired => bail!("cannot renew expired lease"),
            LeaseState::Suspended => bail!("cannot renew suspended lease"),
            _ => {}
        }

        let ttl = record.exp.saturating_sub(record.nbf);
        if ttl == 0 {
            bail!("lease has zero TTL");
        }

        let header = record.token.header.clone();
        let mut payload = record.token.payload.clone();
        payload.nbf = now;
        payload.exp = now.saturating_add(ttl);
        payload.state = LeaseState::Granted;

        let new_token = self.issuer.sign_token(header, payload)?;

        record.exp = new_token.payload.exp;
        record.nbf = new_token.payload.nbf;
        record.state = LeaseState::Granted;
        record.renewal_count = record.renewal_count.saturating_add(1);
        record.heartbeat_pending = false;
        record.token = new_token.clone();

        Ok(new_token)
    }

    pub fn renew(&mut self, jti: &[u8]) -> Result<WireToken> {
        self.renew_at(jti, now_ms())
    }

    pub fn revoke(&mut self, jti: &[u8]) -> Result<()> {
        let record = self
            .registry
            .get_mut(jti)
            .ok_or_else(|| anyhow::anyhow!("lease not found: {}", hex::encode(jti)))?;
        record.state = LeaseState::Revoked;
        record.heartbeat_pending = false;
        record.token.payload.state = LeaseState::Revoked;
        Ok(())
    }

    pub fn list(&self) -> Vec<LeaseSummary> {
        self.registry.list()
    }
}

fn should_enter_renewing(record: &LeaseRecord, now: u64) -> bool {
    let ttl = record.exp.saturating_sub(record.nbf);
    if ttl == 0 {
        return false;
    }
    let elapsed = now.saturating_sub(record.nbf);
    elapsed.saturating_mul(100) >= ttl.saturating_mul(RENEW_THRESHOLD_PERCENT)
}

pub fn lease_state_name(state: LeaseState) -> &'static str {
    match state {
        LeaseState::Requested => "REQUESTED",
        LeaseState::Granted => "GRANTED",
        LeaseState::Renewing => "RENEWING",
        LeaseState::Expired => "EXPIRED",
        LeaseState::Revoked => "REVOKED",
        LeaseState::Suspended => "SUSPENDED",
    }
}

pub fn parse_jti_hex(jti_hex: &str) -> Result<Vec<u8>> {
    hex::decode(jti_hex).map_err(|e| anyhow::anyhow!("invalid jti hex: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use intentkernel_crypto::hash::context_hash;
    use intentkernel_crypto::sign::{KeyPair, Signer, TokenIssuer};
    use intentkernel_crypto::token::{
        Algorithm, FileScope, LeaseState, ResourceScope, TokenHeader, TokenPayload, TokenType,
        TrustAnchor,
    };
    fn test_issuer() -> TokenIssuer {
        let kp = KeyPair::generate(Algorithm::Ed25519).unwrap();
        TokenIssuer::new(kp)
    }

    fn make_lease_token(
        issuer: &TokenIssuer,
        nbf: u64,
        exp: u64,
        state: LeaseState,
        jti: &[u8],
    ) -> WireToken {
        let header = TokenHeader::new(
            TokenType::Lease,
            issuer.keypair.algorithm(),
            TrustAnchor::UiEvent,
        );
        let payload = TokenPayload {
            iss: issuer.keypair.issuer_id(),
            sub: b"agent-42".to_vec(),
            ctx: context_hash(b"lease_resource").to_vec(),
            scope: ResourceScope::File(FileScope {
                path: "/data/lease.txt".into(),
                access: 1,
                inode: None,
            }),
            exp,
            nbf,
            uses: 1,
            state,
            jti: jti.to_vec(),
        };
        issuer.sign_token(header, payload).unwrap()
    }

    const TEST_JTI: &[u8] = b"test-lease-jti-001";

    #[test]
    fn register_rejects_non_lease() {
        let issuer = test_issuer();
        let mut broker = LeaseBroker::new(test_issuer());
        let token = issuer
            .issue_capability(
                b"sub",
                b"act",
                ResourceScope::File(FileScope {
                    path: "/x".into(),
                    access: 1,
                    inode: None,
                }),
                TrustAnchor::UiEvent,
                60_000,
                1,
            )
            .unwrap();
        assert!(broker.register_lease(token).is_err());
    }

    #[test]
    fn register_rejects_non_granted() {
        let issuer = test_issuer();
        let mut broker = LeaseBroker::new(test_issuer());
        let token = make_lease_token(&issuer, 0, 1_000, LeaseState::Requested, TEST_JTI);
        assert!(broker.register_lease(token).is_err());
    }

    #[test]
    fn tick_transitions_to_renewing_at_80_percent_ttl() {
        let issuer = test_issuer();
        let mut broker = LeaseBroker::new(test_issuer());
        let token = make_lease_token(&issuer, 0, 1_000, LeaseState::Granted, TEST_JTI);
        let jti = token.payload.jti.clone();
        broker.register_lease(token).unwrap();

        let actions = broker.tick_at(799);
        assert!(actions.is_empty());
        let record = broker.registry.get(&jti).unwrap();
        assert_eq!(record.state, LeaseState::Granted);
        assert!(!record.heartbeat_pending);

        let actions = broker.tick_at(800);
        assert_eq!(
            actions,
            vec![TickAction::EnteredRenewing { jti: jti.clone() }]
        );
        let record = broker.registry.get(&jti).unwrap();
        assert_eq!(record.state, LeaseState::Renewing);
        assert!(record.heartbeat_pending);
    }

    #[test]
    fn tick_transitions_to_expired_at_zero_ttl() {
        let issuer = test_issuer();
        let mut broker = LeaseBroker::new(test_issuer());
        let token = make_lease_token(&issuer, 0, 1_000, LeaseState::Granted, TEST_JTI);
        let jti = token.payload.jti.clone();
        broker.register_lease(token).unwrap();

        let actions = broker.tick_at(790);
        assert!(actions.is_empty());
        assert_eq!(
            broker.registry.get(&jti).unwrap().state,
            LeaseState::Granted
        );

        let actions = broker.tick_at(1_000);
        assert_eq!(actions, vec![TickAction::Expired { jti: jti.clone() }]);
        let record = broker.registry.get(&jti).unwrap();
        assert_eq!(record.state, LeaseState::Expired);
        assert!(!record.heartbeat_pending);
    }

    #[test]
    fn renew_extends_exp_and_returns_to_granted() {
        let kp = KeyPair::generate(Algorithm::Ed25519).unwrap();
        let issuer = TokenIssuer::new(kp.clone());
        let mut broker = LeaseBroker::new(TokenIssuer::new(kp));
        let token = make_lease_token(&issuer, 0, 1_000, LeaseState::Granted, TEST_JTI);
        let jti = token.payload.jti.clone();
        broker.register_lease(token).unwrap();
        broker.tick_at(850);

        let renewed = broker.renew_at(&jti, 900).unwrap();
        let record = broker.registry.get(&jti).unwrap();

        assert_eq!(record.state, LeaseState::Granted);
        assert_eq!(record.renewal_count, 1);
        assert!(!record.heartbeat_pending);
        assert_eq!(record.nbf, 900);
        assert_eq!(record.exp, 1_900);
        assert_eq!(renewed.payload.exp, 1_900);
        assert_eq!(renewed.payload.state, LeaseState::Granted);
    }

    #[test]
    fn revoke_marks_lease_revoked() {
        let issuer = test_issuer();
        let mut broker = LeaseBroker::new(test_issuer());
        let token = make_lease_token(&issuer, 0, 1_000, LeaseState::Granted, TEST_JTI);
        let jti = token.payload.jti.clone();
        broker.register_lease(token).unwrap();

        broker.revoke(&jti).unwrap();
        let record = broker.registry.get(&jti).unwrap();
        assert_eq!(record.state, LeaseState::Revoked);
    }
}
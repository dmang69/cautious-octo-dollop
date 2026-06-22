use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustAnchor {
    None = 0,
    UiEvent = 1,
    Biometric = 2,
    Hardware = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub action: String,
    pub resource: String,
    pub anchor: TrustAnchor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub ttl_ms: u64,
    pub max_uses: u32,
    pub resource_type: u32,
    pub reason: String,
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("intent denied: {0}")]
    Denied(String),
}

pub fn evaluate(intent: &Intent) -> Result<PolicyDecision, PolicyError> {
    if intent.action.is_empty() || intent.resource.is_empty() {
        return Err(PolicyError::Denied("empty action or resource".into()));
    }

    let needs_ui = intent.action.starts_with("vfs:")
        || intent.action.starts_with("ai:")
        || intent.action.starts_with("net:");

    if needs_ui && (intent.anchor as u32) < TrustAnchor::UiEvent as u32 {
        return Err(PolicyError::Denied(
            "file, ai, and network intents require UI_EVENT anchor".into(),
        ));
    }

    let (resource_type, ttl_ms, max_uses) = if intent.action.starts_with("vfs:read") {
        (1, 60_000, 1)
    } else if intent.action.starts_with("vfs:write") {
        (1, 60_000, 1)
    } else if intent.action.starts_with("vfs:list") {
        (1, 30_000, 1)
    } else if intent.action.starts_with("ai:") {
        (2, 120_000, 1)
    } else if intent.action.starts_with("lease:") {
        (3, 300_000, 0)
    } else {
        (0, 30_000, 1)
    };

    Ok(PolicyDecision {
        allowed: true,
        ttl_ms,
        max_uses,
        resource_type,
        reason: format!("approved {} on {}", intent.action, intent.resource),
    })
}
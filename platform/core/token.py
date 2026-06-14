"""
IntentKernel Capability Token — RFC-INTENT-001 v0

Token format (JSON-serializable):
  {
    "id":             <uuid4 string>,
    "scope":          <string>,       # e.g. "network_request", "file_read"
    "subject":        <string>,       # caller identity (process name / app id)
    "target":         <string|null>,  # optional: specific target (e.g. "93.184.216.34:80")
    "issued_at":      <float>,        # Unix epoch seconds
    "expires_at":     <float>,        # Unix epoch seconds
    "uses_remaining": <int>,          # 1 = one-shot (default)
    "sig":            <hex string>    # HMAC-SHA256 over canonical fields (see below)
  }

Canonical signing surface (pipe-delimited, no whitespace):
  id|scope|subject|target|issued_at|expires_at|uses_remaining

NOTE: Production tokens should use ML-DSA-87 (FIPS 204 / liboqs).
      HMAC-SHA256 is used here as a compatible placeholder until the
      liboqs Python bindings are integrated.  The token structure and
      API are intentionally forward-compatible with PQC signing.

Key storage:
  The broker signing key lives at ~/.intentos/broker.key (32 raw bytes).
  intentd creates it on first run; capd and ip-descramblerd read it.
  In production this should be hardware-backed (TPM / HSM).
"""

import hashlib
import hmac
import json
import os
import secrets
import time
import uuid
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Key management
# ---------------------------------------------------------------------------

_KEY_ENV = "INTENTOS_BROKER_KEY_FILE"
_KEY_FILE = Path(os.environ.get(_KEY_ENV, Path.home() / ".intentos" / "broker.key"))

_KEY_CACHE: Optional[bytes] = None


def load_or_create_key(path: Optional[Path] = None) -> bytes:
    """Load the broker signing key from disk, generating it on first use."""
    key_path = path or _KEY_FILE
    key_path.parent.mkdir(parents=True, exist_ok=True)

    if key_path.exists():
        raw = key_path.read_bytes()
        if len(raw) == 32:
            return raw
        # File is corrupt — regenerate
        key_path.unlink(missing_ok=True)

    key = secrets.token_bytes(32)
    key_path.write_bytes(key)
    try:
        os.chmod(key_path, 0o600)  # owner read/write only
    except OSError:
        pass  # Windows — skip chmod
    return key


def get_key() -> bytes:
    """Return the cached broker key, loading from disk if necessary."""
    global _KEY_CACHE
    if _KEY_CACHE is None:
        _KEY_CACHE = load_or_create_key()
    return _KEY_CACHE


# ---------------------------------------------------------------------------
# Token dataclass
# ---------------------------------------------------------------------------

DEFAULT_TTL: int = 60    # seconds
DEFAULT_USES: int = 1    # one-shot


@dataclass
class CapabilityToken:
    """An IntentKernel capability token."""

    id: str
    scope: str
    subject: str
    target: Optional[str]
    issued_at: float
    expires_at: float
    uses_remaining: int
    sig: str = ""

    # ------------------------------------------------------------------
    # Signing / verification
    # ------------------------------------------------------------------

    def _canonical(self) -> bytes:
        """Deterministic byte string over the fields that are signed."""
        parts = [
            self.id,
            self.scope,
            self.subject,
            self.target or "",
            f"{self.issued_at:.6f}",
            f"{self.expires_at:.6f}",
            str(self.uses_remaining),
        ]
        return "|".join(parts).encode()

    def sign(self, key: bytes) -> "CapabilityToken":
        """Compute and attach HMAC-SHA256 signature."""
        mac = hmac.new(key, self._canonical(), hashlib.sha256)
        self.sig = mac.hexdigest()
        return self

    def verify(self, key: bytes) -> bool:
        """Return True iff the signature is valid for the given key."""
        if not self.sig:
            return False
        mac = hmac.new(key, self._canonical(), hashlib.sha256)
        return hmac.compare_digest(mac.hexdigest(), self.sig)

    # ------------------------------------------------------------------
    # Expiry / validity helpers
    # ------------------------------------------------------------------

    def is_expired(self) -> bool:
        return time.time() > self.expires_at

    def is_valid(self, key: bytes) -> tuple[bool, str]:
        """Return (ok, reason) checking signature and expiry."""
        if not self.verify(key):
            return False, "invalid signature"
        if self.is_expired():
            return False, "token expired"
        if self.uses_remaining <= 0:
            return False, "no uses remaining"
        return True, "ok"

    # ------------------------------------------------------------------
    # Serialization
    # ------------------------------------------------------------------

    def to_dict(self) -> dict:
        return asdict(self)

    def to_json(self) -> str:
        return json.dumps(self.to_dict())

    @classmethod
    def from_dict(cls, d: dict) -> "CapabilityToken":
        return cls(
            id=d["id"],
            scope=d["scope"],
            subject=d["subject"],
            target=d.get("target"),
            issued_at=float(d["issued_at"]),
            expires_at=float(d["expires_at"]),
            uses_remaining=int(d["uses_remaining"]),
            sig=d.get("sig", ""),
        )

    @classmethod
    def from_json(cls, s: str) -> "CapabilityToken":
        return cls.from_dict(json.loads(s))


# ---------------------------------------------------------------------------
# Factory
# ---------------------------------------------------------------------------

def issue_token(
    scope: str,
    subject: str,
    target: Optional[str] = None,
    ttl: int = DEFAULT_TTL,
    uses: int = DEFAULT_USES,
    key: Optional[bytes] = None,
) -> CapabilityToken:
    """Issue a signed capability token."""
    now = time.time()
    token = CapabilityToken(
        id=str(uuid.uuid4()),
        scope=scope,
        subject=subject,
        target=target,
        issued_at=now,
        expires_at=now + ttl,
        uses_remaining=uses,
    )
    return token.sign(key or get_key())

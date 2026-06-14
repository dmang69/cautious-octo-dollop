"""
IntentKernel token lifecycle store.

capd uses this to track issued tokens, enforce one-shot semantics,
and support explicit revocation.  All state is in-memory; on restart
previously issued tokens are treated as unknown (signature-only
verification still applies).
"""

import threading
import time
from typing import Dict, Optional, Set

from .token import CapabilityToken, get_key


class TokenStore:
    """Thread-safe in-memory store for tracking active capability tokens."""

    def __init__(self) -> None:
        self._tokens: Dict[str, CapabilityToken] = {}
        self._revoked: Set[str] = set()
        self._lock = threading.Lock()

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def register(self, token: CapabilityToken) -> None:
        """Register a freshly-issued token so usage can be tracked."""
        with self._lock:
            self._tokens[token.id] = token

    def validate(
        self,
        token: CapabilityToken,
        key: Optional[bytes] = None,
    ) -> tuple[bool, str]:
        """
        Validate a token.  Returns (valid: bool, reason: str).

        Checks (in order):
        1. Cryptographic signature
        2. Expiry
        3. Revocation list
        4. Uses remaining (if registered; otherwise signature check only)
        """
        signing_key = key or get_key()
        ok, reason = token.is_valid(signing_key)
        if not ok:
            return False, reason

        with self._lock:
            if token.id in self._revoked:
                return False, "token revoked"

            stored = self._tokens.get(token.id)
            # Check the *store's* usage counter, not the incoming token's
            # self-reported field — a caller could forge uses_remaining in the
            # presented token even though the signature covers the original value.
            if stored is not None and stored.uses_remaining <= 0:
                return False, "no uses remaining"

        return True, "ok"

    def consume(self, token_id: str) -> bool:
        """
        Decrement uses_remaining for a registered token.

        Returns True if the consume succeeded (or if the token is not
        registered here, which means it passed signature-only validation
        upstream), False if uses are exhausted.
        """
        with self._lock:
            stored = self._tokens.get(token_id)
            if stored is None:
                return True  # not tracked; signature was already verified
            if stored.uses_remaining <= 0:
                return False
            stored.uses_remaining -= 1
            if stored.uses_remaining == 0:
                # Burn one-shot token immediately
                del self._tokens[token_id]
            return True

    def revoke(self, token_id: str) -> None:
        """Explicitly revoke a token by ID."""
        with self._lock:
            self._revoked.add(token_id)
            self._tokens.pop(token_id, None)

    # ------------------------------------------------------------------
    # Housekeeping
    # ------------------------------------------------------------------

    def gc(self) -> int:
        """Evict expired tokens.  Returns the number removed."""
        now = time.time()
        removed = 0
        with self._lock:
            expired = [tid for tid, tok in self._tokens.items() if tok.expires_at < now]
            for tid in expired:
                del self._tokens[tid]
                removed += 1
        return removed

    def stats(self) -> dict:
        with self._lock:
            return {
                "active": len(self._tokens),
                "revoked": len(self._revoked),
            }

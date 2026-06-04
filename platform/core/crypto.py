"""
IntentKernel crypto adapter (pqc simulation for MVP scaffolding).
"""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import hmac
import secrets
from typing import Any

from . import config as cfg


@dataclass
class Signature:
    algorithm: str
    key_id: str
    value: str


@dataclass
class CryptoContext:
    algorithm: str
    key_id: str
    secret_key: bytes


_context: CryptoContext | None = None


def _load_context() -> CryptoContext:
    crypto_cfg = cfg.get_section("crypto")
    algorithm = crypto_cfg.get("algorithm", "SIM-ML-DSA-87")
    key_id = crypto_cfg.get("key_id") or secrets.token_hex(6)
    secret_key = secrets.token_bytes(64)
    return CryptoContext(algorithm=algorithm, key_id=key_id, secret_key=secret_key)


def get_context() -> CryptoContext:
    global _context
    if _context is None:
        _context = _load_context()
    return _context


def sign(payload: bytes) -> Signature:
    ctx = get_context()
    digest = hmac.new(ctx.secret_key, payload, hashlib.sha3_512).hexdigest()
    return Signature(algorithm=ctx.algorithm, key_id=ctx.key_id, value=digest)


def verify(payload: bytes, signature: Signature | dict[str, Any]) -> bool:
    ctx = get_context()
    sig = signature
    if isinstance(signature, dict):
        sig = Signature(
            algorithm=signature.get("algorithm", ""),
            key_id=signature.get("key_id", ""),
            value=signature.get("value", ""),
        )
    if sig.algorithm != ctx.algorithm or sig.key_id != ctx.key_id:
        return False
    expected = hmac.new(ctx.secret_key, payload, hashlib.sha3_512).hexdigest()
    return hmac.compare_digest(expected, sig.value)

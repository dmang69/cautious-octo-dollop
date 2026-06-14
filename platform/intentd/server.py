"""
intentd — Intent Broker Flask service.

Listens on localhost:5001 by default.

Endpoints
---------
POST /issue
    Issue a capability token.
    Body: {"scope": str, "subject": str, "target"?: str, "ttl"?: int, "uses"?: int}
    Response 200: {"token": <CapabilityToken dict>, "ok": true}
    Response 403: {"ok": false, "error": str}

POST /revoke
    Revoke a token by ID.
    Body: {"token_id": str}
    Response 200: {"ok": true}

GET /status
    Health / statistics endpoint.
    Response 200: {"ok": true, "service": "intentd", "stats": {...}}
"""

import logging
import os
import sys
import time
from pathlib import Path

from flask import Blueprint, Flask, jsonify, request

# Make the platform/ directory importable when running as a sub-package
_PLATFORM_DIR = Path(__file__).resolve().parent.parent
if str(_PLATFORM_DIR) not in sys.path:
    sys.path.insert(0, str(_PLATFORM_DIR))

from core.token import DEFAULT_TTL, DEFAULT_USES, issue_token
from core.token_store import TokenStore
from intentd.policy import PolicyEngine

log = logging.getLogger("intentd")
_start_time = time.time()

# ---------------------------------------------------------------------------
# Shared state (module-level so the Flask app can reference it)
# ---------------------------------------------------------------------------

_store = TokenStore()
_policy = PolicyEngine()

# ---------------------------------------------------------------------------
# Flask application factory
# ---------------------------------------------------------------------------

intentd_bp = Blueprint("intentd", __name__)


def _err(msg: str, code: int = 400):
    return jsonify({"ok": False, "error": msg}), code


@intentd_bp.post("/issue")
def issue():
    data = request.get_json(force=True) or {}

    scope = data.get("scope", "").strip()
    subject = data.get("subject", "").strip()
    if not scope or not subject:
        return _err("'scope' and 'subject' are required")

    target = data.get("target") or None
    ttl = int(data.get("ttl", DEFAULT_TTL))
    uses = int(data.get("uses", DEFAULT_USES))

    if ttl < 1 or ttl > 86400:
        return _err("'ttl' must be between 1 and 86400 seconds")
    if uses < 1 or uses > 100:
        return _err("'uses' must be between 1 and 100")

    allowed, reason, _rule = _policy.check(scope, subject, ttl, uses)
    if not allowed:
        log.warning("Policy denied: scope=%s subject=%s reason=%s", scope, subject, reason)
        return jsonify({"ok": False, "error": reason}), 403

    token = issue_token(scope=scope, subject=subject, target=target, ttl=ttl, uses=uses)
    _store.register(token)

    log.info(
        "Token issued: id=%s scope=%s subject=%s target=%s ttl=%ds uses=%d",
        token.id, scope, subject, target, ttl, uses,
    )
    return jsonify({"ok": True, "token": token.to_dict()})


@intentd_bp.post("/revoke")
def revoke():
    data = request.get_json(force=True) or {}
    token_id = data.get("token_id", "").strip()
    if not token_id:
        return _err("'token_id' is required")

    _store.revoke(token_id)
    log.info("Token revoked: id=%s", token_id)
    return jsonify({"ok": True})


@intentd_bp.get("/status")
def status():
    return jsonify({
        "ok": True,
        "service": "intentd",
        "uptime_seconds": round(time.time() - _start_time, 1),
        "stats": _store.stats(),
    })


def create_app() -> Flask:
    app = Flask("intentd")
    app.register_blueprint(intentd_bp)
    return app

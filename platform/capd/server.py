"""
capd — Capability Verifier Flask service.

Listens on localhost:5002 by default.

Endpoints
---------
POST /validate
    Validate (and optionally consume) a capability token.
    Body: {"token": <CapabilityToken dict>, "consume"?: bool}
    Response 200: {"ok": true, "valid": true}
    Response 200: {"ok": true, "valid": false, "reason": str}

POST /revoke
    Revoke a token by ID.
    Body: {"token_id": str}
    Response 200: {"ok": true}

GET /status
    Health / statistics.
    Response 200: {"ok": true, "service": "capd", "stats": {...}}
"""

import logging
import sys
import threading
import time
from pathlib import Path

from flask import Blueprint, Flask, jsonify, request

_PLATFORM_DIR = Path(__file__).resolve().parent.parent
if str(_PLATFORM_DIR) not in sys.path:
    sys.path.insert(0, str(_PLATFORM_DIR))

from core.token import CapabilityToken, get_key
from core.token_store import TokenStore

log = logging.getLogger("capd")

_store = TokenStore()
_gc_interval = 120  # seconds between garbage-collection passes

# ---------------------------------------------------------------------------
# Background GC thread
# ---------------------------------------------------------------------------

def _gc_loop() -> None:
    while True:
        time.sleep(_gc_interval)
        removed = _store.gc()
        if removed:
            log.debug("GC: removed %d expired token(s)", removed)


_gc_thread = threading.Thread(target=_gc_loop, daemon=True, name="capd-gc")

# ---------------------------------------------------------------------------
# Flask application
# ---------------------------------------------------------------------------

capd_bp = Blueprint("capd", __name__)


def _err(msg: str, code: int = 400):
    return jsonify({"ok": False, "error": msg}), code


@capd_bp.post("/validate")
def validate():
    data = request.get_json(force=True) or {}

    token_data = data.get("token")
    if not token_data:
        return _err("'token' is required")

    try:
        token = CapabilityToken.from_dict(token_data)
    except (KeyError, TypeError, ValueError) as exc:
        return _err(f"malformed token: {exc}")

    consume = bool(data.get("consume", True))

    valid, reason = _store.validate(token, key=get_key())
    if not valid:
        log.warning("Token invalid: id=%s reason=%s", token.id, reason)
        return jsonify({"ok": True, "valid": False, "reason": reason})

    if consume:
        if not _store.consume(token.id):
            return jsonify({"ok": True, "valid": False, "reason": "no uses remaining"})

    log.info(
        "Token validated: id=%s scope=%s subject=%s consumed=%s",
        token.id, token.scope, token.subject, consume,
    )
    return jsonify({"ok": True, "valid": True, "token_id": token.id, "scope": token.scope})


@capd_bp.post("/revoke")
def revoke():
    data = request.get_json(force=True) or {}
    token_id = data.get("token_id", "").strip()
    if not token_id:
        return _err("'token_id' is required")
    _store.revoke(token_id)
    log.info("Token revoked: id=%s", token_id)
    return jsonify({"ok": True})


@capd_bp.get("/status")
def status():
    return jsonify({
        "ok": True,
        "service": "capd",
        "uptime": time.time(),
        "stats": _store.stats(),
    })


def create_app() -> Flask:
    app = Flask("capd")
    app.register_blueprint(capd_bp)
    if not _gc_thread.is_alive():
        _gc_thread.start()
    return app

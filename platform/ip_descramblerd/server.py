"""
ip-descramblerd — IP Descrambler Flask service.

Listens on localhost:5003 by default.

Capability model:
  Every call to /analyze must supply a valid capability token with
  scope "network_request" (or "network_inspect").  The token is
  validated by calling capd before the analysis is performed.

Endpoints
---------
POST /analyze
    Analyze an IP address.
    Body: {
      "ip":    str,
      "token": <CapabilityToken dict>   # scope must be network_request / network_inspect
    }
    Response 200: {
      "ok":      true,
      "ip":      str,
      "verdict": "allow" | "warn" | "block",
      "reason":  str
    }
    Response 403: {"ok": false, "error": "capability denied: <reason>"}

GET /status
    Health check.
    Response 200: {"ok": true, "service": "ip-descramblerd"}
"""

import logging
import os
import sys
import time
from pathlib import Path

import requests
from flask import Blueprint, Flask, jsonify, request

_PLATFORM_DIR = Path(__file__).resolve().parent.parent
if str(_PLATFORM_DIR) not in sys.path:
    sys.path.insert(0, str(_PLATFORM_DIR))

from core.token import CapabilityToken
from ip_descramblerd.analyzer import analyze_ip

log = logging.getLogger("ip-descramblerd")

# Where capd lives (overridable via env)
_CAPD_URL = os.environ.get("INTENTOS_CAPD_URL", "http://127.0.0.1:5002")
_ALLOWED_SCOPES = {"network_request", "network_inspect"}

ip_descramblerd_bp = Blueprint("ip_descramblerd", __name__)


def _err(msg: str, code: int = 400):
    return jsonify({"ok": False, "error": msg}), code


def _validate_token_with_capd(token_dict: dict) -> tuple[bool, str]:
    """
    Ask capd to validate (and consume) the presented token.
    Returns (valid: bool, reason: str).
    Falls back to local signature-only check if capd is unreachable.
    """
    from core.token import get_key  # local fallback

    try:
        resp = requests.post(
            f"{_CAPD_URL}/validate",
            json={"token": token_dict, "consume": True},
            timeout=2,
        )
        body = resp.json()
        if body.get("valid"):
            return True, "ok"
        return False, body.get("reason", "capd denied")
    except requests.RequestException as exc:
        log.warning("capd unreachable (%s) — falling back to local signature check", exc)
        try:
            token = CapabilityToken.from_dict(token_dict)
            ok, reason = token.is_valid(get_key())
            return ok, reason
        except Exception as e2:
            return False, f"local validation failed: {e2}"


@ip_descramblerd_bp.post("/analyze")
def analyze():
    data = request.get_json(force=True) or {}

    ip = (data.get("ip") or "").strip()
    if not ip:
        return _err("'ip' is required")

    token_data = data.get("token")
    if not token_data:
        return _err("'token' (CapabilityToken) is required")

    # Scope check before sending to capd
    scope = token_data.get("scope", "")
    if scope not in _ALLOWED_SCOPES:
        return jsonify({
            "ok": False,
            "error": f"token scope '{scope}' is not permitted for IP analysis",
        }), 403

    valid, reason = _validate_token_with_capd(token_data)
    if not valid:
        log.warning("Capability denied for IP analysis: ip=%s reason=%s", ip, reason)
        return jsonify({"ok": False, "error": f"capability denied: {reason}"}), 403

    result = analyze_ip(ip)
    return jsonify({"ok": True, **result})


@ip_descramblerd_bp.get("/status")
def status():
    return jsonify({
        "ok": True,
        "service": "ip-descramblerd",
        "uptime": time.time(),
        "capd_url": _CAPD_URL,
    })


def create_app() -> Flask:
    app = Flask("ip_descramblerd")
    app.register_blueprint(ip_descramblerd_bp)
    return app

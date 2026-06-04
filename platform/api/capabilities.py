"""
IntentOS Control Surface — capability broker API.
"""

from flask import Blueprint, jsonify, request

from core import broker, eventscope

capabilities_bp = Blueprint("capabilities", __name__)


def _json_error(msg: str, code: int = 400):
    return jsonify({"error": msg}), code


@capabilities_bp.post("/capabilities/request")
def request_capability():
    data = request.get_json(force=True) or {}
    cap_type = data.get("cap_type")
    intent = data.get("intent") or {}
    if not cap_type:
        return _json_error("cap_type is required")
    token = broker.request_capability(
        intent=intent,
        cap_type=cap_type,
        ttl_ms=data.get("ttl_ms"),
        uses=data.get("uses"),
        metadata=data.get("metadata"),
        publish=bool(data.get("publish", True)),
    )
    return jsonify(token.to_dict())


@capabilities_bp.get("/capabilities/next")
def next_capability():
    timeout = request.args.get("timeout_s")
    token = broker.await_capability(timeout_s=float(timeout) if timeout else None)
    if not token:
        return jsonify({"token": None})
    return jsonify({"token": token.to_dict()})


@capabilities_bp.post("/capabilities/invoke")
def invoke_capability():
    data = request.get_json(force=True) or {}
    token = data.get("token")
    action = data.get("action")
    if not token or not action:
        return _json_error("token and action are required")
    resource = data.get("resource")
    result = eventscope.authorize(token, action, resource=resource)
    return jsonify({
        "allowed": result.allowed,
        "reason": result.reason,
        "cap_type": result.cap_type,
    })

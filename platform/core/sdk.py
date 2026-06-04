"""
IntentKernel SDK (9 primitive APIs) — MVP reference implementation.
"""

from __future__ import annotations

from typing import Any

from . import broker, eventscope, logger as log


def draw(framebuffer: bytes, token: dict[str, Any]) -> dict[str, Any]:
    _require(token, "draw", {"bytes": len(framebuffer)})
    return {"status": "rendered", "bytes": len(framebuffer)}


def wait_event(timeout_s: float | None = None) -> dict[str, Any] | None:
    token = broker.await_capability(timeout_s=timeout_s)
    return token.to_dict() if token else None


def get_resource(resource_type: str, intent: dict[str, Any], ttl_ms: int | None = None) -> dict[str, Any]:
    token = broker.request_capability(intent=intent, cap_type="resource", ttl_ms=ttl_ms,
                                      metadata={"resource_type": resource_type})
    return token.to_dict()


def put_resource(resource_type: str, token: dict[str, Any]) -> dict[str, Any]:
    _require(token, "put_resource", {"resource_type": resource_type})
    return {"status": "released", "resource_type": resource_type}


def network_request(destination: str, payload: bytes, token: dict[str, Any]) -> dict[str, Any]:
    _require(token, "network_request", {"destination": destination})
    return {"status": "sent", "destination": destination, "bytes": len(payload)}


def schedule_notification(message: str, token: dict[str, Any]) -> dict[str, Any]:
    _require(token, "schedule_notification", {"message": message[:80]})
    return {"status": "scheduled", "message": message}


def create_capability(cap_type: str, intent: dict[str, Any],
                      ttl_ms: int | None = None, uses: int | None = None,
                      metadata: dict[str, Any] | None = None,
                      publish: bool = True) -> dict[str, Any]:
    token = broker.request_capability(intent=intent, cap_type=cap_type,
                                      ttl_ms=ttl_ms, uses=uses, metadata=metadata,
                                      publish=publish)
    return token.to_dict()


def invoke_capability(token: dict[str, Any], action: str, resource: dict[str, Any] | None = None) -> dict[str, Any]:
    result = eventscope.authorize(token, action, resource=resource)
    if not result.allowed:
        raise PermissionError(f"Capability denied: {result.reason}")
    return {"status": "invoked", "action": action, "cap_type": result.cap_type}


def exit(code: int = 0) -> None:
    raise SystemExit(code)


def _require(token: dict[str, Any], action: str, resource: dict[str, Any] | None = None) -> None:
    result = eventscope.authorize(token, action, resource=resource)
    if not result.allowed:
        log.warn("sdk", "Capability denied", {"action": action, "reason": result.reason})
        raise PermissionError(f"Capability denied: {result.reason}")

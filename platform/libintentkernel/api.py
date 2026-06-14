"""
libintentkernel — IntentKernel SDK

Provides the 9 IntentKernel APIs:

  1.  draw(content)                          — render output
  2.  wait_event(timeout)                    — block until an event
  3.  get_resource(path, caps)               — capability-gated file read
  4.  put_resource(path, data, caps)         — capability-gated file write  [v0: stub]
  5.  network_request(host, port, payload)   — capability-gated TCP connect + send
  6.  schedule_notification(msg, delay)      — schedule a user notification
  7.  create_capability(scope, ttl, uses)    — mint a new capability token
  8.  invoke_capability(cap_id)              — exercise a capability by ID
  9.  exit(code)                             — clean exit with capability teardown

Service URLs (overridable via environment variables):
  INTENTOS_INTENTD_URL      default: http://127.0.0.1:5001
  INTENTOS_CAPD_URL         default: http://127.0.0.1:5002
  INTENTOS_IP_DESCRAMBLER_URL  default: http://127.0.0.1:5003
"""

import logging
import os
import socket
import sys
import time
from pathlib import Path
from typing import Any, Optional

import requests

_PLATFORM_DIR = Path(__file__).resolve().parent.parent
if str(_PLATFORM_DIR) not in sys.path:
    sys.path.insert(0, str(_PLATFORM_DIR))

from core.token import CapabilityToken

log = logging.getLogger("libintentkernel")

# ---------------------------------------------------------------------------
# Service URLs
# ---------------------------------------------------------------------------

_INTENTD_URL = os.environ.get("INTENTOS_INTENTD_URL", "http://127.0.0.1:5001")
_CAPD_URL = os.environ.get("INTENTOS_CAPD_URL", "http://127.0.0.1:5002")
_IP_DESCRAMBLER_URL = os.environ.get("INTENTOS_IP_DESCRAMBLER_URL", "http://127.0.0.1:5003")

# In-memory registry of capabilities created via create_capability()
_capabilities: dict[str, CapabilityToken] = {}

# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _request_token(
    scope: str,
    subject: str,
    target: Optional[str] = None,
    ttl: int = 60,
    uses: int = 1,
) -> CapabilityToken:
    """Ask intentd to issue a capability token."""
    try:
        resp = requests.post(
            f"{_INTENTD_URL}/issue",
            json={"scope": scope, "subject": subject, "target": target, "ttl": ttl, "uses": uses},
            timeout=3,
        )
        body = resp.json()
    except requests.RequestException as exc:
        raise RuntimeError(f"intentd unreachable: {exc}") from exc

    if not body.get("ok"):
        raise PermissionError(f"intentd denied capability: {body.get('error', 'unknown')}")

    return CapabilityToken.from_dict(body["token"])


def _validate_token(token: CapabilityToken, consume: bool = True) -> bool:
    """Ask capd to validate (and optionally consume) a token."""
    try:
        resp = requests.post(
            f"{_CAPD_URL}/validate",
            json={"token": token.to_dict(), "consume": consume},
            timeout=3,
        )
        body = resp.json()
        return bool(body.get("valid"))
    except requests.RequestException as exc:
        log.warning("capd unreachable (%s) — falling back to local validation", exc)
        # Local fallback: signature + TTL check only (no usage tracking)
        from core.token import get_key
        ok, _ = token.is_valid(get_key())
        return ok


def _check_ip(ip: str, token: CapabilityToken) -> tuple[str, str]:
    """
    Ask ip-descramblerd to analyse an IP.
    Returns (verdict, reason).
    On error, defaults to "allow" with a warning.
    """
    try:
        resp = requests.post(
            f"{_IP_DESCRAMBLER_URL}/analyze",
            json={"ip": ip, "token": token.to_dict()},
            timeout=3,
        )
        body = resp.json()
        if body.get("ok"):
            return body["verdict"], body["reason"]
        return "block", body.get("error", "ip-descramblerd denied")
    except requests.RequestException as exc:
        log.warning("ip-descramblerd unreachable (%s) — skipping IP check", exc)
        return "allow", "ip-descramblerd unreachable (skipped)"


def _resolve_ip(host: str) -> str:
    """Resolve hostname to its first IPv4 address."""
    try:
        return socket.gethostbyname(host)
    except socket.gaierror as exc:
        raise RuntimeError(f"DNS resolution failed for '{host}': {exc}") from exc


# ---------------------------------------------------------------------------
# Public API — the 9 IntentKernel primitives
# ---------------------------------------------------------------------------


def draw(content: Any) -> None:
    """
    API 1 — Render output to the current display surface.

    v0: writes to stdout.  Future: routed through the IK display manager.
    """
    print(content)


def wait_event(timeout: Optional[float] = None) -> None:
    """
    API 2 — Block until an IntentKernel event arrives or timeout expires.

    v0 stub: sleeps for `timeout` seconds (or 0.1 s if None) and returns None.
    Future: subscribes to the IK event bus.
    """
    time.sleep(timeout if timeout is not None else 0.1)


def get_resource(path: str, caps: Optional[list] = None) -> bytes:
    """
    API 3 — Read a file resource via a capability-gated path.

    Requests a "file_read" capability from intentd, validates it with
    capd, then performs the read.
    """
    subject = os.path.basename(sys.argv[0]) or "libintentkernel"
    token = _request_token("file_read", subject=subject, target=path, ttl=30, uses=1)

    if not _validate_token(token):
        raise PermissionError(f"capd denied file_read capability for '{path}'")

    log.info("get_resource: reading '%s' (token=%s)", path, token.id)
    return Path(path).read_bytes()


def put_resource(path: str, data: bytes, caps: Optional[list] = None) -> None:
    """
    API 4 — Write a file resource via a capability-gated path.

    v0 stub: file_write is denied by default policy.  Raises PermissionError.
    """
    subject = os.path.basename(sys.argv[0]) or "libintentkernel"
    # This will be denied by policy.json (file_write allowed: false)
    _request_token("file_write", subject=subject, target=path, ttl=30, uses=1)
    raise PermissionError("file_write capability is not granted in v0 policy")


def network_request(
    host: str,
    port: int,
    payload: Optional[bytes] = None,
    timeout: float = 10.0,
) -> bytes:
    """
    API 5 — Open a capability-gated TCP connection.

    Flow:
      1. Request "network_request" capability from intentd.
      2. Validate capability with capd.
      3. Resolve hostname to IP.
      4. Check IP with ip-descramblerd (uses the same token for scope matching).
      5. If verdict is "allow" or "warn", open socket and send payload.
      6. Return response bytes.

    Raises:
      RuntimeError     — intentd / capd unreachable
      PermissionError  — capability denied or IP blocked
      ConnectionError  — socket error
    """
    subject = os.path.basename(sys.argv[0]) or "libintentkernel"
    target = f"{host}:{port}"

    # Step 1 — Acquire capability
    token = _request_token("network_request", subject=subject, target=target, ttl=60, uses=1)
    log.info("network_request: token acquired id=%s target=%s", token.id, target)

    # Step 2 — Validate with capd (consume=False so it stays alive for step 4)
    if not _validate_token(token, consume=False):
        raise PermissionError(f"capd denied network_request capability for {target}")

    # Step 3 — Resolve hostname
    ip = _resolve_ip(host)

    # Step 4 — IP analysis (ip-descramblerd consumes the token here)
    verdict, reason = _check_ip(ip, token)
    log.info("network_request: IP verdict=%s reason=%s ip=%s", verdict, reason, ip)

    if verdict == "block":
        raise PermissionError(f"ip-descramblerd blocked {ip}: {reason}")
    if verdict == "warn":
        log.warning("network_request: suspicious IP %s — %s (proceeding)", ip, reason)

    # Step 5 — Connect and send
    try:
        with socket.create_connection((host, port), timeout=timeout) as sock:
            if payload:
                sock.sendall(payload)
            chunks = []
            sock.settimeout(timeout)
            try:
                while True:
                    chunk = sock.recv(4096)
                    if not chunk:
                        break
                    chunks.append(chunk)
            except socket.timeout:
                pass
        return b"".join(chunks)
    except OSError as exc:
        raise ConnectionError(f"socket error connecting to {target}: {exc}") from exc


def schedule_notification(msg: str, delay: float = 0.0) -> None:
    """
    API 6 — Schedule a user-visible notification.

    v0: prints after `delay` seconds in a fire-and-forget thread.
    """
    import threading

    def _deliver():
        if delay > 0:
            time.sleep(delay)
        print(f"[IK notification] {msg}")

    threading.Thread(target=_deliver, daemon=True).start()


def create_capability(
    scope: str,
    ttl: int = 60,
    uses: int = 1,
    target: Optional[str] = None,
) -> str:
    """
    API 7 — Mint a new capability token and return its ID.

    The token is stored locally and can be passed to invoke_capability().
    """
    subject = os.path.basename(sys.argv[0]) or "libintentkernel"
    token = _request_token(scope, subject=subject, target=target, ttl=ttl, uses=uses)
    _capabilities[token.id] = token
    log.info("create_capability: id=%s scope=%s ttl=%d uses=%d", token.id, scope, ttl, uses)
    return token.id


def invoke_capability(cap_id: str) -> bool:
    """
    API 8 — Exercise a previously created capability by ID.

    Returns True if the capability was valid and consumed, False otherwise.
    """
    token = _capabilities.get(cap_id)
    if token is None:
        raise KeyError(f"No local capability with id '{cap_id}'")

    valid = _validate_token(token, consume=True)
    if valid:
        _capabilities.pop(cap_id, None)
    log.info("invoke_capability: id=%s valid=%s", cap_id, valid)
    return valid


def exit(code: int = 0) -> None:
    """
    API 9 — Clean exit with capability teardown.

    Revokes all locally held capabilities before exiting.
    """
    log.info("exit: revoking %d local capabilities", len(_capabilities))
    for cap_id in list(_capabilities.keys()):
        try:
            requests.post(
                f"{_CAPD_URL}/revoke",
                json={"token_id": cap_id},
                timeout=2,
            )
        except requests.RequestException:
            pass
        _capabilities.pop(cap_id, None)
    sys.exit(code)

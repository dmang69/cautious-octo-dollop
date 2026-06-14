#!/usr/bin/env python3
"""
secure_curl — IntentKernel end-to-end demo (Option D).

A minimal HTTP/HTTPS client that routes every outbound connection
through the IntentKernel capability stack:

  network_request flow
    → intentd  (issue capability token)
    → capd     (validate token)
    → ip-descramblerd  (check target IP)
    → TCP connect  (only if verdict == allow/warn)

Usage:
    cd platform
    python demo/secure_curl.py http://example.com [--verbose]
    python demo/secure_curl.py http://example.com:8080/path

Requirements:
    intentd, capd, and ip-descramblerd must be running:
        python -m intentd &
        python -m capd &
        python -m ip_descramblerd &

    Or use start_services.sh / start_services.bat.
"""

import argparse
import sys
import time
from pathlib import Path
from urllib.parse import urlparse

# Make platform/ importable when run directly
_PLATFORM_DIR = Path(__file__).resolve().parent.parent
if str(_PLATFORM_DIR) not in sys.path:
    sys.path.insert(0, str(_PLATFORM_DIR))

from libintentkernel import draw, network_request, schedule_notification


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _build_http_request(host: str, path: str, port: int) -> bytes:
    """Construct a minimal HTTP/1.1 GET request."""
    lines = [
        f"GET {path or '/'} HTTP/1.1",
        f"Host: {host}:{port}" if port not in (80, 443) else f"Host: {host}",
        "Connection: close",
        "User-Agent: secure_curl/0.1 (IntentKernel)",
        "Accept: */*",
        "",
        "",
    ]
    return "\r\n".join(lines).encode()


def _parse_http_response(raw: bytes) -> tuple[int, str, bytes]:
    """
    Split raw HTTP response into (status_code, status_text, body).
    Returns (0, '', raw) on parse failure.
    """
    try:
        header_end = raw.find(b"\r\n\r\n")
        if header_end == -1:
            return 0, "", raw
        header_part = raw[:header_end].decode(errors="replace")
        body = raw[header_end + 4:]
        status_line = header_part.split("\r\n", 1)[0]
        parts = status_line.split(" ", 2)
        code = int(parts[1]) if len(parts) > 1 else 0
        text = parts[2] if len(parts) > 2 else ""
        return code, text, body
    except Exception:
        return 0, "", raw


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="secure_curl — capability-gated HTTP client (IntentKernel demo)",
    )
    parser.add_argument("url", help="URL to fetch (http:// only in v0)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Show capability flow")
    parser.add_argument("--timeout", type=float, default=10.0, help="Socket timeout (seconds)")
    args = parser.parse_args()

    parsed = urlparse(args.url)
    if parsed.scheme not in ("http", ""):
        print(
            "[secure_curl] NOTE: HTTPS is not yet supported in v0 (requires TLS shim).\n"
            "              Connecting on port 80 via HTTP instead.",
            file=sys.stderr,
        )

    host = parsed.hostname or args.url
    port = parsed.port or 80
    path = parsed.path or "/"
    if parsed.query:
        path = f"{path}?{parsed.query}"

    if args.verbose:
        draw(f"\n  secure_curl v0 — IntentKernel end-to-end demo")
        draw(f"  ═══════════════════════════════════════════════")
        draw(f"  Target   : {host}:{port}{path}")
        draw(f"  Flow     : intentd → capd → ip-descramblerd → TCP")
        draw("")

    t_start = time.time()

    try:
        payload = _build_http_request(host, path, port)

        if args.verbose:
            draw(f"  [1/4] Requesting capability from intentd …")

        raw_response = network_request(host, port, payload=payload, timeout=args.timeout)

        elapsed = round(time.time() - t_start, 3)

        if args.verbose:
            draw(f"  [4/4] Response received ({len(raw_response)} bytes in {elapsed}s)")
            draw("")

        status_code, status_text, body = _parse_http_response(raw_response)

        if args.verbose and status_code:
            draw(f"  HTTP {status_code} {status_text}")
            draw(f"  ───────────────────────────────────────────────")

        # Print body
        try:
            sys.stdout.buffer.write(body)
        except AttributeError:
            print(body.decode(errors="replace"))

        if args.verbose:
            draw("")
            draw(f"  ✓ Done ({elapsed}s)")

        schedule_notification(f"secure_curl completed: {host}:{port} → HTTP {status_code}")

    except PermissionError as exc:
        draw(f"\n  ✗ BLOCKED: {exc}")
        draw(  "    The IntentKernel capability model denied this request.")
        sys.exit(1)
    except RuntimeError as exc:
        draw(f"\n  ✗ ERROR: {exc}")
        draw(  "    Make sure intentd, capd, and ip-descramblerd are running.")
        draw(  "    Start them with:  ./start_services.sh")
        sys.exit(2)
    except ConnectionError as exc:
        draw(f"\n  ✗ CONNECTION ERROR: {exc}")
        sys.exit(3)
    except KeyboardInterrupt:
        draw("\n  Interrupted.")
        sys.exit(130)


if __name__ == "__main__":
    main()

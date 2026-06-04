"""
IntentKernel SDK demo — exercises the 9 primitive APIs.
"""

from __future__ import annotations

import os
import sys

BASE_DIR = os.path.dirname(os.path.dirname(__file__))
sys.path.insert(0, BASE_DIR)

from core import sdk


def main() -> int:
    print("IntentKernel SDK demo")
    print("=====================\n")

    intent = {"source": "user_click", "action": "network_request"}
    token = sdk.create_capability("network", intent, ttl_ms=5000, uses=1)
    print("Issued token:", token["capability"]["id"])

    result = sdk.network_request("https://example.com", b"hello", token)
    print("Network request result:", result)

    try:
        sdk.network_request("https://example.com", b"second", token)
    except PermissionError as exc:
        print("Second request blocked:", exc)

    sdk.create_capability("event", {"source": "timer"}, ttl_ms=3000, uses=1)
    queued = sdk.wait_event(timeout_s=0.1)
    if queued:
        print("Event token queued:", queued["capability"]["id"])
    else:
        print("No event token received")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

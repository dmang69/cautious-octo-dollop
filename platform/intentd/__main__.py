"""
intentd — entry point.

Usage:
    python -m intentd [--host HOST] [--port PORT]

Default: http://127.0.0.1:5001
"""

import argparse
import logging
import sys
from pathlib import Path

# Make platform/ importable
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from intentd.server import create_app


def main() -> None:
    parser = argparse.ArgumentParser(description="IntentKernel Intent Broker (intentd)")
    parser.add_argument("--host", default="127.0.0.1", help="Bind host (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=5001, help="HTTP port (default: 5001)")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    logging.basicConfig(
        level=logging.DEBUG if args.debug else logging.INFO,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    )

    app = create_app()
    print(f"\n  intentd — IntentKernel Intent Broker")
    print(f"  ─────────────────────────────────────")
    print(f"  Listening on http://{args.host}:{args.port}\n")
    app.run(host=args.host, port=args.port, debug=False, use_reloader=False)


if __name__ == "__main__":
    main()

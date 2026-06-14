"""
capd — entry point.

Usage:
    python -m capd [--host HOST] [--port PORT]

Default: http://127.0.0.1:5002
"""

import argparse
import logging
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from capd.server import create_app


def main() -> None:
    parser = argparse.ArgumentParser(description="IntentKernel Capability Verifier (capd)")
    parser.add_argument("--host", default="127.0.0.1", help="Bind host (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=5002, help="HTTP port (default: 5002)")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    logging.basicConfig(
        level=logging.DEBUG if args.debug else logging.INFO,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    )

    app = create_app()
    print(f"\n  capd — IntentKernel Capability Verifier")
    print(f"  ─────────────────────────────────────────")
    print(f"  Listening on http://{args.host}:{args.port}\n")
    app.run(host=args.host, port=args.port, debug=False, use_reloader=False)


if __name__ == "__main__":
    main()

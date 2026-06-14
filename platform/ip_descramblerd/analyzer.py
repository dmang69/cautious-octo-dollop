"""
ip-descramblerd — IP threat analyzer.

Loads a local threat database (threats.json) and evaluates whether
a given IP address should be allowed, warned about, or blocked.

Verdict values: "allow" | "warn" | "block"

In v0, only local threat DB lookups are performed.
Future versions will integrate AbuseIPDB, VirusTotal, and Shodan.
"""

import ipaddress
import json
import logging
import os
import time
from pathlib import Path
from typing import Optional

log = logging.getLogger("ip-descramblerd.analyzer")

_THREATS_PATH = Path(
    os.environ.get(
        "INTENTOS_THREATS",
        Path(__file__).resolve().parent.parent / "config" / "threats.json",
    )
)

_RELOAD_INTERVAL = 300  # re-read threats.json every 5 minutes


class ThreatDB:
    """
    Loads and queries the local IP threat database.

    threat DB schema (threats.json):
      {
        "blocked_ips":   ["1.2.3.4", ...],
        "blocked_cidrs": ["1.2.3.0/24", ...],
        "suspicious_ips":["5.6.7.8", ...]
      }
    """

    def __init__(self, path: Optional[Path] = None) -> None:
        self._path = path or _THREATS_PATH
        self._blocked_ips: set[str] = set()
        self._blocked_nets: list[ipaddress.IPv4Network | ipaddress.IPv6Network] = []
        self._suspicious_ips: set[str] = set()
        self._loaded_at: float = 0.0
        self._load()

    def _load(self) -> None:
        try:
            data = json.loads(self._path.read_text())
        except (OSError, json.JSONDecodeError) as exc:
            log.warning("Could not load threats.json: %s — using empty DB", exc)
            data = {}

        self._blocked_ips = set(data.get("blocked_ips", []))
        self._suspicious_ips = set(data.get("suspicious_ips", []))
        self._blocked_nets = []
        for cidr in data.get("blocked_cidrs", []):
            try:
                self._blocked_nets.append(ipaddress.ip_network(cidr, strict=False))
            except ValueError:
                log.warning("Invalid CIDR in threats.json: %s", cidr)
        self._loaded_at = time.time()
        log.debug(
            "ThreatDB loaded: %d blocked IPs, %d CIDRs, %d suspicious IPs",
            len(self._blocked_ips), len(self._blocked_nets), len(self._suspicious_ips),
        )

    def _maybe_reload(self) -> None:
        if time.time() - self._loaded_at > _RELOAD_INTERVAL:
            self._load()

    def lookup(self, ip: str) -> tuple[str, str]:
        """
        Returns (verdict, reason).

        verdict: "allow" | "warn" | "block"
        """
        self._maybe_reload()

        try:
            addr = ipaddress.ip_address(ip)
        except ValueError:
            return "block", f"'{ip}' is not a valid IP address"

        # 1. Loopback / link-local are always allowed (localhost traffic)
        if addr.is_loopback or addr.is_link_local:
            return "allow", "loopback/link-local address — always allowed"

        # 2. Explicit block list
        if ip in self._blocked_ips:
            return "block", f"{ip} is in the blocked IP list"

        # 3. Blocked CIDRs
        for net in self._blocked_nets:
            if addr in net:
                return "block", f"{ip} falls within blocked network {net}"

        # 4. Suspicious list (allow but warn)
        if ip in self._suspicious_ips:
            return "warn", f"{ip} is in the suspicious IP list"

        return "allow", "no threat indicators found"


# Module-level singleton
_db = ThreatDB()


def analyze_ip(ip: str) -> dict:
    """
    Analyze an IP address and return a verdict dict.

    Returns:
        {
          "ip":      str,
          "verdict": "allow" | "warn" | "block",
          "reason":  str,
          "ts":      float
        }
    """
    verdict, reason = _db.lookup(ip)
    result = {
        "ip": ip,
        "verdict": verdict,
        "reason": reason,
        "ts": time.time(),
    }
    log.info("IP analysis: ip=%s verdict=%s reason=%s", ip, verdict, reason)
    return result

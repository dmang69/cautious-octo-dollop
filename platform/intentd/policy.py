"""
intentd — capability issuance policy loader.

Reads platform/config/policy.json and decides whether a given
(subject, scope, ttl, uses) combination is permitted.
"""

import fnmatch
import json
import os
from pathlib import Path
from typing import Optional

_POLICY_PATH = Path(
    os.environ.get(
        "INTENTOS_POLICY",
        Path(__file__).resolve().parent.parent / "config" / "policy.json",
    )
)


def _load_policy() -> dict:
    try:
        return json.loads(_POLICY_PATH.read_text())
    except (OSError, json.JSONDecodeError):
        return {"version": "1", "default_deny": False, "rules": []}


class PolicyEngine:
    """Evaluates whether a capability request is allowed by policy."""

    def __init__(self, policy_path: Optional[Path] = None) -> None:
        path = policy_path or _POLICY_PATH
        try:
            self._policy = json.loads(path.read_text())
        except (OSError, json.JSONDecodeError):
            self._policy = {"version": "1", "default_deny": False, "rules": []}

    def check(
        self,
        scope: str,
        subject: str,
        ttl: int,
        uses: int,
    ) -> tuple[bool, str, dict]:
        """
        Return (allowed: bool, reason: str, rule: dict).

        Rules are evaluated in order; the first matching rule wins.
        If no rule matches, the default_deny setting applies.
        """
        for rule in self._policy.get("rules", []):
            if rule.get("scope") != scope:
                continue
            pattern = rule.get("subject_pattern", "*")
            if not fnmatch.fnmatch(subject, pattern):
                continue

            if not rule.get("allowed", False):
                return False, rule.get("description", f"scope '{scope}' is denied by policy"), rule

            max_ttl = rule.get("max_ttl", 300)
            max_uses = rule.get("max_uses", 1)

            if ttl > max_ttl:
                return False, f"requested TTL {ttl}s exceeds policy max {max_ttl}s", rule
            if uses > max_uses:
                return False, f"requested uses {uses} exceeds policy max {max_uses}", rule

            return True, "ok", rule

        # No rule matched
        if self._policy.get("default_deny", False):
            return False, f"no policy rule for scope '{scope}' (default deny)", {}
        return True, "ok (default allow)", {}

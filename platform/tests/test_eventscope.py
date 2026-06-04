"""
Tests for eventscope action mapping enforcement.
"""

from __future__ import annotations

import os
import sys
import unittest


BASE_DIR = os.path.dirname(os.path.dirname(__file__))
sys.path.insert(0, BASE_DIR)

from core import broker, eventscope


class EventscopeMappingTests(unittest.TestCase):
    def _token_for(self, cap_type: str) -> dict:
        token = broker.issue_capability(
            intent={"source": "unit_test"},
            cap_type=cap_type,
            ttl_ms=5000,
            uses=1,
        )
        return token.to_dict()

    def test_action_mapping_allows_matching_type(self):
        for action, cap_type in eventscope.ACTION_CAP_TYPES.items():
            token = self._token_for(cap_type)
            result = eventscope.authorize(token, action, resource={"action": action})
            self.assertTrue(result.allowed, f"{action} should allow {cap_type}")

    def test_action_mapping_rejects_mismatched_type(self):
        for action, cap_type in eventscope.ACTION_CAP_TYPES.items():
            mismatch = "network" if cap_type != "network" else "resource"
            token = self._token_for(mismatch)
            result = eventscope.authorize(token, action, resource={"action": action})
            self.assertFalse(result.allowed, f"{action} should reject {mismatch}")
            self.assertEqual(result.reason, "capability_type_mismatch")


if __name__ == "__main__":
    unittest.main()

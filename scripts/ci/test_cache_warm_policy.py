#!/usr/bin/env python3
"""Reject unsafe or mismatched trusted-main cache warming."""
import unittest
from pathlib import Path

import check_workflow as policy

ROOT = Path(__file__).resolve().parents[2]
WARMER = ROOT / ".github/workflows/ci-cache-warm.yml"
REUSABLE = ROOT / ".github/workflows/ci-reusable.yml"


class CacheWarmPolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.reopen = policy.job_blocks(REUSABLE.read_text(encoding="utf-8"))["installed-reopen"]
        cls.warmer = WARMER.read_text(encoding="utf-8")

    def test_current_warmer_matches_the_reopen_job(self):
        policy.verify_macos_reopen_cache_warm(self.warmer, self.reopen)

    def test_rejects_untrusted_triggers_guards_and_mismatched_build(self):
        mutations = [
            ("  workflow_dispatch:\n", "  workflow_dispatch:\n  pull_request:\n"),
            ("  workflow_dispatch:\n", "  workflow_dispatch:\n  push:\n"),
            ("github.ref == 'refs/heads/main'", "github.ref != 'refs/heads/main'"),
            ("    runs-on: macos-latest", "    if: ${{ true }}\n    runs-on: macos-latest"),
            ("installed-reopen-macos-latest", "wrong-key"),
            ("          persist-credentials: false", "          ref: attacker\n          persist-credentials: false"),
            ("      - name: Build cache inputs only", "      - run: unexpected-command\n      - name: Build cache inputs only"),
            ("cargo test -p rrflow-cli --test installed_reopen --release --locked --no-run", "cargo test -p rrflow-cli --test installed_reopen --locked --no-run"),
        ]
        for before, after in mutations:
            with self.subTest(before=before):
                self.assertIn(before, self.warmer)
                with self.assertRaises(policy.PolicyError):
                    policy.verify_macos_reopen_cache_warm(self.warmer.replace(before, after), self.reopen)

    def test_rejects_drift_in_the_candidate_cache_key(self):
        old = "shared-key: installed-reopen-${{ matrix.os }}"
        self.assertIn(old, self.reopen)
        with self.assertRaises(policy.PolicyError):
            policy.verify_macos_reopen_cache_warm(
                self.warmer, self.reopen.replace(old, "shared-key: changed-${{ matrix.os }}")
            )


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
"""Reject unsafe or mismatched trusted-main cache warming."""
import unittest
from pathlib import Path

import check_workflow as policy

ROOT = Path(__file__).resolve().parents[2]
WARMER = ROOT / ".github/workflows/ci-cache-warm.yml"
WINDOWS_WARMER = ROOT / ".github/workflows/ci-windows-cache-warm.yml"
REUSABLE = ROOT / ".github/workflows/ci-reusable.yml"


class CacheWarmPolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.reopen = policy.job_blocks(REUSABLE.read_text(encoding="utf-8"))["installed-reopen"]
        cls.warmer = WARMER.read_text(encoding="utf-8")
        cls.windows_warmer = WINDOWS_WARMER.read_text(encoding="utf-8")

    def test_current_warmer_matches_the_reopen_job(self):
        policy.verify_macos_reopen_cache_warm(self.warmer, self.reopen)
        policy.verify_windows_reopen_cache_warm(self.windows_warmer, self.reopen)

    def test_windows_warmer_rejects_untrusted_or_mismatched_inputs(self):
        mutations = [
            ("  workflow_dispatch:\n", "  workflow_dispatch:\n  pull_request:\n"),
            ("  workflow_dispatch:\n", "  workflow_dispatch:\n  push:\n"),
            ("  contents: read\n", "  contents: read\n  actions: write\n"),
            ("  cancel-in-progress: false", "  cancel-in-progress: true"),
            ("jobs:\n", "defaults:\n  run:\n    shell: attacker\njobs:\n"),
            ("    runs-on: windows-latest", "    permissions:\n      actions: write\n    runs-on: windows-latest"),
            ("    runs-on: windows-latest", "    runs-on: windows-latest\n    runs-on: self-hosted"),
            ("github.ref == 'refs/heads/main'", "github.ref != 'refs/heads/main'"),
            ("runs-on: windows-latest", "runs-on: self-hosted"),
            ("timeout-minutes: 120", "timeout-minutes: 999"),
            ('CARGO_BUILD_JOBS: "2"', 'CARGO_BUILD_JOBS: "1"'),
            ('CARGO_PROFILE_DEV_DEBUG: "0"', 'CARGO_PROFILE_DEV_DEBUG: "1"'),
            ('  CARGO_INCREMENTAL: "0"', '  CARGO_INCREMENTAL: "1"'),
            ("installed-reopen-windows-latest", "wrong-key"),
            ("cache-on-failure: false", "cache-on-failure: true"),
            ("          submodules: recursive", "          submodules: false"),
            ("          persist-credentials: false", "          persist-credentials: true"),
            ("          persist-credentials: false", "          ref: attacker\n          persist-credentials: false"),
            ("          components: clippy", "          components: clippy\n          toolchain: attacker"),
            ("          cache-on-failure: false", "          cache-on-failure: false\n          save-if: false"),
            ("      - name: Build cache inputs only", "      - run: unexpected-command\n      - name: Build cache inputs only"),
            ("--release --locked --no-run", "--locked --no-run"),
            ("        run: cargo test", "        continue-on-error: true\n        run: cargo test"),
        ]
        for before, after in mutations:
            with self.subTest(before=before):
                self.assertIn(before, self.windows_warmer)
                with self.assertRaises(policy.PolicyError):
                    policy.verify_windows_reopen_cache_warm(
                        self.windows_warmer.replace(before, after), self.reopen
                    )
        with self.assertRaises(policy.PolicyError):
            policy.verify_windows_reopen_cache_warm(
                self.windows_warmer + "  cache-reopen:\n    runs-on: windows-latest\n",
                self.reopen,
            )

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

    def test_rejects_missing_or_unsafe_toolchain_isolation(self):
        marker = "      - name: Isolate macOS reopen toolchain\n"
        for surface, value in (("warmer", self.warmer), ("reopen", self.reopen)):
            self.assertIn(marker, value)
            mutations = (
                (marker, "      - name: Toolchain setup removed\n"),
                ('toolchain_home="${RUNNER_TEMP}/rrflow-reopen-rustup"',
                 'toolchain_home="${RUNNER_TEMP}/different"'),
                ('test ! -e "$toolchain_home"', 'true'),
                ("printf 'RUSTUP_HOME=%s\\n'", "printf 'OTHER_HOME=%s\\n'"),
                ("if: runner.os == 'macOS'", "if: runner.os != 'macOS'"),
            )
            for before, after in mutations:
                with self.subTest(surface=surface, before=before):
                    self.assertIn(before, value)
                    changed = value.replace(before, after)
                    with self.assertRaises(policy.PolicyError):
                        policy.verify_macos_reopen_cache_warm(
                            changed if surface == "warmer" else self.warmer,
                            changed if surface == "reopen" else self.reopen,
                        )

    def test_rejects_isolation_after_toolchain_installation(self):
        marker = "      - name: Isolate macOS reopen toolchain\n"
        install = "      - uses: dtolnay/rust-toolchain@"
        for surface, value in (("warmer", self.warmer), ("reopen", self.reopen)):
            start = value.index(marker)
            end = value.index("      - uses:", start)
            block = value[start:end]
            changed = value[:start] + value[end:]
            install_end = changed.index("      - uses:", changed.index(install) + len(install))
            changed = changed[:install_end] + block + changed[install_end:]
            with self.subTest(surface=surface), self.assertRaises(policy.PolicyError):
                policy.verify_macos_reopen_cache_warm(
                    changed if surface == "warmer" else self.warmer,
                    changed if surface == "reopen" else self.reopen,
                )

    def test_rejects_drift_in_the_candidate_cache_key(self):
        old = "shared-key: installed-reopen-${{ matrix.os }}"
        self.assertIn(old, self.reopen)
        with self.assertRaises(policy.PolicyError):
            policy.verify_macos_reopen_cache_warm(
                self.warmer, self.reopen.replace(old, "shared-key: changed-${{ matrix.os }}")
            )
        with self.assertRaises(policy.PolicyError):
            policy.verify_windows_reopen_cache_warm(
                self.windows_warmer, self.reopen.replace(old, "shared-key: changed-${{ matrix.os }}")
            )

    def test_windows_warmer_rejects_candidate_build_drift(self):
        mutations = [
            ("CARGO_BUILD_JOBS: ${{ matrix.os == 'windows-latest' && '2' || '1' }}",
             "CARGO_BUILD_JOBS: 1"),
            ('CARGO_PROFILE_TEST_DEBUG: "0"', 'CARGO_PROFILE_TEST_DEBUG: "1"'),
            ("cache-on-failure: false", "cache-on-failure: true"),
            ("cargo test -p rrflow-cli --test installed_reopen --release --locked --no-run",
             "cargo test -p rrflow-cli --test installed_reopen --locked --no-run"),
            ("dtolnay/rust-toolchain@d124f86f28ff42855784ff3319f48ac182c248f8",
             "dtolnay/rust-toolchain@" + "a" * 40),
        ]
        for before, after in mutations:
            with self.subTest(before=before):
                self.assertIn(before, self.reopen)
                with self.assertRaises(policy.PolicyError):
                    policy.verify_windows_reopen_cache_warm(
                        self.windows_warmer, self.reopen.replace(before, after)
                    )


if __name__ == "__main__":
    unittest.main()

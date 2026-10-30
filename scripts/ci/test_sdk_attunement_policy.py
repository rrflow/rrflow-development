#!/usr/bin/env python3
"""Keep the real-primary Python stale-source recovery test required in CI."""
import unittest

from check_workflow import REUSABLE, PolicyError, job_blocks, verify_sdk_conformance


class SdkAttunementPolicyTests(unittest.TestCase):
    def setUp(self):
        jobs = job_blocks(REUSABLE.read_text(encoding="utf-8"))
        self.sdk = jobs["sdk-conformance"]
        self.verify = jobs["verify"]
        self.step = (
            "      - name: Exercise installed Python attunement recovery\n"
            "        timeout-minutes: 10\n"
            "        shell: bash\n"
            "        run: |\n"
            "          set -euo pipefail\n"
            "          cargo build -p rrflow-cli --bin rrflow --profile test --locked\n"
            '          export RRFLOW_TEST_BINARY="$GITHUB_WORKSPACE/target/debug/rrflow"\n'
            '          test -x "$RRFLOW_TEST_BINARY"\n'
            '          uv --directory sdks/python run --frozen pytest -o addopts= --color=no -q tests/test_installed_attunement.py | tee "$RUNNER_TEMP/rrflow-python-attunement.txt"\n'
            "          grep -Eq '^1 passed in [0-9.]+s[[:space:]]*$' \"$RUNNER_TEMP/rrflow-python-attunement.txt\"\n"
        )
        self.assertEqual(self.sdk.count(self.step), 1)

    def test_checked_in_step_and_policy_are_accepted(self):
        verify_sdk_conformance(self.sdk, self.verify)

    def test_missing_or_weakened_step_is_rejected(self):
        for changed in (
            "", self.step + self.step,
            self.step.replace("        shell: bash\n", "        if: false\n        shell: bash\n"),
            self.step.replace("          set -euo pipefail\n", ""),
            self.step.replace("--profile test", "--release"),
            self.step.replace("          test -x", "          # test -x"),
            self.step.replace("          grep -Eq", "          # grep -Eq"),
            self.step.replace("timeout-minutes: 10", "timeout-minutes: 0"),
            self.step + "        continue-on-error: true\n",
        ):
            with self.subTest(changed=changed):
                with self.assertRaises(PolicyError):
                    verify_sdk_conformance(self.sdk.replace(self.step, changed), self.verify)

    def test_missing_or_bypassed_policy_regression_is_rejected(self):
        command = "          python3 scripts/ci/test_sdk_attunement_policy.py\n"
        for replacement in ("", "          # python3 scripts/ci/test_sdk_attunement_policy.py\n"):
            with self.subTest(replacement=replacement):
                with self.assertRaises(PolicyError):
                    verify_sdk_conformance(self.sdk, self.verify.replace(command, replacement))
        marker = "      - name: Enforce repository CI policy\n"
        for bypass in ("        if: false\n", "        continue-on-error: true\n"):
            with self.subTest(bypass=bypass):
                with self.assertRaises(PolicyError):
                    verify_sdk_conformance(self.sdk, self.verify.replace(marker, marker + bypass))


if __name__ == "__main__":
    unittest.main()

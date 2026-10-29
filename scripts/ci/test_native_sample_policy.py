#!/usr/bin/env python3
"""Reject CI changes that omit the Windows native worker lifecycle oracle."""
import unittest
from check_workflow import REUSABLE, PolicyError, job_blocks, verify_native_sample_worker_ci


class NativeSamplePolicyTests(unittest.TestCase):
    def setUp(self):
        jobs = job_blocks(REUSABLE.read_text(encoding="utf-8"))
        self.reopen = jobs["installed-reopen"]
        self.verify = jobs["verify"]
        self.step = (
            "      - name: Verify owned Windows native sampler lifecycle\n"
            "        if: runner.os == 'Windows'\n"
            "        timeout-minutes: 5\n"
            "        run: python scripts/ci/test_native_sample_worker.py --profile release\n"
        )
        self.assertEqual(self.reopen.count(self.step), 1)

    def test_checked_in_lane_and_policy_are_accepted(self):
        verify_native_sample_worker_ci(self.reopen, self.verify)

    def test_removed_or_duplicated_oracle_is_rejected(self):
        for replacement in ("", self.step + self.step):
            with self.subTest(replacement=replacement):
                with self.assertRaises(PolicyError):
                    verify_native_sample_worker_ci(self.reopen.replace(self.step, replacement), self.verify)

    def test_weakened_condition_command_timeout_or_failure_is_rejected(self):
        for changed in (
            self.step.replace("runner.os == 'Windows'", "false"),
            self.step.replace("--profile release", "--profile debug"),
            self.step.replace("timeout-minutes: 5", "timeout-minutes: 0"),
            self.step + "        continue-on-error: true\n",
        ):
            with self.subTest(changed=changed):
                with self.assertRaises(PolicyError):
                    verify_native_sample_worker_ci(self.reopen.replace(self.step, changed), self.verify)

    def test_missing_or_bypassed_regression_runner_is_rejected(self):
        command = "          python3 scripts/ci/test_native_sample_policy.py\n"
        for replacement in ("          true\n", "          # python3 scripts/ci/test_native_sample_policy.py\n"):
            with self.subTest(replacement=replacement):
                with self.assertRaises(PolicyError):
                    verify_native_sample_worker_ci(self.reopen, self.verify.replace(command, replacement))
        marker = "      - name: Enforce repository CI policy\n"
        with self.assertRaises(PolicyError):
            verify_native_sample_worker_ci(
                self.reopen, self.verify.replace(marker, marker + "        continue-on-error: true\n")
            )


if __name__ == "__main__":
    unittest.main()

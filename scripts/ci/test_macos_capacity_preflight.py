"""Selection tests do not create or fill a native filesystem."""

import io
import unittest
from contextlib import redirect_stderr
from types import SimpleNamespace
from unittest.mock import patch

import run_macos_capacity_preflight as preflight
from run_native_capacity import POINTS


class MacosPreflightSelection(unittest.TestCase):
    def test_default_preserves_all_points_and_three_fresh_existing_pairs(self):
        self.assertEqual(
            preflight.case_arguments("all", "both", "3"),
            ["--mode", "fresh", "--mode", "existing"] * 3,
        )

    def test_one_point_and_mode_can_be_repeated_without_other_cases(self):
        self.assertEqual(
            preflight.case_arguments(POINTS[0], "fresh", "2"),
            ["--point", POINTS[0], "--mode", "fresh", "--mode", "fresh"],
        )

    def test_invalid_or_unbounded_input_cannot_become_a_subprocess_argument(self):
        for point, mode, repeats in [
            ("--help", "fresh", "1"),
            (POINTS[0], "unknown", "1"),
            (POINTS[0], "both", "0"),
            (POINTS[0], "both", "4"),
            (POINTS[0], "both", "1; echo bad"),
        ]:
            with (
                self.subTest(point=point, mode=mode, repeats=repeats),
                self.assertRaises(ValueError),
            ):
                preflight.case_arguments(point, mode, repeats)

    def test_runner_uses_an_argument_vector_and_preserves_failure(self):
        with (
            patch(
                "sys.argv",
                [
                    "preflight",
                    "--point",
                    POINTS[0],
                    "--mode",
                    "existing",
                    "--repetitions",
                    "1",
                ],
            ),
            patch.object(
                preflight.subprocess, "run", return_value=SimpleNamespace(returncode=17)
            ) as run,
        ):
            self.assertEqual(preflight.main(), 17)
        args = run.call_args.args[0]
        self.assertEqual(args[2:], ["--point", POINTS[0], "--mode", "existing"])
        self.assertTrue(args[1].endswith("run_native_capacity.py"))
        self.assertIs(run.call_args.kwargs["check"], False)

    def test_invalid_dispatch_cannot_start_the_native_runner(self):
        with (
            patch("sys.argv", ["preflight", "--point", "not-a-point"]),
            patch.object(preflight.subprocess, "run") as run,
            redirect_stderr(io.StringIO()),
            self.assertRaises(SystemExit) as error,
        ):
            preflight.main()
        self.assertEqual(error.exception.code, 2)
        run.assert_not_called()


if __name__ == "__main__":
    unittest.main()

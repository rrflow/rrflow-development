#!/usr/bin/env python3
"""Exercise native create-failure retention; never a capacity qualification."""
import json
import os
from pathlib import Path
import sys
import tempfile
from unittest.mock import patch

import run_native_capacity as runner


def main():
    runner.require(sys.platform == "darwin", "native failure preflight requires macOS")
    parent = Path(os.environ["RUNNER_TEMP"]).resolve(strict=True)
    scratch = parent / "rrflow-native-failure-probe"
    runner.require(not scratch.exists() and not scratch.is_symlink(), "probe scratch already exists")
    scratch.mkdir()
    fixture = Path(tempfile.mkdtemp(prefix="rrflow-capacity-", dir=scratch)).resolve(strict=True)
    diagnostics = scratch / "rrflow-capacity-diagnostics"
    config = {"modes": ["fresh", "existing"], "points": list(runner.POINTS),
              "size_mib": 256, "executable": "not-run", "harness_sha256": "0" * 64}
    original = runner.native_command
    calls = []

    def invalid_filesystem(*args, **kwargs):
        runner.require(args[:2] == ("hdiutil", "create"), "unexpected native operation")
        calls.append(args[:2])
        altered = list(args)
        altered[altered.index("-fs") + 1] = "RRFLOW_INVALID_FILESYSTEM"
        return original(*altered, **kwargs)

    with patch.dict(os.environ, {"RUNNER_TEMP": str(scratch),
                                "RRFLOW_CAPACITY_DIAGNOSTICS": str(diagnostics)}), \
            patch.object(runner, "native_command", side_effect=invalid_filesystem):
        try:
            runner.cases(config, fixture)
        except runner.NativeCommandFailure as error:
            runner.require((error.tool, error.verb) == ("hdiutil", "create") and error.returncode != 0,
                           "unexpected native failure")
        else:
            raise RuntimeError("invalid native create unexpectedly succeeded")

        runner.require(len(calls) == 1, "later capacity cases must not run")
        files = list(diagnostics.glob("*.json"))
        runner.require(len(files) == 1, "exactly one diagnostic receipt is required")
        receipt = json.loads(files[0].read_text(encoding="utf-8"))
        runner.require((receipt["mode"], receipt["point"], receipt["case_index"])
                       == ("fresh", runner.POINTS[0], 1), "diagnostic case identity differs")
        runner.require((receipt["tool"], receipt["verb"]) == ("hdiutil", "create")
                       and receipt["returncode"] != 0 and receipt["github_sha"] == os.environ.get("GITHUB_SHA")
                       and bool(receipt["stderr_head"] or receipt["stdout_head"])
                       and receipt["cleanup_state"] == "unknown"
                       and files[0].stat().st_size <= 256 * 1024, "native diagnostic receipt invalid")
        # A newly created image is not an expected outcome of this injection.
        # Leave the owned fixture intact for inspection rather than deleting it.
        runner.require(not (fixture / "capacity.dmg").exists(), "unexpected native image; fixture retained")
        runner.cleanup_detached_volume(fixture)
        fixture.rmdir()
        print("native_failure_capture=verified; actual_hdiutil=true; capacity_pass=false; fixture_cleanup=complete")


if __name__ == "__main__":
    main()

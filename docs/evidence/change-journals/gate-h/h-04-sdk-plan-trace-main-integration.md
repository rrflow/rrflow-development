# H-04: integrate Windows SDK harness with merged plan and C-07 traces

**Status:** active integration evidence; new-head candidate and merge-group CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-sdk-plan-trace-main-integration`
**Owner:** H-04 / POAM-011 / [PR177](https://github.com/rrflow/rrflow-development/pull/177)

The PR described this integration before its effects. Clean candidate
`137b90a13a7712ced9f96a927d6963d58c1c532c` passed candidate run
[40743798490](https://github.com/rrflow/rrflow-development/actions/runs/40743798490),
including the Windows manifest test and macOS crash job that failed in the
preceding run. Public main `81fce08e14497bbc280790beefe4666716355aec`, tree
`0d92234da88dc5d4c0f14e15b01e6efa13a11f4f`, includes merged PR176 and PR178.
Ordinary merge conflicted only in POAM-011, POAM-035 and generated execution
inventory. POAM-011 retained the original Windows probe and candidate repair
history in its newer combined wording and records the prior candidate pass;
POAM-035 retained both the macOS fixture failure and the separate C-07 trace
observation. Generated inventory was regenerated. No source or existing journal
was removed, no permission check was weakened and no status was closed.

On Windows x64, rustc 1.99.1, isolated `target/sdk-native-fix-build`, locked
offline dependencies and disabled debug info: `python -m unittest discover -s
scripts/ci -p test_sdk_private_manifest.py` passed four tests, with one POSIX
case skipped; `cargo test --offline --locked -p rrd-client --example
sdk_conformance_server`, `cargo test --offline --locked -p rrd-client
--all-targets`, and `cargo clippy --offline --locked -p rrd-client --all-targets
-- -D warnings` passed. The exact commands and results for real Windows server,
Rust, Go and .NET conformance are in the previous immutable correction journal;
they were not rerun during this merge-only refresh. The prior full candidate
pass is not transferred to the new head.

Reviewed the PR brief, both conflicted POAM sections, complete non-generated
diff and prior [correction journal](h-04-sdk-windows-manifest-ci-correction.md).
This refresh only integrates already implemented code and evidence; no new
upstream adaptation, runtime trace, benchmark or architecture authority is
introduced. Final navigation, inventory, formatting, documentation and workflow
checks are required before commit. New-head candidate and protected merge-group
CI remain required. Full Windows six-client, separate-principal, ARM and macOS
qualification and the broader provider bridge remain open. Checklist: exact
baseline and prior CI recorded; conflicts preserved and inspected; narrow and
owning tests rerun; new journal and generated projections; no status promotion.

The first `python scripts/ci/check_documentation.py` check rejected this record because its status line did not declare `active` or `historical`; the status was corrected and the check rerun.

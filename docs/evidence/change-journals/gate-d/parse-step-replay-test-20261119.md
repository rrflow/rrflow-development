# A replayed Parse Step commits nothing, now tested

**Status:** active change-journal evidence; D-02 remains open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/parse-step-replay-test-20261119`

**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), attunement Parse

**Starting revision:** main `e2c763f1c`.

## Change brief

- **Current behaviour.** A Step asked again under the same key is answered from its stored receipt before dispatch. For Parse that means nothing is parsed, committed or advanced a second time. No Parse test asserted it: the archive record noted that the deleted Parse executor's test `a_step_replay_returns_its_receipt_and_commits_nothing` was not carried, because the general replay path covers it.
- **Target behaviour.** Unchanged. One test now holds it for Parse.
- **Scope.** `crates/authority/rrd-engine/src/engine/tests/attunement/parse_step.rs`, this record, one row of [the reading of unmerged work](../repository/unmerged-pull-requests-by-intent-20261119.md).
- **Failure oracle.** The test passes on `main`; it would fail if a replay parsed the waiting file, added a symbol, changed a parse state or advanced the job.
- **Research and trace decisions.** None needed.
- **Stop conditions.** Stop if the test failed on `main`, because that would be a defect to fix first. It passed.

## Change

`a_parse_step_replay_returns_its_receipt_and_commits_nothing`: with a one-file limit and two files, the first Step parses one file. The same Step under the same key returns the same job, and the parsed symbols and every `source-parse` state are unchanged. The next Step then parses the waiting file and moves to Verify.

## Evidence

Windows 11, one run each.

| Command | Result |
|---|---|
| `cargo test -p rrd-engine --lib a_parse_step_replay` | 1 passed |
| `cargo test -p rrd-engine --lib parse_step` | 9 passed |
| `cargo clippy -p rrd-engine --all-targets -- -D warnings` | exit 0 |
| `check_selected_tests.py`, `check_code_shape.py` | OK |

## Not proven

The Parse test module is compiled on Windows and Linux only, because Parse has no supervisor on macOS (POAM-085). Linux is for the candidate gate.

## Roadmap and POA&M

No status change.

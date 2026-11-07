# Comparison lab: restore the existing evaluation command

**Status:** active draft evidence; no gate completed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/comparison-lab-default-eval-command`
**Owner:** OBJ-10/OBJ-11, J-04 and PR #253

## Package and review

Candidate PR #253 head `400633f113d1309db33a22e4eb30983561c1d619` added a
second binary to `rrflow-eval`. Candidate CI attempt 2, run `42313109731`,
verify job `128223262048`, failed when `cargo run --locked -p rrflow-eval --
verify eval/results/2026-08-18-summary.json` could not choose between
`rrflow-eval` and `rrflow-lab`. The PR description was updated with the
observed failure and fix before the source edit.

Read `crates/evaluation/rrflow-eval/Cargo.toml`, the failing workflow command,
and matching commands in `eval/README.md`. The package manifest now explicitly
sets `default-run = "rrflow-eval"`, preserving the existing evaluation CLI;
lab invocations continue to select `--bin rrflow-lab` explicitly. No engine,
workflow gate, binary behavior, data or version changed. No implementation was
moved or removed.

## Oracle and checks

On the unmodified candidate, `cargo run --offline --locked -p rrflow-eval --
verify eval/results/2026-08-18-summary.json` failed with the same ambiguous
binary error. With the manifest change, the exact command passed and reported
`trials=8 successes=8 regressions=0`. This verifies the historical summary; it
does not prove an agent comparison or new product capability. The explicit lab
binary validation and repository documentation/navigation checks are recorded
below. No external research or new runtime traces were necessary for this
Cargo dispatch regression.

## Change checklist and remaining evidence

- [x] Recorded owner, current/target behavior, scope and failure before editing.
- [x] Reproduced the actual CI failure and restored the established CLI.
- [ ] Candidate CI must pass at the corrected head, followed by merge-queue CI.
- [ ] #258 must land before #253; its candidate is independently running.

This correction changes no roadmap or POA&M status. Alpha, C-07, current
real-project reasoning quality and supervised training are not established.

`cargo run --offline --locked -p rrflow-eval --bin rrflow-lab -- validate
--manifest eval/lab/vector-small.json` passed and printed `manifest is valid`.
`python -X utf8 scripts/ci/check_documentation.py` passed (980 document
statuses), `scripts/knowledge/test_navigation.py` passed six tests, and
`scripts/ci/build_execution_inventory.py` followed by `--check` passed
with 2,433 records. `git diff --cached --check` passed before this addition.

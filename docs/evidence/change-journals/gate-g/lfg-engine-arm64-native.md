# LFG engine admission: Linux ARM64 focused evidence

**Status:** active focused platform evidence; qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/lfg-engine-arm64-native`
**Owner:** OBJ-12 / G-01a, G-01b and G-02 prerequisites / POAM-009
**Pull request:** [249](https://github.com/rrflow/rrflow-development/pull/249)

## Scope and baseline

This follow-up records focused native-platform evidence for the existing
[admission candidate](lfg-engine-admission-checkpoint.md), without changing
engine behavior or advancing a gate. The tested commit is
`7aadf8fbc6f9bdee16b646eb646885ff0ccb69aa`, tree
`86507739063d28c44c98630e381067296b1c12ea`. The PR brief was amended before
adding this record and regenerating navigation. The tests ran in an isolated,
detached clean checkout on the user's `clyffy-01` server; the Connectome
submodule was fetched at its pinned revision. The host reports `Linux aarch64`
and `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.

The existing candidate is meant to bind an optional bounded LFG advisory to
an engine-owned attempt. This evidence must not be interpreted as a real LFG
or SWAG service test, a benchmark, installed-product validation, candidate CI,
native capacity or soak proof. The synthetic controller and analyzer in the
flow test exercise HTTP composition and retained state; they do not exercise
model quality or prove service identity. A project-source edit during the
call remains untested. No code was adapted from external projects for this
evidence-only follow-up.

## Commands and results

The first `rrd-contract` attempt used `--offline --locked` and stopped before
compilation because the server's local Cargo cache did not contain locked
`gix-config 0.62.2`. The retry allowed Cargo to acquire locked build inputs;
it did not introduce an install-time or runtime fetch. Both successful commands
used the pinned toolchain, two low-priority build jobs and the exact checkout:

```sh
env CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 nice -n 10 \
  cargo +1.99.1 test --manifest-path /home/hades/rrflow-ci-probes/pr249-7aadf8fb/Cargo.toml \
  --locked -p rrd-contract --test endpoint_policy
# 5 passed; 0 failed

env CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 nice -n 10 \
  cargo +1.99.1 test --manifest-path /home/hades/rrflow-ci-probes/pr249-7aadf8fb/Cargo.toml \
  --locked -p rrd-engine --lib reasoning_budget::lfg_flow -- --nocapture
# 3 passed; 0 failed; 0 ignored; 340 filtered out
```

The engine command was repeated after the first build because its terminal
output was unavailable; the displayed result above is from the repeat. The
three cases cover synthetic HTTP composition and reopen, configuration change
while the analyzer responds, and a malformed controller response with the LFG
checkpoint retained. They do not cover a project-source edit or the entire
engine suite on Linux ARM64. Neither the Windows/macOS installed build nor
candidate or merge-group CI was run for this follow-up. PR 246 integration,
real-service evidence and the C-07 capacity and soak work remain open.

## Review and accounting

Read in full for this follow-up: the prior admission journal and
`crates/authority/rrd-engine/src/engine/tests/reasoning_budget/lfg_flow.rs`;
the linked journal template and PR brief were also inspected. No runtime
symbols or trace behavior were changed. Existing attempt readback is the
diagnostic surface exercised, and the two-job low-priority build limits
resource contention; neither is a production latency claim. The first
navigation generation rejected this record because its status metadata was
missing; after adding it, documentation policy required the status to identify
the record as active or historical. The status was corrected and navigation
regenerated. Generated navigation is a discovery projection, not a new
authority.

After correction, `python scripts/knowledge/render_navigation.py --check`
and `python scripts/ci/check_documentation.py` passed. An inventory check
initially reported drift because inventory generation ran before the record
and index were corrected; it was regenerated after the final edits.

Change checklist: recorded starting revision, bounded scope, test command and
results, offline failure, unchecked evidence and owner; reviewed the added
record and generated navigation in the PR diff. No implementation was moved,
deleted or replaced. No roadmap or POA&M status changed. This is local server
evidence for the named commit, not a CI run or merge receipt.

# Controller and installed Inventory composition

**Status:** active integration evidence; native object-publication failure blocks qualification
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-inventory-integration`
**Owner:** G-01b / G-02 / D-03 / POAM-006; PR206 driver: active SWAG session

## Baseline, scope and retained implementation

PR206 received its integration brief before the ordinary merge of
`27e87749cb011828a35c8f0e1a9fb88a9a996012` into controller source
`71d53e1e86e17bcc3663dd4d82a7f03451b991dd`. The incoming branch contains PR224/226
and PR227's installed startup evidence; public main at observation is
`f256c8e1337d85fa01f2a0c61e895a4d213163fa`. No dependency is declared landed.

The three conflicts preserve both engine feature/dependency sets and public Rust
exports, and use the installed Python test's actual current job revision. The
remaining CLI credential/proposal composition, engine module declarations,
contract exports and conformance fixture merged without semantic replacements.
The incoming SDK fixture identity is confirmed by the combined planner's
read-only generation check; it is not an invented observed result.

Controller `endpoint_proposal`, capacity and shared reasoning-budget owners
match their controller parent. Installation, Inventory publication/reuse and
attunement tests match the incoming parent. Existing engine authority remains
canonical; neither implementation is replaced by a client lifecycle. Prior BF16,
desktop, package and installed Inventory receipts retain their original source.
This merge alone cannot establish their joined installed runtime behavior.

Reviewed AGENTS/README ownership, Gate G/I, LFG/SWAG and project-command contracts,
change-authoring/CI procedures, the complete three conflicted files, affected
auto-merge hunks, proposal/capacity/attempt owners and Inventory publication/read
boundaries. The prior journals retain their implementation reviews. No new
external algorithm or runtime tracing is required for deterministic integration.
The explicit stop condition forbids hiding a new functional repair in this merge.

## Validation and first failure

The dedicated `target/inventory-27e87749` uses two build jobs and disables dev/test
debug information. The resolved staged source tree before evidence edits is
`e32836491a42c377a878b91978f97a9aca341d1f`.

Passed:

- `cargo test -p rrd-engine --all-features --lib reasoning_budget --locked`:
  27 passes, including guarded controller completion and uncertainty.
- The same command with `attunement`: 34 passes, two existing ignored helpers;
  includes large/partial Inventory, startup and ordinary-edit successor behavior.
- `cargo run -p rrd-engine --example refresh_sdk_installation_identity --all-features --locked -- --check`:
  passed without installation or fixture changes.
- Generated parity: 71 public operations and seven signal projections; workflow
  policy, installed Python test Ruff lint/format and initial whitespace checks pass.

`cargo test -p rrd-engine --all-targets --all-features --locked --no-fail-fast`
failed: 289 library passes, one failure and four existing ignored helpers; all
63 integration tests passed, including source closure and 32 architecture checks.
The failing `concurrent_builds_with_one_key_publish_one_generation` unwrapped
`Vector("object store: Access is denied. (os error 5)")` at quantization_replay.rs:315.
Its source and object-store implementation were not changed by this integration.

The validation chain stopped on that failure. Remaining contract/inference/client
suites, strict Clippy, installed CLI/SDK validation and a combined source-identified
primary have not passed. The log SHA-256 is
`b4ad363794155d2f63d521eb55bac186a570834d1afc38fd886e6e63b17ac609`.
A single focused run with backtraces passed; it does not clear the full-suite
failure or establish a cause.

## Publication-race investigation and handoff

Read-only inspection found `LocalObjectStore::publish_stage` checks destination
existence and then calls replacing `rename_to`. A concurrent winner can appear
between those actions. A separate native Windows probe held an existing winner
without delete sharing: replacing MoveFileExW failed with code 5; non-replacing
MoveFileExW returned code 183 (already exists), preserving both files and the
winner's digest. This reproduces a relevant primitive hazard, not the original
engine error's exact operation. The existing anchored `rename_new_to` primitive
is available for a bounded owner-level characterization and repair.

Keep the merged source intact and draft. Diagnose and fix any confirmed object
publication race in its own persistence package before refreshing this branch;
do not remove the concurrent oracle, weaken sharing, or retry until green.
No code repair is included in this evidence journal. Correctness probe bytes
are excluded from training and routing data. Native resource/performance and
model execution were not measured here.

Raw records remain at
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/` under
`controller-inventory-*` plus `object-publication-native-probe.json` and its
retained disposable directory. The validation record retains command exits,
source tree and log digests. The original full-suite failure remains intact.

This increment adds the linked journal, the verified deficiency in POAM-035 and regenerated navigation alongside the
ordinary dependency merge; no historical journal is rewritten. Documentation,
navigation and execution-inventory checks accompany the final merge record.
No roadmap/POA&M acceptance closes. PR224's candidate passed all 23 jobs and
entered the protected queue; merge-group run 41210074230 is separate pending
evidence. Worker execution, correlated outcomes and reviewed learning remain open.

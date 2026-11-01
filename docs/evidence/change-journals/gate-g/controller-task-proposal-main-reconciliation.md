# Controller proposal: reconcile landed engine contracts

**Status:** active local reconciliation evidence; protected landing and native failure resolution pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-task-proposal-main-reconciliation`
**Owner:** G-06 controller proposal admission; shared native failure at C-07 / POAM-035
**Pull request:** [206](https://github.com/rrflow/rrflow-development/pull/206)

## Bound change and traceability

Starting controller revision `54fb845dffd702e1504c7f9a0cc2a3c5757d2bf2` ordinarily merges landed public main `6fe8b7590d913beda9b009551e5d2d2644e406d9`. The driver is the controller implementation session; packaging PR212 depends on this candidate. The PR brief binds this refresh before the restarted merge. No history rewrite, routing policy change or new runtime feature is included.

Git reported 19 conflicts: the contract OpenAPI digest and 18 generated SDK/protocol projections or digest citations. The semantic endpoint-proposal and storage-cursor implementations merged automatically. The contract test merge retains the controller cases and adds main's three bounded/typed storage cursor cases. All five SDK generators were run against the combined canonical contract, followed by the repository surface generator. The resulting OpenAPI SHA256 is `52b7e85144b8b7b4e75291033cf151a0674c97f87fff2054d66f23c97a4bfb6d`. Digest citations follow that projection. No hand-authored generated models or competing owner were introduced.

The original proposal implementation and tests remain in their canonical engine, contract and inference modules. Incoming storage behavior and native diagnostics retain their landed main owners and tests. The installed development and optimized bundle receipts still prove only their original revisions; this merge does not inherit installed qualification.

## Review and engineering decisions

Reviewed the repository instructions, README owner entry points, change-authoring procedure, CI contract, Gate D and Connectome boundaries, the existing proposal journals and PR brief, the shared native-create diagnostic record, all conflict hunks and the auto-merged public contract tests. Full source reviews from the implementation remain in the linked historical journals; this refresh does not claim a new full audit of unrelated incoming main code. Git provides the incoming path inventory and merge history; generated parity checks validate replaceable projections.

Created this journal and its generated index entry; changed only conflict reconciliation projections/citations in addition to the ordinary incoming main changes and generated inventory. No implementation deletion or relocation. Research/source adaptation is unnecessary for deterministic regeneration. Preserve the actual CI log for diagnosis; no new performance measurement, model call, training or credential operation is needed. Stop on unexpected semantic loss, failed relevant checks or any requirement to weaken protected CI.

## First failure and retained evidence

[Protected run 40982529726](https://github.com/rrflow/rrflow-development/actions/runs/40982529726), head `83cef0fef520e16efe2f270df94fa7a39162a017`, failed macOS job `123825754303`, step `Require repeated APFS exhaustion and recovery`: `hdiutil: create failed - Resource busy`, exit 1. This is an APFS image-creation failure, not a failed controller routing assertion. Its cause remains unproved. New main carries the [native create diagnostics](../gate-c/c-07-native-create-diagnostics.md), which are preserved, not described as a root-cause repair. PR215's native capture work does not itself clear this distinct failure. C-07 / POAM-035 remains the owner; no acceptance status changes.

The first diagnostic command failed with a Windows console encoding error before its intended PR brief update. An initial untouched merge was aborted; the corrected brief was published, then the merge restarted. This procedural error is retained. No conflict edit preceded that restart.

## Acceptance and limits

- `cargo test --locked -p rrd-contract -p rrd-inference`: passed, including merged public contracts and inference adapter cases.
- `cargo test --locked -p rrd-engine --lib reasoning_budget`: 27 passed.
- `cargo test --locked -p rrd-engine --lib endpoint_configuration`: 5 passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked -p rrd-contract -p rrd-inference -p rrd-engine --all-targets -- -D warnings`: passed.
- Five SDK generation commands and `python scripts/ci/check_generated_surfaces.py --write`: passed.

Local command logs are retained under the operator's `controller-main-integration` evidence directory dated 2026-10-30. TypeScript `pnpm check` passed generation parity, lint, type checking and all 6 tests. Python, Go, Java and .NET generator `--check` commands passed. `python scripts/ci/check_generated_surfaces.py --check` passed all 63 HTTP operations and seven signal projections. The first documentation policy check rejected this new journal's missing active/historical status marker; the marker was corrected. The next check identified the index made stale by that status correction; navigation was regenerated before the final checks. Navigation and inventory generation passed; documentation, workflow and inventory checks passed. The staged whitespace check then exposed Windows CRLF in this newly authored journal; it was normalized to LF without changing captured receipt bytes. Final whitespace and inventory checks are retained with the command logs. No new installed build, live endpoint evaluation, native macOS qualification or full candidate pipeline was run for this refresh. Keep the candidate draft while the shared native failure is unresolved; green local checks do not replace protected landing. The existing engineering bundle remains unchanged and unqualified. Selected-worker execution, ordinary-edit successor planning, public UI/outcomes, model qualification and public release remain separate open outcomes. Roadmap/POA&M acceptance is unchanged.

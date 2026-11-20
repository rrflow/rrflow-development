# Branch recovery and project-isolation evidence: 2026-11-20

**Status:** active bounded recovery evidence; no runtime gate closed
**Coordinate:** `rrflow://rrflow-instance/data/change-journal/branch-reconciliation-20261120`
**Owner:** evidence for the owner's branch-recovery request, under [session recovery](../../../operations/agent-sessions.md#absorbing-old-work)

## Brief and baseline

This package supplements the immutable [November 17 review](official-remote-branch-absorption-20261117.md), which ran no tests. It records new behavior checks, corrects its overbroad inventory claim, and separates retained implementation from the unfinished project-sharing journey. It is a repository prerequisite for OBJ-07 and H-06/H-07/H-10; [POAM-011](../../../poam/rrflow-1.0-alpha/poam-011.md) remains open. No runtime implementation, public contract or gate status changes.

The source examined and tested is RRFlow `ab6cff7bf4b28bbac1d7ad7efcbf03c85e311787`, tree `3831277715395c3aa1f279c86d39f444861ab05e`, with Connectome `e780304b39afe8becee4beca63c1f8566d0ba5fd`. This package starts at merged main `7364163181cfa06e06d7a2bc865a6bd6e66e15ea` under [PR443](https://github.com/rrflow/rrflow-development/pull/443). The branch brief precedes repository edits. Its scope is this record, its generated index and the POAM-011 evidence link. Stop on a changed tip, unmatched behavior, failed check or unapproved official-remote mutation.

## Remaining historical tips

The remote is `https://github.com/rrflow/rrflow.git`, named `origin`; it is the official repository, not the development push target. `git ls-remote --heads origin` found these six non-main refs. Recovery file `D:/.usr/Temp/rrflow-selective-recall-audit-20261120/official-six-20261120.bundle` contains all six complete histories; `git bundle verify` passed. This is a recovery bundle required by the session standard, not parked implementation work or an archive ref.

| Ref | Exact tip | Disposition supported here |
|---|---|---|
| `agent/connectome-temporal-runtime-visualizer` | `63bc2331fcbc5c71aaa123cd44aedeb65d5c7b65` | Authority catalogue, validation, receipts and recovery behavior survives in `rrd-engram`; five focused tests and the 23-test owning suite pass. This tip is already an ancestor, but ancestry is not the behavior proof. |
| `backup/agent/c07-backup-scheduling-20261029-1024` | `2665f99110b0407eb3f38c573185de91e835ca61` | Failure journal is identical after newline normalization. Added growth/readback requirements remain in POAM-035 and C-07. The old blanket prohibition on UI work was superseded by the later owner implementation order. No separate runtime implementation is added by this tip. |
| `backup/agent/c07-run-progress-20261029-1024` | `65385004d8f9d01401606a3918fe45025c1bfb74` | Empty intent commit plus a merge; its complete tree equals already-integrated `d0c446df1d7244212fe56e9daff8c82ec98d6d4b`. No independent source behavior to merge. |
| `backup/agent/rrflowdb-postgres-identity-20261029-1024` | `f661cc965093f9ed4b3ae7a222da35c65599b43e` | Documentation carried; the retained identity journal is identical after newline normalization. This does not implement the SQL protocol. |
| `backup/agent/rrflowdb-postgres-plan-20261029-1024` | `d134734bd765545475d5d6020b22749feae8045e` | Three journals and protocol index match after newline normalization; the protocol contract differs only in six estate-to-engram substitutions. The inventory mapping line remains. Wave order is explicitly superseded; runtime conformance remains H-08. |
| `codex/consolidate-project-structure-and-context` | `73c75a852b0e576d135cda543103105cb5aa93ef` | Inventory digest/order/staleness behavior is carried and tested. The old Markdown/`--output` convenience command is not carried. Unified reasoning entry point and template bindings remain open. The old Python implementation is not restored beside the canonical engine. |

The earlier review's phrase **strict superset** is incorrect as a claim about all old generator capabilities: its own export row is missing. This record replaces that claim with the narrower mapping above; the historical receipt is not silently rewritten. Existing documentation and tests do not prove the full original product requirements were absorbed. No old branch is merged merely to manufacture ancestry or resurrect superseded architecture.

Official-ref deletion requires the owner's deletion-only exception to AGENTS.md's prohibition on pushing to `origin`. The request names exactly these six refs, excludes official `main` and source promotion, and remains separate from development branch cleanup. No official ref has been deleted by this record.

## Source mapping and read coverage

All current paths below are bound to the source revision above. Historical paths are bound to the exact tip in the table. This is a bounded comparison, not a whole-repository line review.

- Authority tip: read the full old `crates/rrd-estate/src/authority.rs` and `tests/authority_catalogue.rs`, their current counterparts in `crates/authority/rrd-engram/`, and the old/current reconciler recovery test comparison. Read the added `lib.rs` behavior and contract/test patch. Current `src/repository.rs` lines 88-145, `src/document.rs` lines 156-212, `src/validation.rs` lines 1-78 and `src/model.rs` lines 260-280 locate persistence, snapshot, validation and required authority ownership. Current `crates/transport/rrd-contract/src/engram_authority.rs` and `crates/authority/rrd-engine/src/engine/engram_control.rs` were read in full. Changes are naming, required pre-release fields, version and canonical control/store ports; the catalogue algorithm survives. Read the historical generated SDK patch: five endpoint files change only the OpenAPI digest; the TypeScript projection adds four authority blocks. A whitespace-normalized comparison finds one unique added block, present unchanged in the current projection in all four cases. The current generated-surface and SDK conformance checks are separately qualified by PR440's gates; this is not a full SDK implementation review.
- Inventory tip: read all of `scripts/project_shape.py`, `scripts/test_project_shape.py` and `consolidation-contract.md` at the old tip. Read all current `crates/authority/rrd-engine/src/engine/installation/inventory/scan.rs`; read inventory `tests.rs` lines 312-349, `engine/tests/attunement/replan.rs` lines 1-92 and `engine/project_map.rs` lines 260-375. Sorted bounded capture, per-file/aggregate identity, role recognition and stale-plan refusal have canonical destinations. This does not prove every old exclusion or export presentation is equivalent.
- Documentation tips: read their added patches, compare the retained journals, protocol contract and index, inspect the owning POAM/gate paragraphs, and retain the prior row-by-row disposition rather than creating another global implementation inventory. No PostgreSQL runtime implementation is inferred from these planning commits.
- Current project placement: read `crates/operations/rrd-kubernetes/src/controller.rs`, `spec.rs`, `validation.rs`, `tests/contract.rs`, and `resources.rs` through the complete `desired_resources` implementation (lines 1-218); read `eval/lab/pod.sh`. Read Connectome `src-tauri/src/rrd/engram/lifecycle.rs` lines 1-190, the complete `project_registry_keeps_parallel_engrams_and_sessions_isolated` test in `client/process/startup.rs`, the multi-engine process test in `client/process/many.rs`, and `src/features/runtime/shell/workspace.tsx`. This does not review every process-manager method or prove a Kubernetes deployment.
- Owners read: AGENTS.md, README.md, session standard, change-authoring procedure, November 17 review and POAM-011; relevant project-link, Gate H placement and DevPulse sharing/isolation sections. Unreviewed remainder includes the full older Connectome cloud deletion, all inherited UI bodies and every historical ancestor of these six tips. The existing child absorption ledger and visual parity record remain narrower independent records.

For the deleted cloud path, only three additional files were read in full at Connectome `a3e00eecfa319e29d20e6be0f5e9aad05b2707c7`: `src/cloud/hooks/instances.tsx`, `queries/instances.tsx` and `api/endpoints.tsx`. They poll organization/instance API listings when authenticated and select configured service endpoints. They contain no two-party project-link consent or stamped federation. This is not a full cloud-source absorption review; the remaining deleted files still need behavior mapping before replacement claims.

## What project isolation actually proves

The native two-project driver installs two independent projects, uses separate credentials/endpoints, rejects invalid session rebinding, writes data/vectors to one project and proves absence from the other, then disconnects/stops one without stopping the other. It passed on Windows. This proves bounded data/session/process ownership behavior for that fixture.

The retained authority catalogue has a separate integration limit. `rg -n 'apply_authority\(|AuthorityMutation' crates --glob '*.rs'` finds the mutation method in `rrd-engram/src/repository.rs` and calls only in `tests/authority_catalogue.rs`. The inspected engine admin enum exposes lifecycle/backup/recovery operations, not that mutation. Connectome's `src/features/runtime/cluster/cluster-view.tsx` and `authority.tsx`, both read in full, fetch and display the catalogue through `client.engram()`; they do not provision its resources. This is retained persistence code with component proof, not a joined resource-management capability. Integrating it must also obey the current topology owner rather than preserving a redundant authority hierarchy.

The Kubernetes renderer authors a one-replica StatefulSet, retained volume, non-root/read-only-root security context, disabled service-account token, dropped capabilities, CPU/memory limits and a NetworkPolicy. The reconciler applies them and derives readiness from the StatefulSet's ready replica count; TCP probes are not authenticated engine readback. `eval/lab/pod.sh` is a benchmark container helper sharing the host Docker socket, not the product's per-project pod manager.

There is also existing native containment below attunement: read `crates/authority/rrd-engine/src/engine/attunement/linux_supervisor.rs` lines 1-125, `windows_supervisor.rs` lines 266-315 and `parse/worker.rs` lines 116-185. The parser worker uses a Windows Job Object with process/job memory limits and kill-on-close, or Linux `RLIMIT_AS` with parent-death handling; unsupported native platforms refuse Parse. These are parser-worker boundaries, not proof of whole-project CPU, filesystem or noisy-neighbor isolation. The complete supervisor implementations and their platform qualification were not newly reviewed or rerun in this recovery package.

Neither source presence nor the two-project test proves enforced noisy-neighbor/resource isolation, idle suspension/resumption, an authenticated deployed Kubernetes journey, opt-in context sharing, or account synchronization. H-10's two-party link records, revocation and two stamped audited reads are still specified but not implemented in the inspected engine. H-07e owns the friendly independent context-sharing and physical-placement controls. Connecting an HTTPS endpoint is not synchronization. These existing POAM-011 gaps remain open.

## Acceptance and retained output

Windows, four Cargo jobs, existing target `D:/.usr/Temp/rrflow-selective-recall-audit-20261120/target`; no existing cache or user process removed. Logs are in that session scratch directory. The code tested did not change during this review.

| Command / driver | Result | Log |
|---|---|---|
| `cargo test --locked -p rrd-engram --test authority_catalogue --test reconciler_recovery` | 5 passed, 0 failed/ignored | `absorption-authority-recovery.log` |
| `cargo test --locked -p rrd-engram` | 23 passed, 0 failed/ignored; doc tests 0 | `absorption-engram-suite.log` |
| `cargo test --locked -p rrd-engine exclusions_and_creation_order_do_not_change_the_digest_or_admitted_count` | 1 passed | `absorption-shape-order.log` |
| `cargo test --locked -p rrd-engine stale_attunement_can_be_replanned_without_reinstalling_or_losing_receipts` | 1 passed | `absorption-shape-staleness.log` |
| Connectome `project_registry_keeps_parallel_engrams_and_sessions_isolated`, installed sidecar, `--ignored --exact` | 1 passed, 16.54 seconds; fixture executable SHA-256 `fefdf228cc16999b1d3f0074700ec6b92318e590379f7865be9c2fecb013c365` | `startup-project-isolation.log` |
| `cargo test --locked -p rrd-kubernetes --test contract` | 2 passed; deterministic secured rendering and unsafe-spec rejection before Kubernetes I/O | `absorption-kubernetes-contract.log` |
| `git bundle verify .../official-six-20261120.bundle` | Six exact refs, complete history, valid bundle | Command result retained by this session |

No live Kubernetes enforcement, BEAM comparison, cloud sync, cross-project shared read, full manual desktop journey or competitive benchmark was run. No external research or source adaptation is needed for these Git/source/test comparisons. Existing command logs supply debugging evidence; no product telemetry or new measurement code is added. PowerShell rejected an unquoted tree expression and one quoted GitHub search; quoting the tree expression and listing/filtering issues corrected the inspection commands. Those failures changed no repository state and were not test failures.

## Implementation branches landed before this package

| Package | Candidate / merge-group runs, both green | Merge revision |
|---|---|---|
| [Connectome PR107](https://github.com/rrflow/connectome-development/pull/107) | `44808683576` / `44808724615` | `7cbe1f10b248e6477dc2c5b46581a639a6337344` |
| [Connectome PR108](https://github.com/rrflow/connectome-development/pull/108) | `44814689370` / `44815373768` | `e780304b39afe8becee4beca63c1f8566d0ba5fd` |
| [RRFlow PR440](https://github.com/rrflow/rrflow-development/pull/440) | `44816918046` / `44819008868` | `7364163181cfa06e06d7a2bc865a6bd6e66e15ea` |

For each merged source branch, the source/merged tree comparison was empty and a recovery bundle was verified before local deletion. The server deleted each merged remote branch; fetch/prune and `ls-remote` confirmed their absence. Bundles in the session scratch directory are `connectome-merged-107.bundle`, `connectome-merged-108.bundle` and `engine-merged-440-20261120.bundle`. The unused local Connectome `fix/startup-workspace-navigation` tip `eb31a718f73c3588fbdb210567d3d4aa518ae6de` had no commit outside main and was preserved in the first child bundle before removal. This is not evidence that every inherited capability is native.

After PR440 cleanup, both development remotes had only `main`, both worktrees were clean, and `python scripts/ci/check_session_end.py` passed. This documentation package then opened its own short-lived branch and draft PR before editing files. The final cleanup of that branch follows its own green merge. The official six refs are a separate unresolved deletion permission, not hidden development branches.

## Change checklist and integration

- Current/target behavior, owners, exact scope, failure oracle and stop conditions are in the PR brief before edits.
- Source adaptation: none; no code is copied back, deleted or moved.
- Created: this journal. Changed: POAM-011 evidence link. Generated: nearest journal index. The complete authored journal and changed POAM content were reread, and the generated index diff reviewed before commit.
- Earlier source/journal evidence is linked, not rewritten. No objective, roadmap or POA&M status changes.
- `python scripts/ci/check_documentation.py` passed (1,281 document statuses, 1,277 classified coordinates); `python scripts/knowledge/render_navigation.py --check` passed (21 indexes, 1,278 nodes, 4,487 edges); `git diff --check` passed. `python scripts/ci/build_execution_inventory.py` and its `--check` passed with 3,686 records. Candidate and merge-group gate results are pending on PR443 at authoring; their exact revision/run identities are retained by that PR. A skipped draft gate is not acceptance.
- Official deletion stays blocked on the narrow requested exception; development cleanup follows verified merges. The next product work remains the installed journey and governed sharing at their existing owners.

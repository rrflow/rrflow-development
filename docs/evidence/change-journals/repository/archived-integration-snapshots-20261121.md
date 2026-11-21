# Archived integration snapshots

**Status:** active integration evidence; qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-integration-snapshots-20261121`
**Owner:** repository development gate

## Outcome and ownership

Join original saved integration snapshots into the current canonical engine, completing source recovery rather than claiming new alpha capability. Driver: this repository recovery session. Baseline e163ad7db0330f7a88dec23841ab4b9ff5dc798c. Owner: repository development gate; Gates D installation/attunement, F authenticated cold history, H vectors and reasoning. PR488 owns the separate read-path source joins; use queue ordering, without editing its kernel/read implementation.

## Change brief

Sources: aa169c31fbcdbee65d0984824d2b7b6dd92c024f and d7ef0a13bc14fafde4267aabd4da0ba50d7ef2e8 (identical engram snapshot trees); 72102d29ba40f39e7f363033994483ce57071b4d and 8caf95962b76ece49b2bed21a71872c7df6bf5bf (identical conflict-diagnosis trees); a2b9021a1951d9fa17fc00357387067c31e92afa and 2be5aa80509dea4dc7ca190ac6514f2212ab3718 (identical F06 trees, including 1296c3837a02b595af8b8a629f02fc4045831ecb); f902b24652711f2775854bef9ca5b86040f44963 (parser integration); a064b6d5fe865f111b39ab2600e684f61246e7c0 (read-path/reasoning integration). None has an untracked-file third parent. Preserve each original source as merge parent; inspect actual conflict and automatic merge deltas before keeping any change.

Exact scope is source consolidation across the original changed paths: rrd-engine installation/attunement/cold_history, endpoint/task assessment and reasoning resume; rrd-store/rrd-query cold history and rrd-vector read path; CLI installed lifecycle; public catalogue/SDK projections; CI/evidence and engram rename. Current split modules remain canonical. Preserve current child gitlink 36d388e919c27de7b72fbccb3d7e7af7bbb71030. Do not restore retired monoliths, provider permission overrides, old PLAN/STATUS authorities, generated runtime data, obsolete API identities or benchmark artifacts.

Already reviewed before this package: source commit/path inventories and original added-test identifiers; current installation/configuration_and_adoption.rs 495-590; current endpoint_policy and endpoint_worker identity/approval assertions and endpoint_selection actual choice/abstention assertions; source cold-history implementation in earlier PR480; read-path behavior in PR488; parser behavior in PR485. This is not a full line review of all broad snapshots. The former rejection of an inherited .rrflow folder is superseded by tested in-place preservation; old projected endpoint configuration identity is superseded by the actual captured identity. Missing original test identifiers were traced to these changed behaviors and engram renaming, not simply discarded. Review each unresolved delta before resolution, carrying reusable behavior into existing owners.

## Proof and limits

Smallest oracles: existing installed lifecycle/reopen, cold_history tests, contract endpoint_policy/endpoint_worker/endpoint_selection tests, source-bound assessment and reasoning resume tests, architecture. Run focused suites selected from actual final runtime changes, then documentation/navigation/whitespace; candidate and merge-group CI remain required. Use D:/.usr/Temp/rrflow-root-target with four jobs; no source edits during Cargo. No external research, comparison workload, instrumentation, new memory architecture or maturity claim is needed. Stop on unexplained unique behavior or failing assertions and repair before readiness. Retain source and exact evidence; no force push, history rewrite or gate weakening.

## Evidence and handoff

Journal: docs/evidence/change-journals/repository/archived-integration-snapshots-20261121.md; regenerate nearest index. Record source joins, resolutions, all changed paths, failures and actual commands. After green queue merge delete the branch, then continue checkpoint-source reconciliation and remaining source objects. This package does not claim all archives are recovered or the screenshot installation problem is fixed.

## Source joins and resolution evidence

| Source | Merge | Canonical destination |
| --- | --- | --- |
| aa169c31fbcdbee65d0984824d2b7b6dd92c024f | 1e4d96009917169913c275495cf6ec6257c72ce2 | Current engram modules, installed preservation, reasoning resume; missing fixture assertions carried below |
| d7ef0a13bc14fafde4267aabd4da0ba50d7ef2e8 | d2670900 | Same source tree; same resolutions, distinct original stash identity retained |
| 72102d29ba40f39e7f363033994483ce57071b4d | 605c59d9 | Current transaction/source-change errors, inventory publication and typed SDK replan |
| 8caf95962b76ece49b2bed21a71872c7df6bf5bf | c675734 | Same source tree and resolutions |
| a2b9021a1951d9fa17fc00357387067c31e92afa | 482226164 | Current cold_history modules and stamped readback; includes original 1296c3837 preview |
| 2be5aa80509dea4dc7ca190ac6514f2212ab3718 | 2e8455208 | Same source tree and resolutions |
| f902b24652711f2775854bef9ca5b86040f44963 | 47b3caf20 | Current parser, reasoning worker, vector identities and client validation |
| a064b6d5fe865f111b39ab2600e684f61246e7c0 | 85e17deb023182b4eb559d4f2154f69986c45043 | Current HNSW isolation regressions and source-bound assessments |

These joins initially leave the implementation tree unchanged. That alone is
not behavioral consolidation proof: the missing fixture assertions below are
a concrete correction required before this package is ready.

Current modules replace incoming monoliths in engine attunement, diagnostic,
installation, flight, restore_publication, vector_catalog; engram backup_job,
local_process and recovery; query execute/graphql/plan/syntax; vector kernels;
core runtime/schema/trace; store semantic_commit/versioned_read/keyspaces;
contract attunement/deployment/diagnostic/websocket; and server websocket.
Current split tests replace old public_contract, attunement_contract,
http_process, installed_lifecycle and workspace_architecture files. The old
rrd-estate test path maps to rrd-engram/tests/engram_authority.rs. Generated SDK
catalogues retain current executable-owner output, not obsolete identities.

Specific reviewed deltas retained: repeated-work resume support and its public
client re-stop oracle; publication source rechecks, publication digest and
successor advancement; in-place installation ownership proofs; source-file
projection verification; current typed stale-transaction/source-change recovery
details; assessment capacity accounting and request identity validation;
collection-derived HNSW identifiers. The automatic merges duplicated the typed
attunement test, stale-source response test, two HNSW isolation tests, resume
module and Parquet dependency: retain their single existing definitions.
Original cold_history_oracle.expected_changes is now inside cold_history/tests.rs.
Current cold paging and query tests differ from the source by engram naming.
Keep current default-on cold policy and SDK corpus identity; the old custom
policy preview is not the current bundled corpus configuration. Keep current
completed LFG diagnostic evidence and historical journal headers, not old draft
status or duplicated/misencoded POA&M paragraphs. Inventory-delta's interrupted
refresh oracle remains explicitly ignored under POAM-006; not repaired here.

The source submodule merge failed because old child merge bases were absent.
Current separately integrated child pin 36d388e9 was retained. An intermediate
resolution used cat-file to test path presence, which is wrong for a gitlink;
it staged removal of connectome and .gitmodules. That error was caught before
commit, both restored from HEAD, and recursive submodule update restored the
exact child revision with a clean worktree. No submodule deletion is committed.

## Restored capture provenance assertions

The before-edit statement that the endpoint tests used actual captured identity
was incorrect. Their shared helper rewrote captured configuration bytes and
replaced its digest even in returned receipt bytes. The archived aa169 source
had explicit file-hash and historical canonical-hash assertions, and explicitly
rejected authorization of a derived identity by the recorded allowance.

On 85e17deb plus the new regression, `cargo test -p rrd-contract --test
endpoint_selection captured_receipt_bytes --locked` failed: returned receipt
bytes differed from the saved receipt. This is the red oracle, not a live
provider failure. A first apply_patch attempt was rejected for duplicate target
operations and changed no files.

The shared helper now validates original file bytes and original canonical
configuration identity before deriving current-contract inputs. Only
configuration/allowance/request inputs can be derived; receipts, responses,
observations and hash manifests remain recorded bytes. Tests explicitly prove
that the recorded allowance cannot authorize the derived configuration, and
reject altered file bytes and altered canonical identity. Misleading test names
and assertions now distinguish derived identity from the receipt. Three copied
helpers are replaced by test-only module references to the contract helper;
all affected call sites name read_projected explicitly. No captured file or
runtime code changes, no compatibility adapter, no live provider qualification.

## Verification

On 85e17deb plus the final fixture repair: contract endpoint_policy 7,
endpoint_selection 11 and endpoint_worker 6 passed. The original receipt
regression is now green; both corruption probes passed. Inference endpoint-chat
12 and model-chat 6 passed. Engine reasoning_budget 79 passed, 2 existing tests
ignored; endpoint_configuration 7 and cold_history 14 passed. Installed CLI
lifecycle 17 passed (35.52 seconds), architecture 35 passed. Commands use the
locked workspace and the target/jobs in the brief. Log integration-snapshots-tests.log
is retained in the session scratch directory. No test result from an earlier
candidate is substituted for these runs. No benchmarks or external comparisons
were executed. Full candidate and queue CI are still required.

Review coverage is the original patches, conflict/automatic-delta sections and
specific owner paths recorded above, plus the complete final changed code diff.
Broad historical diffs were truncated; this is not a full line review of every
snapshot. The remaining checkpoints and oversized original history are separate
recovery work, not declared absorbed by these merges.

Final validation: strict `cargo clippy -p rrd-contract --tests --locked -- -D
warnings`, documentation policy, formatting and whitespace passed. The derived
execution inventory initially reported drift; regeneration and its check passed
with 3762 records, without changing a tracked inventory file. Complete final
code diff reviewed. PR488 queue admission has a separate expected-check blocker
tracked in issue #490; this draft will not bypass it or the required gate.

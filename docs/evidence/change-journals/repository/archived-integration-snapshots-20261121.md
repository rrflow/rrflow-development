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

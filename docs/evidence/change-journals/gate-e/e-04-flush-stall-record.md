# Recording storage maintenance stalls

**Status:** active immutable change-journal evidence; E-04 and OBJ-13 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-flush-stall-record`
**Owner:** evidence for recording POAM-056

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: OBJ-13, E-04 and OBJ-10, by recording a verified deficiency. No behavior changes.
starting revision/tree and pull request: 88b753ce on agent/comparison-lab (#253), because POAM-053, POAM-054 and POAM-055 are recorded there and numbers stay contiguous; the pull request holding this journal.
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the description of the pull request holding this journal. The stall had no owning record. It now has POAM-056.
files and symbols read in full: rrd-store rrflow_kv.rs try_maintenance_step and lock, rrd-lsm database.rs maintenance_step_inner and its callers, rrd-server http/server.rs's maintenance worker, rrd-security commit_with_audit, the installed lifecycle's native storage limits.
implementation/evidence paths created, changed, moved, or deleted: created docs/poam/rrflow-1.0-alpha/poam-056.md and this journal, changed docs/poam/rrflow-1.0-alpha.md, and generated the nearest indexes.
research decision and primary-source/adaptation record: none needed.
trace/resource/debug decision (add/preserve/not applicable, with reason): not applicable to a record. The measurements used a per-second reading of the engine's /proc write_bytes and the lab's raw per-search latencies.
first-failure or characterization oracle and result: the runs named in POAM-056, on clyffy-01, one run of each. The raw samples of a COCO run sum to its wall time and hold stalls of 711.0, 640.5 and 562.2 ms at ef-64, and the raised flush target moved the stalls on vector-small.
acceptance commands and exact results: render_navigation.py, build_execution_inventory.py, check_version.py and check_workflow.py pass. check_documentation.py fails: "must hold POAM-001 through POAM-053 exactly once and in order". That rule cannot pass on this branch with or without this record, because it already jumps from POAM-049 to POAM-053, and POAM-050 to POAM-052 are on pull request #258. It has to be rerun once #258 and #253 are in.
surfaced failures and corrections: my first guess was time outside the measured searches, and the raw samples disproved it. My first raised-limit run was refused, because 16777216 is above the ceiling for memtable_flush_target_versions, so the run used 1048576. A monitor column for store files read the engine's working directory and not its store, so it says nothing and I did not use it.
checks not run and reason: a direct trace of a flush's duration under the lock, since moving the flush target moved the stall. Whether audit is the only source of these writes: the run only searches, and a tree without search audit was not measured for writes. Whether flushes or compaction steps cause each stall individually.
remaining known errors and owner: POAM-056 is open. Building segments outside the exclusive lock is a storage package with recovery risk. Whether search audit changes is the owner's decision, and changing it would reduce how often this fires without removing the stall.
full-file reread and diff review: complete for both records.
change checklist: complete. Documentation only; no provider state; no model call.
roadmap/POA&M status change, or explicit none: adds POAM-056 as Open.
pull request, CI run, and merge evidence: recorded on the pull request holding this journal.
```

## Links

- [POAM-056](../../../poam/rrflow-1.0-alpha/poam-056.md)
- [Gate E](../../../roadmap/rrflow-1.0/gate-e.md)

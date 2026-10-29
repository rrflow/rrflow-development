# D-02: correct status candidate inventory after CI failure

**Status:** active correction evidence; fresh candidate CI required
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-status-inventory-correction`
**Owner:** OBJ-07 / D-02a / POAM-006 / [PR146](https://github.com/rrflow/rrflow-development/pull/146)

Candidate `0ce79973a39fe3e8ff1e9d6f05ed7feae437a384` failed the verify job in [run 40539774180](https://github.com/rrflow/rrflow-development/actions/runs/40539774180): `python3 scripts/ci/build_execution_inventory.py --check` reported drift. The job log is retained locally at `target/d02-status-verify-40539774180.log`, SHA-256 `ceb427da52692cb27520831a150cf76851ccff6e87631b1de3e531103f3e6d9`. The integration journal was changed after generating its inventory entry, leaving its byte count and SHA-256 stale. The failed run was cancelled after retaining the log; unfinished jobs are not passes.

This correction regenerates the inventory after this record and all other journal contents are final, then checks the committed file set. It changes no engine, contract, API, test or acceptance threshold. The prior installed HTTP/reopen and contract suite results remain associated with their actual tested candidate, while the corrected head requires fresh candidate CI. Reviewed the failed job log, generated inventory diff and existing integration journal. No upstream adaptation or runtime diagnostic change is involved; source freshness, replan, full project workflow and objective/roadmap/POA&M closure remain open.

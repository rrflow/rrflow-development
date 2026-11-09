# C-07g: integrate indexed reads for fresh native characterization

**Status:** active integration evidence; native qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-indexed-native-candidate`
**Owner:** OBJ-02 / C-07g / POAM-035 / PR139

## Pre-effect brief and source traceability

Before integration, [PR139](https://github.com/rrflow/rrflow-development/pull/139)
was updated with the exact dependency revisions, source/evidence scope,
unchanged acceptance limits, stop conditions and current Codex driver.
Starting head was `9af0abc79b54b4fb0011a69913d1ea8551ae0d98`.
Ordinary merges integrated:

- PR141 `3b28e5b75db461e4bddb8ee88fb0d5da4a56d66f`, merge `f2b68df1`:
  [point-range indexing](c-07g-point-range-index.md), replacing linear point
  candidate enumeration at the existing database boundary. Its 229 passing
  storage tests, strict lint and known development failures belong to that
  journal; they are not re-labelled as tests of this integration commit.
- PR140 `b623bd441d5b83abefcae845f872688767eeab39`, merge `0bafebdf`:
  [APFS fixture settlement](c-07f-apfs-capacity-settlement.md). Native crash and
  disk-full jobs passed on Windows/Linux/macOS. Three 100-cycle reopen jobs
  were still active at integration; candidate CI and normal landing remain
  required.

Production database and new point-index/test files match PR141 byte-for-byte.
Capacity fixture matches PR140 byte-for-byte. PR139's `sustained/corpus.rs`
and `corpus_window.rs` match the starting revision byte-for-byte. A scripted
comparison verified all six paths. The complete source diff from the starting
head contains exactly those three PR141 files and the PR140 capacity fixture;
all other changed paths are journals, their index, POAM appendices and the
generated execution inventory. Existing PR134 proof reuse/trace correction
remains in the candidate. No source is deleted, duplicated or moved.

Both merges conflicted only in POAM-035 appendices and the replaceable file
inventory. All evidence paragraphs were retained. The inventory was resolved
in the index and regenerated mechanically; no generated conflict content was
accepted as authoritative. New integration evidence links from POAM-035 and
the generated journal index. No roadmap/objective status changes.

## Native campaign boundary

This package prepares a fresh exact-source primary and installed-reopen harness
in the existing owned KVM guest. It does not reuse or alter failed estates.
The source is fetched solely from the public development repository with its
pinned submodules. Locked build inputs are reused offline. Operator scripts
were reviewed in full: `stage_c07_proof_candidate.py`,
`prepare_c07_linux_characterization.py`, `prepare_c07_desktop_integrated.py`
and `launch_c07_desktop_integrated.py` under the local ignored `target/`.
They are staging/measurement tools, not hidden engine build inputs or runtime
dependencies. The actual runtime/campaign launcher remains repository-owned
`scripts/ci/run_sustained_capacity.py`.

Unchanged characterization: 4 guest vCPUs, 8 GiB host memory, 2 GiB engine,
64 MiB cache, 100000 records, 500000 edges, 100000 vectors of dimension 384,
102400000 logical source bytes, four clients and 5 requested operations/sec.
Keep zero error allowance, 500 ms p99 limit, 30-second per-operation deadline
and two-hour setup limit. The short campaign remains one-seed characterization
with two forced reopens; its short soak phase is not a 24-hour qualification.
Complete independent corpus readback precedes measured acceptance.

Before actual launch, require a clean exact revision/tree, successful native
build/preparation, binary/configuration/campaign hashes and repository launcher
resource admission. Retain the launch and terminal receipts separately. Stop
on insufficient disk, an active conflicting campaign, build failure, source
or artifact mismatch, unauthorized resource changes or incomplete cleanup.
No launch or native performance result is claimed by this integration record.

## Review, research and acceptance

Full reads: dependency PR briefs/journals, changed source/test diffs, point-index
module and lifecycle test, all four operator staging/preparation/launch scripts.
Targeted owner reads: README/AGENTS, C-07, POAM-035 and change-authoring routine.
Primary research/profiling is recorded in PR141's journal; integration does not
adapt further source. Preserve page/I/O diagnostics, source identity, resource
limits, independent record/relation/vector values and authentication.

Source provenance comparison and whitespace/conflict review passed. Final
documentation (622 statuses, 620 coordinates), navigation (621 nodes, 2451
edges), inventory (1751 paths), workflow and workspace-format checks passed. Storage-suite results are attributed to the exact dependency; the fresh
installed candidate is a distinct observation and must retain its own results.
Full capacity matrix, throughput comparison and actual 24-hour soak remain
unrun/unqualified. C-07 and POAM-035 remain open.

Checklist: pre-effect PR brief, exact baseline/dependencies, canonical
replacement mapping, research/resource/trace decisions, all retained failures,
source equivalence checks, complete integration diff review, linked immutable
record, mechanical projections and explicit limits. The current Codex driver
owns preparation, actual launch observation and subsequent terminal evidence.

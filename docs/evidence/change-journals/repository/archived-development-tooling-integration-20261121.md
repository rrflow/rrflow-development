# Archived development tooling integration

**Status:** active integration evidence; qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-development-tooling-integration-20261121`
**Owner:** repository development gate and Gate C tooling

## Before-edit brief

Join the original archived development-container, CI diagnostics and lab-tooling sources into their current canonical owners.

Driver: finish original source integration after PR485. Baseline green main db97d68c1ca9c432a186c4de159a5b8f634ec8b6; join PR485 before readiness. Owner: repository development gate / Gate C qualification tooling, not new engine capability. Sources: dd74d0bd6dd5375786cfc3ce19e6268822031bc2, cd7e208001dbd6e9e17435432f143fc37a10e066 (devcontainer parent e4103a72137f03c028456d76b5bd824d2dfac76c), 0cbd08ec73892036200f8caa04fc43d35e84d16b, 2458039e22aacac80684c96b1e33ba06d68887e7, 5a53fd096f20580790c425db592df15fb43bea15 and original integration merge 417e4eb04c47a3dfb36b28bf2c4ae99111d4b874. The latter has no unjoined non-merge ancestors; its integration object is not claimed as new implementation.

Current boundaries: .devcontainer files; scripts/ci/{check_workflow,test_candidate_admission,run_sustained_capacity,sustained_progress,test_sustained_launch,inspect_candidate_runs,test_inspect_candidate_runs}; scripts/lab; eval/lab import/run/source manifests; crates/evaluation/lab/rrflow-lab/tests/contract.rs. Keep current conditional CI lanes, merge-queue cache restrictions, engram naming, split qualification crates, pinned container inputs and lab Git identity. Original old rrflow-eval test modules map to split owners, not a second crate. Preserve source evidence with current status. No comparison or capacity workload, container execution, new provider, runtime instrumentation or installation change.

Read: original dd74 workflow/cache/preflight patch; current progress reader and its tests, runner change, candidate assertions; current devcontainer JSON/compose/seed and Containerfile first 130 lines; current pod scripts; full corpus importer and source manifest assertion; original inspection patch and current summarizer/test; original D03 journal and current inventory_delta ignored-test annotation. The existing interrupted-refresh oracle remains ignored with POAM-006 explicitly open; no repair is claimed. Inspect remaining conflict hunks before resolution; broad truncated diff is not a full review.

Acceptance: candidate-admission, sustained-launch and inspection Python suites; workflow policy; rrflow-lab contract tests (metadata only); architecture; docs/navigation and whitespace. Dedicated Cargo target D:/.usr/Temp/rrflow-root-target, four jobs. No new research or performance experiment is needed to join existing code. Stop on missing assertions or broken boundaries, repair before readiness, never weaken the gate. Journal records each source merge and changed-tree result; green CI/queue then automatic remote branch deletion, local deletion after main advances.

## Integration and preservation

| Original source | Merge | Canonical result |
| --- | --- | --- |
| dd74d0bd6dd5375786cfc3ce19e6268822031bc2 | ab797c69 | Existing progress observer and tests retained; current lane-aware preflight/cache checks retained |
| cd7e208001dbd6e9e17435432f143fc37a10e066 | a5b61524 | Original container branch joined; pinned tools, recursive seed and later lab target retained |
| 0cbd08ec73892036200f8caa04fc43d35e84d16b | 25a0e1c0 | Runner stash joined; current Git identity capture and split rrflow-lab package retained |
| 2458039e22aacac80684c96b1e33ba06d68887e7 | 29f005b60 | Importer and COCO contract assertion retained; current system registry replaces old hard-coded launch cases |
| 5a53fd096f20580790c425db592df15fb43bea15 | abdd193f | Pending-step filtering and regression retained alongside later bounded timing diagnostics |
| 417e4eb04c47a3dfb36b28bf2c4ae99111d4b874 | recorded in Git parentage | Original integration merge joined cleanly; no unjoined non-merge ancestors |

All six original joins leave the implementation/test tree unchanged. This is
source consolidation, not a new runtime capability. No test or gate was removed.
The lab stash's third parent cccd0f21b98e11112046de45a8c6f9918b7485f1 is empty.

Conflict review covered the original dd74 cache/preflight patch and conflicting
current lane selection/reduction; current changes must select their required
lanes and require success, while unselected jobs must be skipped. Storage
preflight remains prerequisite to the full installed-reopen matrix. Current
rrflow-qualification-storage and selected_tests names replace old rrflow-eval
and stale hard-coded test paths. The original observer and its tests are
byte-identical at the current owner. The D03 interrupted-refresh journal is
also retained unchanged; inventory_delta.rs explicitly marks its oracle ignored
and unrepaired under POAM-006. It is not counted as passing.

Container sources match except the later pinned Docker CLI lab target. Current
journal contains the subsequent actual integration evidence, which replaces
its source's pending prose. The incoming duplicate Local development container
section was removed because the existing section already owns pinned versions,
recursive seeding and the same instructions. Generated indexes/file-plan were
retained at current state, not restored from obsolete inventories.

Lab corpus importer is byte-identical; its neighbor score-gap validation and
hashes remain. The COCO manifest assertion lives in the split rrflow-lab
contract test. run_system.sh now consumes signed definitions and the system
registry, including VectorDBBench arguments and embedded-process behavior;
old fixed-name start cases would discard that boundary. The current launcher
also records Git identity before mounting, preserves pack notes, and cleans up
by validated container ID. The inspection source's pending/queued filtering and
original regression remain; later bounded longest-step diagnostics and their
tests are retained. Historical journals keep their completed verification.

Reviewed scope is the before-edit list plus original source patches and actual
conflict sections above. Large historical CI/file-plan and lab documentation
diffs were truncated; those outputs are not claimed as full line review. No
container, benchmark, comparison, imported external dataset or capacity run was
executed. No hosted/UI/alpha result is inferred from this package.

## Verification

Focused tooling and contract acceptance follows on this joined tree. No engine
publication code is changed here; PR485 owns its separate tested repair.

All acceptance commands passed on 1c0a68e7 plus this record: Python
scripts/ci/test_candidate_admission.py (21 tests), test_sustained_launch.py (10),
test_inspect_candidate_runs.py (9), check_workflow.py (19 container pins);
`cargo test -p rrflow-lab --test contract --locked` (32),
`cargo test -p rrd-engine --test workspace_architecture --locked` (35);
`python scripts/ci/check_documentation.py`, navigation and `git diff --check`.
No benchmark/container/installed-runtime workload was run for this unchanged
implementation tree. The original native diagnostic integration merge is
1c0a68e7. PR485 must land and be joined before this draft becomes ready.

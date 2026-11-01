# Startup recovery composed with landed publication repair

**Status:** active composed validation; candidate/protected qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-startup-landed-main`
**Owner:** OBJ-14 / D-01 / D-02 / D-03 / POAM-006; active SWAG session; PR227

## Source, brief and traceability

The pre-edit reconciliation brief is in [PR227](https://github.com/rrflow/rrflow-development/pull/227).
Starting head `27e87749cb011828a35c8f0e1a9fb88a9a996012` ordinarily merged landed
main `906bec82c605ed095a2dbf9b0217cd89bd41c05f`, producing
`8ea31f5af95aa5148d5371ec1db30d24b9141dc7`, tree
`c5520cb7d13edaaf30504493140abaf481874ffd`. Both parents are retained and no
conflicts occurred. PR224 and PR228 are verified merged. PR226 remains open
until its retained implementation has a verified landed destination.

The production delta consists solely of the already-landed
[publication repair](../gate-c/c-07-object-publication-repair.md):
`rrd-store/src/object.rs::publish_object_path` preserves a verified concurrent
winner using non-replacing anchored rename; corrupt collisions retain both files,
other errors propagate and quarantine retains its prior semantics.
`rrd-lsm/src/filesystem.rs::rename_new_to` carries the existing crash observations.
Both files equal current main byte-for-byte. The engine source is unchanged
from the starting head. Remaining incoming paths are existing journals/indexes
and POAM-035; there are no new contracts, dependencies or alternate authorities.

The [startup implementation](d-03-startup-binding-inspection.md) and
[incremental publication](d-03-incremental-publication.md) remain at their
canonical installation and Inventory owners. The reviewed diff retains their
source-dependent guards, current-record reuse, shared read budget and progress
work-set identity. Opening a verified estate still does not certify project
freshness or authorize a dependent effect.

## Composed acceptance

All commands below exited zero at clean implementation revision `8ea31f5a`.
The dedicated target was `target/startup-f1a033b7`, with two build jobs and
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`.

| Command | Result |
|---|---|
| `cargo test -p rrd-store publication_ --locked` | Seven selected passes; includes reader-held winner, corrupt collision and missing-source cases |
| `cargo test -p rrd-engine installed_startup_keeps_source_admission_at_the_dependent_operation --locked` | One selected pass |
| `cargo test -p rrd-engine --all-targets --all-features --locked --no-fail-fast` | 342 passed across 13 targets; four existing ignored helpers |
| `cargo test -p rrflow-cli --test installed_lifecycle --locked` | Seven passed, none ignored |
| `cargo clippy -p rrd-engine --all-targets --all-features --locked -- -D warnings` | Passed |
| `cargo fmt --all -- --check` | Passed |

The full engine suite includes all 32 architecture checks, source closure,
current-record/edge/retirement reuse, unchanged Inventory and installed startup
regressions. The startup oracle observes unchanged job readback with invalid Git
input, denies Start without mutating the job, permits Start after input repair,
retains actual full-inspection evidence and still denies bootstrap tampering.
The seven CLI cases execute the installed lifecycle including authentication,
restart and duplicate-serving refusal. These checks are not training data.

Local raw command receipts and logs are under
`D:/projects/swag/.swag/evidence/20261031/startup-main-composition/`.
Engine log SHA-256: `6de5e61b4bb045597fa99929e69d030cf7cc0b618ac52c5714db539b93315d06`.
Lifecycle log SHA-256: `c2b80c726336e4fa5203ceae230b56f6f9053c3be7c485ef9cc4b75f2188faec`.
No test failure occurred in this composition. Exploratory reads initially
used a nonexistent test path; locating the actual attunement test corrected the
read, without source changes. A compound shell write/check command was rejected
before execution; the journal was instead added using the file-patch tool and
checks run separately. The first documentation check rejected this new record's
status because it omitted the required active/historical classification; the
status was corrected to active. Prior failed experiments remain retained.

`python scripts/knowledge/render_navigation.py` passes with 834 nodes and 3,046
edges; `python scripts/ci/build_execution_inventory.py --check` passes with
2,108 path records after regeneration. `python scripts/ci/check_documentation.py`
passes with 835 statuses and 833 classified coordinates. `git diff --check`
passes. The final review contains only this new record and its generated index
in addition to the declared ordinary merge; AGENTS.md is unchanged.

## Change checklist and limits

Review covered README ownership, AGENTS, the full change-authoring procedure,
the prior implementation/installed journals, landed storage repair journal,
CI admission and the complete incoming production diff. The startup regression,
Inventory reuse module and Inventory delta tests were reread in full; startup
open/inspection and all remaining engine changes were reviewed in their diff and
surrounding implementation. Prior full-file implementation reviews remain
attributed to their immutable journals. Final increment adds this journal and
regenerates its navigation; imported implementation is unchanged after testing.

No external algorithm adaptation or new research is needed for ordinary
composition. Existing bounded logs and exact revisions supply diagnostics;
no new content trace or resource measurement is introduced. Stop conditions
remain loss of binding checks, invalid-source effect admission, weakened storage
semantics or a separate repair outside this package. None was observed.

The earlier 1,870-path installed trial remains attributed to `39a7b644` and its
original binary; it was not rerun or reassigned to this merge. The historical
intermittent binding rejection remains unexplained. Scheduled Verify, safe
upgrades, full UI/worker/project effects, RROFLOW, reviewed learning, latency/RSS,
model qualification and native/container release are not established here.
Required candidate and protected CI remain outstanding. No roadmap gate or
POA&M status changes; product version remains `1.0.0`.

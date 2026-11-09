# C-07g bounded-window candidate integration and native launch

**Status:** active characterization evidence; qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-window-native-launch`
**Owner:** OBJ-02 / C-07g / POAM-035 / PR139

## Frozen source and dependency correction

[PR139](https://github.com/rrflow/rrflow-development/pull/139) implemented and
locally verified bounded corpus readback at exact source
`04559542ada72d335cdf21c3b586eec46895f205`, tree
`2f85814a5c988f84f01204db68d718119cae17c2`. Its
[implementation journal](c-07g-bounded-corpus-readback.md) owns those commands
and outcomes. This follow-up records native execution and ordinary integration
of PR134's [trace correction](c-07g-proof-read-trace-conformance.md), revision
`4e2dbaffad29039d742f6b151f7636378ce3e7fe`. The driver (current Codex session)
published the dependency brief in PR139 before the merge.

Current to target: preserve PR139's driver and engine bytes, absorb its
explicit dependency's exact trace fixture correction, retain both evidence
records and regenerate navigation/inventory. Scope is that test, POAM-035,
linked journals and generated projections. No production, public API, profile,
Cargo dependency, corpus or acceptance changes. Stop for any unexpected source
change, dropped record, new native failure or changed frozen workload identity.

The ordinary merge conflicted only in the appended POAM descriptions and
replaceable file inventory. Both POAM paragraphs were retained in order; the
inventory was regenerated. Its first regeneration refused unmerged index
entries; resolving the index and regenerating corrected that procedural error.
No hand-edited generated inventory or conflict markers remain. The new trace
fixture matches PR134 byte-for-byte; its linked journal records reproduced
failure, three focused passes, 269 owning-suite passes and strict lint. This
follow-up does not claim those tests ran on a different binary or revision.

Full reads: corpus.rs, corpus_window.rs, PR139 brief and journal, trace fixture,
new trace journal, stage/prepare/launch/inspection operator scripts and exact
source diffs. Targeted reads: README/AGENTS, change-authoring procedure,
POAM-035, candidate jobs and previous failed characterization. Research:
existing code and retained failures suffice; no upstream adaptation. Trace and
resource decisions: preserve source/binary/harness/configuration identities,
original limits and independent expected-value readback; retain old estates.

## Native boundary

The owned Linux KVM guest had no running `rrflow-c07-*` service before staging.
Staging fetched only the public development source, recursively fetched its
pinned Connectome gitlink and checked a clean exact-source checkout. Preparation
service `rrflow-c07-desktop-prep-04559542` started at 2026-10-26 03:54:43 UTC.
It builds the locked release primary and installed-reopen harness, validates
installation/configuration and prepares a new estate; it does not reuse the
failed `05817485` data. That older failure and all its evidence remain intact.

The same Desktop characterization preserves 4 vCPUs, 8 GiB guest memory,
2 GiB engine limit, 100000 records, 500000 edges, 100000 vectors of dimension
384 and 102400000 logical source bytes. It preserves the previous 64 MiB page
cache setting, four clients, 5 requested operations/second, zero error allowance,
500 ms p99 limit, 30-second operation and two-hour setup deadlines. The
campaign's short measured/reopen phases are characterization, not an actual
24-hour soak or the full capacity-profile matrix. Complete independent corpus
readback must precede measured workload acceptance.

The native source remains `04559542` even after this test/documentation merge.
A later source revision does not inherit its qualification. Exact-source
candidate CI and final profile/24-hour qualification remain required.
No roadmap status or maturity change is authorized by this record.

## Observed launch and retained identities

Preparation completed successfully. The reviewed operator launcher started
`rrflow-c07-desktop-integrated-04559542` at 2026-10-26 04:02:05 UTC, bounded by
15150 seconds. A subsequent read-only inspection confirmed active/running
and 576 acknowledged corpus operations (last ordinal 575); independent corpus
verification and measured load had not yet begun. No failure event or launcher
stderr was present at that observation. These are launch/setup facts, not
completion, durability qualification or a throughput improvement claim.

Retained evidence root in the owned guest:
`/home/devcontainers/rrflow-c07-native-79663cdd/desktop-integrated-preflight-04559542/`.
It contains campaign/configuration, build/installation records,
`service-launch.json`, `evidence/progress.jsonl` and launcher logs. The fresh
estate temporary parent is `desktop-integrated-files-04559542` under the same
owned base. Preparation log: `desktop-preparation-04559542.log`. A local copy
of the launch receipt is `target/c07-window-service-launch.json`.

- Source/tree: the frozen `04559542`/`2f85814a` identities above.
- Primary SHA-256: `be77466dffe12cd2eee91bc3f5996702089c69481fcefd26310064e40f33c5ca`.
- Harness SHA-256: `fab6dd8536095a6b4ea50775f631169cf535756770d3cb23e593d9d2c5ffc70f`.
- Configuration SHA-256: `d2013902bcc7c9efe4840170d35a35aae22e5fcffd0f1f23eca6e9c0e7fcfbf2`.
- Campaign-file SHA-256: `a0e7fca8eb2c05e603e6aea25574c5ccba6a17883517d09442ade6bceeb97727`.

Cgroup-v2 enforcement reported the frozen 2 GiB engine/8 GiB host allocation
and four vCPUs with the exact harness digest. No continuation or qualification
service exists for this revision; **no actual 24-hour soak has started**.
The primary matches the prior characterization because this package changes
the driver; its distinct harness and source identities are retained explicitly.

## Acceptance and limits

Read-only source comparison against the frozen commit found exactly one change
under crates/Cargo/eval/scripts: the imported trace test. That fixture matches
`4e2dbaff` byte-for-byte; all production and corpus-driver source is preserved.
The merge's runtime oracles are the linked package results, not newly claimed
Cargo execution. No additional Cargo rerun was performed for this nonoverlapping
test/documentation integration. Corrected PR134 candidate 40266752462 is running;
the superseded failed candidate 40265699310 was cancelled after retaining both
failure logs. PR130's separate normal merge-group run remains untouched.

Documentation passed with 618 statuses/616 coordinates; navigation regenerated
17 indexes/617 nodes/2438 edges; file inventory regenerated 1746 records.
Final projection, workflow, conflict and whitespace checks passed. The complete
integration diff and journal were reviewed before commit.
Checklist: pre-effect brief, exact dependency map, preserved source/readback
oracles, retained failures, explicit research/resource decisions, actual launch
observed separately from setup or soak completion, complete diff review and
linked evidence. C-07 and POAM-035 remain open. Full independent readback,
measured native phases, profile matrix and 24-hour qualification are not yet
passing evidence for this candidate.

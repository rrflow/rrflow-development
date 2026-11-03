# Roadmap rewrite for the owner's relational and adaptive engine direction

**Status:** active planning-correction evidence; implementation and qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-a/owner-direction-roadmap-rewrite`
**Owner:** OBJ-02/05/06/07/09/10/14 and Gates C/D/F/G/H/J
**Pull request:** [264](https://github.com/rrflow/rrflow-development/pull/264)

## Starting point and requested correction

The reviewed public main is `ed3027b79e41dfcdf5b27e57f07346f11f6da347`, tree
`18fbef0dc4a271f5ce8004cd865b9272b1115a9c`. The PR brief preceded source
edits; the scope was amended before editing the graph reference and existing
gap details. The bootstrap commit is empty and supplies no product behavior.

The owner described an AI-oriented multi-model database with relational
rrflowQL, table/kind-plus-ID record identity, configurable per-request
relationships and reasoning, persistent hot and temporary prompt state,
Parquet cold compaction, SQLite and PostgreSQL compatibility, configurable
LFG/SWAG, source knowledge paths and a measurable real-project developer
build. Main already captured many parts of this direction, but the owning
gate did not require Parquet cold compaction; SQLite appeared as a migration
source without a compatibility target; D-05 and G-03 could be read as fixed
runtime sequences; and the graph reference required every active family to
persist for each prompt. The reference also overstated ACID/C-04 closure.

This change rewrites those conflicting requirements and adds F-06 and H-12 as
unchecked work, plus the missing H-11 checklist link. It preserves the
one-engine transaction and source authorities, existing gate statuses,
accepted evidence, local-project-first delivery, external-effect authorization
and C-07 native/soak requirements. It does not remove, move or modify runtime
implementation, tests or historical evidence. No roadmap checkbox is closed.

## What was examined

Read in full: the root README; the alpha objective and main roadmap; roadmap
chapter index, current-alpha and execution-checklist chapters; dependency
spine; gate chapters A through J, C-07 and their linked A/B/C/D acceptance
chapters; the adaptive developer-build design and the complete installed
project-indexing plan; the graph-family capability reference; and POAM-004
and POAM-044. The inventory generator was reread in full after the mapping
change; its override precedence replaces package defaults, so the new entries
retain each affected file's prior assignments as well as the new ones. The
roadmap's current status and historical acceptance were
reviewed as records of their stated revisions, not independently rerun here.

Targeted source reads covered `rrd-query/src/syntax.rs`, `plan.rs` and
`parquet.rs`; `rrd-core/src/runtime.rs`; LSM WAL and flush/compaction symbol
locations and the current-format reference; the native/Fjall historical
comparison and its two JSON verdicts; and PRs 252, 255 and 256. These are not
a complete semantic review of every source file or every line in the
repository. The real Windows document/relationship process journey at PR 255
was rerun separately using the journaled executable digests and passed; its
synthetic input is not a real-project indexing or compatibility proof. A Linux
ARM64 real-store equijoin/reopen test passed at PR 249 code commit `7aadf8fb`;
that is not SQL compatibility or joined alpha evidence.

## Research, changes and limits

[PostgreSQL SELECT](https://www.postgresql.org/docs/18/sql-select.html) and
[transaction isolation](https://www.postgresql.org/docs/18/transaction-iso.html)
show why parsing, relational result semantics and transaction isolation need
separate tests. [SQLite type affinity](https://www.sqlite.org/datatype3.html)
and its [file format](https://www.sqlite.org/fileformat.html) distinguish SQL
behavior from application ABI and file compatibility. The
[Parquet file format](https://parquet.apache.org/docs/file-format/) specifies
column chunks and metadata; it does not establish RRFlow publication, WAL
durability or recovery. No upstream code is copied. Existing bounded traces
and durable receipts remain the intended observability owners; this
documentation does not create instrumentation or performance evidence.

Changed owners: README; alpha objective; the canonical roadmap, dependency
spine, C-07/D/F/G/H/J chapters and project-indexing plan; the prompt-graph
reference; POAM-004 and POAM-044; the execution-inventory source-owner mapping
and its regression test; this journal and generated navigation.
Review the complete diff before commit. The new compatibility and cold-tier
requirements are target work, not an assertion that the current engine
implements them. Choosing which SQLite application ABI or direct file surfaces
to promise remains explicit implementation-profile work; no unspecified
surface is silently excluded or claimed supported.

## Verification and remaining work

First characterization: inspect the source and owner text above. The code uses
a bounded FROM/JOIN/PROJECT grammar, a scoped `(kind, id)` reference, a native
WAL and segment implementation, and a bounded uncompressed Parquet export.
The exact runtime capability and end-to-end gaps remain open; this is not a
replacement test for them. An initial patch attempt against Gate D failed
because its surrounding text did not match; the edit was reapplied against
the actual lines. `python scripts/knowledge/render_navigation.py` updated one
index; its `--check` passed. `python scripts/ci/check_documentation.py` passed
with the journal present, and `git diff --check` passed. The inventory
generator then refused unmapped F-06, H-11 and H-12; the PR brief was amended
before mapping those gates to the existing LSM, schema and query source
boundaries. The first mapping omitted inherited package assignments because
file overrides replace defaults; review caught this and the mappings now retain
them. A focused regression checks both preserved assignments and new gates.
No inventory validator was weakened. The inventory test passed 11 cases,
inventory generation/check and navigation check passed, and
`uvx --offline ruff==0.17.1 check` passed for the generator. Direct `ruff`
was not on PATH; the cached pinned tool was used. Its format check identified
one changed line and one already-unformatted baseline line in the same file;
the file was formatted and the baseline failure retained here. Ruff also
identified an existing unsorted import in the changed test; it was reordered.
Final checks are rerun after the record and reviewed diff are complete.

Checklist: owner and starting revision recorded; scope and PR brief bound before
the affected edits; existing behavior and failure oracles mapped; research and
trace/resource decisions documented; no implementation was removed or
relocated; changed files and full diff reviewed before commit; candidate and
merge-group CI remain required before merge. No runtime tests, C-07 capacity
matrix, soak, PostgreSQL/SQLite conformance, installed end-to-end alpha or
competitive benchmark was run for this documentation change. None is claimed
complete, and no POA&M lifecycle status changes.


## Supporting-record correction, 2026-11-03

A follow-up review against public main `ed3027b79e41dfcdf5b27e57f07346f11f6da347`
found that the active engine data-flow record described the eleven-step
sequence as a frozen target and said no installed executor or inventory
implementation existed. It also presented three router decisions as a
permanent limit. The historical Connectome audit treated the eleven names as
a target projection, and the active historical research matrix presented its
old attunement status as current. The PR brief was extended before edits to
those three supporting records. The corrections distinguish today's fixed
default and limited execution from the capability-selected target; they do
not claim adaptive planning has been implemented. Source checks inspected both
profile JSON files, `installation.rs` selection and sealed-preview validation,
`rrd-contract/src/attunement.rs` plan/checkpoint/job validators, and
`engine/attunement.rs` execution and source checks. PR #252 head
`f39b09c44bc6eadacbf1f7ac57cc7261f93466b5` remains a draft with the
interrupted-refresh mixed-generation oracle failing. PR #241 remains a
dependency-blocked parser draft; its branch is not proof of installed Parse.

[Temporal replay](https://docs.temporal.io/encyclopedia/event-history/event-history-typescript)
informs replay and external-activity separation;
[Camunda workflow patterns](https://docs.camunda.io/docs/components/concepts/workflow-patterns/)
show conditional branches are supported orchestration patterns, not evidence
that any particular sequence is necessary for RRFlow.
[OpenTelemetry sampling](https://opentelemetry.io/docs/concepts/sampling/)
may omit spans, so sampled diagnostics cannot replace durable receipts. No
upstream implementation was copied; no runtime trace or performance test was
needed to correct these documentation statements. No runtime behavior,
roadmap completion state, candidate merge or C-07 qualification is claimed.

The full data-flow review additionally found that its incremental section
asserted zero content reads for every unchanged refresh and imposed a complete
project snapshot for every durable effect. The correction requires a complete
relevant dependency boundary, conservatively refreshes where its completeness
cannot be established, and treats zero content reads as an evidence-dependent
optimization. The historical research introduction now identifies its dated
baseline and no longer presents a three-decision router or a complete-project
snapshot as the universal target. The PR brief recorded these specific extra
findings after their data-flow prose edit; the earlier brief had named the
supporting records but not these two findings. The three supporting records
were read in sections; source citations and their historical status remain.

Follow-up checks: `python scripts/knowledge/render_navigation.py` and
`python scripts/knowledge/render_navigation.py --check` passed (18 indexes,
948 nodes, 3388 edges, no index changes); `python scripts/ci/check_documentation.py`
passed (949 document statuses, 947 coordinates); `python
scripts/ci/test_execution_inventory.py` passed all 11 tests; the inventory
generator and its `--check` passed with 2349 records; and `git diff --check`
passed. An initial `python -m unittest scripts.ci.test_execution_inventory`
failed because that invocation omits the script directory from Python's import
path (`ModuleNotFoundError: build_execution_inventory`); the repository's
script invocation passed. The first inventory `--check` observed expected
drift while the documentation edits were ungenerated; regeneration fixed it.
No engine or C-07 tests were run in this follow-up. Candidate CI must evaluate
the later commit, not inherit the prior run's result.


## Owner clarification: streamed relationship access

The PR brief was expanded before editing Gate E, Gate F and engine data flow.
The owner directed rrflowQL to remain relational; canonical relationships and
dynamically selected graph-shaped views share the rrflowDB transaction. The
target now requires native ordered adjacency, the C-06g-derived stamped
projected stream, the configured physical page cache, bounded memtable merge
and mapped-buffer lifetime, with no raw-pointer bypass. An estate-scoped
(table, ID) remains the record identity for structured and schemaless records.
The current executor uses `AdjacencyExpansion` and
`read_versioned_expanding`, not the C-06g stream; the current `begin_projected_read`
and `ProjectedReadStream` provide only the underlying physical stream. F-01
and F-03 remain unimplemented for this end-to-end path. E-01/F tests must
measure relevant edges and versions, seeks, LSM runs, keys/pages, cache,
resources and exact correctness while increasing unrelated data. Ordered seeks
and version/proof work mean an unconditional zero-overhead O(E) claim is not
shown. Current relationship bytes, persisted identity and tests were not
changed by this documentation package.

Research: [Arrow columnar layout](https://arrow.apache.org/docs/format/Columnar.html)
describes relocatable buffers and conditional physical characteristics;
[DataFusion TableProvider](https://docs.rs/datafusion/latest/datafusion/catalog/trait.TableProvider.html)
describes streaming and projection/filter support. These are design inputs,
not RRFlow streaming, zero-copy or performance evidence. No upstream code was
copied. The owner also requested PostgreSQL and project source support; native
compatibility and an explicitly admitted external source remain separate
implementation requirements rather than a single claimed adapter.

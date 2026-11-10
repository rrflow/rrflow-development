# E-04: authenticated vector generation source identity

**Status:** active draft; the main-source failures are reproduced and the local repair is under verification
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-vector-source-identity-20261107`
**Owner:** E-04, OBJ-13, [POAM-053](../../../poam/rrflow-1.0-alpha/poam-053.md), [PR #298](https://github.com/rrflow/rrflow-development/pull/298)

## Baseline and scope

Public main `b555d22c891c511775b5e54a9f9d00c8a51da48e` is the source baseline. Empty bootstrap commit `f29d42131fc91945c0e75dc18fdb67414b5871af` changed no files and allowed a draft PR and effect brief to exist before source edits. Before its push the remote was resolved as `development`, `https://github.com/rrflow/rrflow-development.git`, ref `refs/heads/agent/vector-source-binding-20261107`, exact bootstrap revision; no prior ref existed. The PR brief owns scope, failure oracles, preservation and stop conditions.

On Windows x86-64, with `CARGO_TARGET_DIR=C:/Users/jessa/AppData/Local/Temp/rrflow-vector-source-binding-target-20261107`, `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_TEST_DEBUG=0`, the baseline plus only tests (test-file Git blob `d56df945e57d784c295d38df6936d72d3c8e8d28`) produced:

- `cargo test --offline --locked -p rrd-engine --lib distinct_named_vector_sources_have_distinct_hnsw_indexes -- --nocapture`: failed, exit 1. Both `(a-b, c)` and `(a, b-c)` returned `hnsw-a-b-c`.
- `cargo test --offline --locked -p rrd-engine --lib an_empty_foreign_hnsw_generation_cannot_answer_for_a_populated_source -- --nocapture`: failed, exit 1. Approximate search returned `{}`; exact search returned `{mirror-point}`.

These tests exercise real in-process engine and persistent temporary storage. They are not installed-binary, crash, multi-platform, competitive or alpha acceptance evidence. The separate unmerged [#278 integration](https://github.com/rrflow/rrflow-development/pull/278) is not on this branch and is not a dependency of reproducing the failures.

## Boundary and research

The current HNSW build joins source name components with hyphens for `ProjectionId`; its generic durable catalogue entry binds the descriptor and immutable object but does not carry a structured source address. The authenticated runtime commit atomically publishes its record and object. A missing source cannot be inferred from an empty graph or from that ambiguous ID. Historical bytes and existing logical data require explicit, non-destructive handling.

[FoundationDB's tuple guidance](https://apple.github.io/foundationdb/data-modeling.html) explains delimiter ambiguity and typed tuple keys; [PostgreSQL's unique indexes](https://www.postgresql.org/docs/current/indexes-unique.html) describe uniqueness on combined column values. [SQLite's WAL documentation](https://www.sqlite.org/wal.html) demonstrates why older-reader and recovery behavior must be considered during format changes. These are design references, not imported code or proof of RRFlow behavior. Existing authenticated commit and read evidence remain the trace boundary.

For the subsequent comparative workbench, [Qdrant's benchmark methodology](https://qdrant.tech/benchmarks/) compares vector speed at comparable precision and identifies filter selectivity as a separate variable. [SurrealDB's EXPLAIN reference](https://surrealdb.com/docs/reference/query-language/clauses/explain) exposes query plans for diagnosing selected access paths. Neither supplies an oracle for reasoning or code quality: project-derived task outcomes, grounding, failure and latency must be evaluated separately. [OpenTelemetry's SDK sampling specification](https://opentelemetry.io/docs/specs/otel/trace/sdk/) permits dropping spans and their events, so sampled telemetry alone cannot establish complete effect audit. These are design inputs for that future joined driver, not evidence of a comparison or a working workbench.

Files examined in full for this initial oracle: `crates/authority/rrd-engine/src/engine/tests/vector_index/collection_moves.rs` (prior baseline before these appended tests), `docs/roadmap/rrflow-1.0-execution/change-authoring.md`, `docs/poam/rrflow-1.0-alpha/poam-048.md` on the integrated draft, and `docs/decisions/0002-adaptive-reasoning-governed-effects.md`. Relevant sections of `rrd-vector/src/catalog.rs`, `engine/vector/index.rs`, `runtime/vector_catalog.rs`, Gate E and README were inspected. No claim of a whole-repository review is made.

## Implementation in progress

The engine now authors a typed source in versioned catalogue entries, derives HNSW identities from a domain-separated serialization of the address, verifies the stored address on reuse, and excludes unbound or foreign approximate generations for named searches. The source participates in the catalogue digest, and decoded bound graphs are checked for foreign nodes. Version-2 records retain their original identity calculation and remain unbound; no automatic durable rewrite is introduced. These are unmerged working-tree changes, not completed qualification.

An initial focused pass of the two original cases succeeded before trace enrichment. The subsequent `cargo test --offline --locked -p rrd-engine --lib collection_moves -- --nocapture` failed with eight HNSW publication errors because the added `source_bound` and `catalog_format` trace attributes were not accepted by the canonical trace catalogue. Those unregistered attributes were replaced with the existing typed source link, which carries a digest of the source and its covered cursor.

To honor C-05's single digest owner, the source is included in the existing
operation receipt rather than in a separate digest implementation. On the
local candidate, `cargo test --offline --locked -p rrd-engine --lib
collection_moves -- --nocapture` passed 18 tests; `cargo test --offline
--locked -p rrd-vector --lib named_source_is_authenticated_and_older_entries_remain_unbound
-- --nocapture` passed one; and `cargo test --offline --locked -p rrd-engine
--test runtime_vector_artifact_catalog -- --nocapture` passed eight. These
include the baseline oracles, readback, and refusal before a mismatched source
can be published. The retained v2 test independently reconstructs its old digest;
it is not a process-crash or installed-estate compatibility qualification.

The first owning library run, `cargo test --offline --locked -p rrd-engine
--lib`, failed: 393 passed, one failed and four were ignored. It exposed that
payload-index deletion still reconstructed the obsolete ambiguous HNSW name.
The related collection deletion guard had the same problem. After the PR brief
was amended, `engine/vector/collection.rs` was read in full and both guards
were changed to consult authenticated source bindings; v2's ambiguous old
name is used only to conservatively refuse deletion. The existing
`collection_filtered_hnsw_overlays_retirement_and_protects_payload_index`
oracle passed after the change. A separate empty-collection deletion/reopen
oracle was added. The complete `cargo test --offline --locked -p rrd-engine
--lib` rerun then passed 395 tests, with four ignored. `cargo test --offline
--locked -p rrd-vector` completed successfully; its library had 73 passed and
one ignored, and its integration and documentation tests also passed. These
are Windows local source tests at the working-tree identity, not candidate CI.
Navigation generation reported 18 indexes, 1,018 nodes and 3,577 edges, with
two index files updated; `git diff --check` passed before the latest edits.
`cargo test --offline --locked -p rrd-client --test vector_indexes` passed one
real local client/server test, and `cargo test --offline --locked -p rrd-engine
--test workspace_architecture` passed 33 checks. The client test uses a
synthetic fixture; it is not an installed real-project journey.

Review found a remaining lifecycle limitation: the conservative v2 deletion
guard may block either source sharing an ambiguous old name, while the engine
has a public quantization-retirement operation but no HNSW retirement operation.
Rebuilding does not remove the old HNSW entry. This needs an authorized,
non-destructive lifecycle solution and is recorded under POAM-053. Do not mark
the compatibility or complete feature done. Remaining package checks,
candidate and protected CI remain open. `cargo fmt --all -- --check`,
`python scripts/knowledge/render_navigation.py --check`, and
`cargo clippy --offline --locked -p rrd-engine -p rrd-vector --all-targets --
-D warnings` passed. The C: target directory above was used for Clippy, with
`CARGO_PROFILE_DEV_DEBUG=0` and incremental compilation disabled.

After those checks, source review found that hashing the full serialized
address struct would allow a future unrelated struct field to change a v1
projection identity. The hash input was narrowed to the two explicitly ordered
address strings under the same domain. The tests and lint above precede this
last source change. The current identity is pinned by two independent SHA-256
test vectors calculated over the domain and serialized ordered strings. After
the change, `cargo test --offline --locked -p rrd-engine --lib --quiet` again
passed 395 with four ignored, and `cargo test --offline --locked -p rrd-vector
--quiet` passed its full suite with 74 library tests passed and one ignored.
Client/server, trace integration and lint must be rerun against this identity.
They subsequently passed: eight runtime catalogue tests, one client/server
vector-index test, and all-target Clippy for `rrd-engine` and `rrd-vector`
with `-D warnings`, using the commands and isolated target recorded above.

The first `python scripts/ci/check_documentation.py` run failed because this
new POA&M detail lacked three required section headings and the journal linked
to a local file that exists only on the separate #278 draft. The headings and
link were corrected; the rerun must be recorded.
That rerun also failed because the owning-gates section lacked its required
roadmap link and the checker interpreted the initial letter of "Gate" as a
separate family. The section was corrected to the existing ledger convention.
The third documentation-policy run passed, reporting 1,019 document statuses,
1,017 classified coordinates and valid parent indexes and local links.
The final formatting and whitespace checks passed; the latest navigation
check reported 18 indexes, 1,018 nodes, 3,579 edges and no updates.

## Checklist

[done] PR brief and baseline recorded before test edits; two real-engine failures reproduced; research and preservation constraints recorded. [open] Resolve the single canonical authenticated binding and unambiguous identity with historical-data/recovery semantics; implement; rerun both oracles and owning suites; verify authorization, bounds, reopen, replay and corruption; review every changed file and final diff; regenerate navigation; candidate and protected CI. No roadmap or POA&M status is closed.

## 2026-11-10 candidate correction

Candidate run 42690947540 failed `direct_read_source_closure::causal_log_and_snapshot_reducer_uses_match_the_explicit_allowlist` in the engine-authority suite and the rrd-engine optional-feature job: the new named-source test reads the unstamped causal log once more to prove the runtime trace event carries the `named_vector` source link, so `runtime_vector_artifact_catalog.rs` now has two such reads where the allowlist permitted one. The allowance is raised to two with both oracles named; no engine source path reads the log. Local: the closure test passes, `cargo test -p rrd-engine --all-targets --all-features --locked` otherwise passed 12 of 13 targets before the correction with this the only failure, fmt clean.

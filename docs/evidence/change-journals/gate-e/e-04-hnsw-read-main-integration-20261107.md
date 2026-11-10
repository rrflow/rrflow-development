# HNSW generation reads: current-main integration

**Status:** active integration; E-04 and alpha remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-hnsw-read-main-integration-20261107`
**Owner:** E-04, OBJ-13 and [PR #278](https://github.com/rrflow/rrflow-development/pull/278)

## Boundary and integration

The branch started at `49bd2a0278a125c72e23d339cf55a1253add52b9`; the public main merge target was `b555d22c891c511775b5e54a9f9d00c8a51da48e`. The PR brief was updated before the normal merge. Its existing [implementation journal](e-04-hnsw-generation-read-path.md) binds the original real client/server characterization and failure oracle. This integration combines its generation candidate and authenticated exact rerank path with main's newer HNSW cost sampling and graph connectivity implementation. The original approximate semantics, shared read budget, authorization, recovery, and documented fallback paths must remain intact; an unproved semantic collision is a stop condition.

The merge preview identified one source file and the generated Gate E index. The three source conflict blocks added independent derived in-memory fields, construction, and initializer entries; both the collection membership and the cost sample were retained. `python scripts/knowledge/render_navigation.py` regenerated the combined index from its owning records. No persisted HNSW format or wire schema was changed by this resolution.

## Verification in progress

On Windows using isolated `CARGO_TARGET_DIR=C:/Users/jessa/AppData/Local/Temp/rrflow-hnsw-read-main-20261107`, `CARGO_INCREMENTAL=0`, and `CARGO_PROFILE_TEST_DEBUG=0`, `cargo test --offline --locked -p rrd-vector --lib hnsw` passed 23 tests with one large manual lab test ignored. The engine vector-index tests, client/server characterization, owning package checks, full changed-file reread, final diff review and exact-candidate CI remain in progress. This is local component evidence, not a project journey, performance comparison or alpha qualification. No roadmap or POA&M status changed.

## Combined-path results and a red source-identity oracle

`cargo test --offline --locked -p rrd-engine --lib vector_index` passed 65 tests before adding the new regression. `cargo test --offline --locked -p rrd-client --test vector_read_path -- --nocapture` passed both real local HTTP client/server tests; it observed 240 and 342 keys for the plain 256- and 2,048-vector cases respectively and 32 hydrated candidates in each. Those synthetic points do not prove recall on real embeddings or competitive speed.

After the PR brief was amended, `cargo test --offline --locked -p rrd-engine --lib an_emptied_hnsw_generation_does_not_answer_for_another_collection -- --nocapture` failed with exit 1: exact search returned the mirror record, but required-approximate search returned `empty foreign HNSW answered []`. The first tested test-file Git blob was `78627a138b8293fa1911f1f62fff473f43c9e275`; after rustfmt, blob `50ae307730acc1847bac3c8db1bc1788a55af1a3` reproduced the same failure with the same command. The tested merged working-tree `search.rs` blob is `7a199e2d1253f47ffa95bcb34c5d4e05a2f7c56b`, and `hnsw.rs` is `7c86915b47fb9bf776adb8bea147b0e3ceaef11d`. No integration commit or push has occurred. The candidate is now a draft and auto-merge is disabled.

An empty generation cannot establish its collection from its nodes, while its source cursor may include a later exit; therefore treating empty as universal makes a foreign source's earlier vectors invisible to the delta. Inspection of `VectorArtifactBinding` shows kind, descriptor and immutable object identity, but no explicit collection address. An authenticated durable source binding may cross this package's stated format/catalogue stop boundary; the exact design remains open. The finding is added to POAM-048 without changing its status. Do not convert this red oracle into a passing empty result or claim the combined candidate is ready.

## Two distinct source addresses collide

After the PR brief was amended, `cargo test --offline --locked -p rrd-engine --lib distinct_source_addresses_receive_distinct_hnsw_indexes -- --nocapture` ran against the same combined Windows worktree and failed with exit 1. Creating indexes for `(a-b, c)` and `(a, b-c)` returned `hnsw-a-b-c` for both. The formatted test file's Git blob is `7e527a34707f7b127a83041606012f02216f1925`; the unmodified-from-main `engine/vector/index.rs` blob is `0c5100a50775d49cc346bb992dd7142f95d1445f`. This is real-engine evidence on the combined candidate; a separate unmodified-main execution was not run. It demonstrates that the hyphen-joined ID is not a unique structured source identity. Do not infer ownership from that ID. The candidate remains draft.

## Next source-identity boundary

`rrd-vector/src/catalog.rs::VectorArtifactCatalogEntry` is a version-2 durable entry whose digest covers the descriptor, object and operation evidence, but not a collection or vector name. `rrd-engine/src/runtime/vector_catalog.rs` commits its catalog record and object together and replays both from the caller's authenticated change set; its `VectorArtifactBinding` drops any source address because the entry has none. `engine/vector/index.rs::ensure_vector_index` has the requested typed address while constructing and publishing the graph, but constructs the projection ID by joining the names with hyphens. That text alone is not an unambiguous structured ownership proof. The quantization lifecycle does carry a typed collection/name binding; it does not prove ownership of an HNSW entry.

A successor package must bind the actual named source to the authenticated generation publication and replay, preserve historical-stamp and restart semantics, explicitly handle existing version-2 entries without inventing their missing source, and reject or safely rebuild unknown ownership. It must test empty/nonempty generations, shared fields, ambiguous component names, source moves, independent writes, bounded reads, corruption, restart and replay. The durable representation and upgrade/admission behavior need a fresh brief and exact-source evidence before implementation. This is a documented next boundary, not an implemented repair.

## Change checklist

[done] Recorded exact starting revisions, owner, current/target behavior, stop conditions and integration brief in the PR before the merge. [done] Inspected the source conflict and preserved both independent structures; regenerated the index. [in progress] Review changed source and tests, run integrated oracles and owning suites, record failures and unrun work, complete the linked journal, and review the final diff before push. [open] Run candidate and protected merge-group CI; no gate closure follows from this record. No third-party source adaptation was made, and existing authenticated read evidence remains the debugging boundary.

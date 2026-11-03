# H-04 public vector index operations

**Status:** active immutable change-journal evidence; H-04, E-04 and OBJ-13
remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-vector-index-operations`
**Owner:** evidence for the operations that let a client build a graph index and declare a payload index

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: H-04 and E-04 for OBJ-13 and OBJ-10, C-05 for the catalogue identities. POAM-046, and the operation half of POAM-053.
starting revision/tree and pull request: bf0b86a4, this repository's main; pull request #271.
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the description of pull request #271. The engine implemented HNSW and payload index administration, but no catalogue operation reached them, and one published generation made every changefeed page of its scope fail. Now three catalogue operations reach them from every client, retries are safe, and the changefeed keeps working. How an index is built or searched is unchanged. The brief named VectorCollectionEnsure for all three operations; the payload index operations use the VectorPayloadIndexEnsure and VectorPayloadIndexDelete grants the engine already checks.
files and symbols read in full: docs/poam/rrflow-1.0-alpha/poam-046.md; POAM-053 on agent/comparison-lab; commit 59e48ebf; rrd-engine engine/vector/index.rs, engine/vector/collection.rs (ensure and delete payload index), runtime/vector_catalog.rs (publication and record decoding), engine/invocation.rs, capabilities.rs; rrd-vector catalog.rs, quantization_catalog.rs; rrd-contract lib.rs (vector types, endpoint_catalogue, CanonicalId), operation_reference.rs, sdk_conformance.rs; rrd-server http/router.rs, http/handlers/vector.rs, http/envelope.rs; rrd-client src/operation.rs, tests/sdk_conformance.rs, examples/sdk_conformance_server.rs; every shared conformance runner; scripts/ci/run_sdk_conformance.py.
implementation/evidence paths created, changed, moved, or deleted: created crates/transport/rrd-client/tests/vector_indexes.rs, crates/authority/rrd-engine/src/engine/tests/vector_index/index_identities.rs and this journal. Changed the catalogue writer and entry (rrd-vector catalog.rs and lib.rs, rrd-engine runtime/vector_catalog.rs), ensure_vector_index (engine/vector/index.rs, now taking a RequestContext and recording its key), the operation catalogue and reference text, the server router and handlers, the Rust client, the capability row, the shared conformance corpus, harness and six runners, the vector tests that call ensure_vector_index, two catalogue test fixtures that named projections with a colon, docs/reference/vector/hnsw-projection.md, POAM-046 and its ledger row. Generated: every SDK endpoint and signal surface, OpenAPI, docs/reference/protocol/signal-catalogue.md, the digest citations in docs/reference/protocol/public-contract.md and docs/reference/sdk/, the storage contract fixtures, and navigation.
research decision and primary-source/adaptation record: none needed. The engine code exists and no outside code is adapted.
trace/resource/debug decision (add/preserve/not applicable, with reason): preserved. The invocation audit records each call under its grant, publication keeps its durable trace span, and a refused identity finishes that span at the new catalog_identity stage. Results keep their build and read evidence. The budget stays max_storage_keys, bounded by MAX_VECTOR_STORAGE_KEYS.
first-failure or characterization oracle and result: each committed failing before its fix. 7b9c6068: the changefeed read refused the old identity, and the catalogue accepted a colon in a projection id. 965dd425: a retry after the source moved built generation 2. 6214e260: the Rust client got 404 endpoint not found.
acceptance commands and exact results: cargo test --locked -p rrd-vector, 72 passed; -p rrd-contract, 165 passed; -p rrd-client, 30 passed, including tests/vector_indexes.rs against a real server; -p rrd-engine, 399 passed; -p rrd-server --no-fail-fast, 33 passed and 1 failed because rrd-estate-controller was not built, then after cargo build -p rrflow-cli --bin rrd-estate-controller --locked, tests/local_estate_driver.rs 4 passed; -p rrflow-eval storage, 43 passed. cargo clippy --locked for those five crates with --all-targets and -D warnings, clean. cargo fmt --all --check, clean. Shared SDK conformance at corpus 40374996ef502161c9402320435b56a34c3f19e25cdba18378c67b4526fcdd49: Rust, Python, Go and .NET passed against one real daemon. TypeScript tsc --noEmit (which covers its runner) and biome check passed. python scripts/ci/check_generated_surfaces.py --check, 78 HTTP operations at OpenAPI 1d586cd72e7046890616cb7ab01dd26e544fa87d06db5d9f78d827a421092f16. render_navigation.py, build_execution_inventory.py, check_documentation.py, check_workflow.py and check_version.py passed.
surfaced failures and corrections: a payload rewrite first changed create_session instead of the index call in collection_moves.rs; that uncommitted file was restored and edited exactly. The first reference text was 747 bytes against a 600-byte limit, which failed the catalogue's validation in the client's build script, so it was shortened. My test first committed without a deadline, and the server correctly refused it. The pinned pnpm 12.14.0 launcher on this host is broken, so its pnpm.mjs entry point generated the TypeScript surfaces; the install itself was left alone.
checks not run and reason: the TypeScript conformance runner's live run (this host has Node 22.23.2, the SDK requires 24.22.0, and it stalls at its first await, as the H-04q2 journal records). The Java runner (Maven is not installed here, so its edit has not been compiled). Candidate CI runs both once this pull request is ready, and it has not run. The local conformance wrapper replaced those two commands with notices, so its log prints an OK line for each; those two lines are not results. dotnet run printed a NuGet SDK lookup message about SDK 10.0.402 on this host and then ran 5 tests with 0 failures. The rest of the workspace's suites, and any benchmark: the lab's rrflow-hnsw arm needs #253 and clyffy-01, whose login is awaiting the owner.
remaining known errors and owner: POAM-053 stays open, because its closure needs the lab to measure the arm through these operations (#253, clyffy-01). An index build runs inside one request, and the Rust client's default request timeout is five seconds, so a large build needs a longer one; job control belongs to D-06 and I-03. max_storage_keys is bounded at the existing limit, which caps the index a client can build, and that limit is the owner's open 1,000,000-key decision. An ensure that found the index already current published nothing, so its key binds nothing and that key can later build. The corpus digest cited in docs/reference/sdk/ and gate-a-evidence.md was already different from the fixture before this package, and no check enforces it, so it is left as recorded history. The server still has no public vector-payload-index-list or vector-collection-delete; vector-collection-list returns each collection's payload indexes.
full-file reread and diff review: complete for the engine and catalogue changes, the client test and this record. Generated files were checked by check_generated_surfaces.py and not read line by line.
change checklist: complete. Brief before implementation; one catalogue drives dispatch, OpenAPI, SDKs, capabilities and conformance; no new grant; no provider state; no parallel path; failing tests committed before fixes; no migration, because no public operation could create the old identities.
roadmap/POA&M status change, or explicit none: POAM-046 moves from Open to Verifying. No roadmap gate changes, and POAM-053's record stays on #253.
pull request, CI run, and merge evidence: recorded on pull request #271.
```

<a id="writer-review"></a>

## Writer review

POAM-046 requires a review of every engine-internal writer of records and
objects. A client reads only its own `instance:` scope (`query_scope`), so this
table covers what can reach a public changefeed page, and it says where a writer
cannot. It comes from reading each writer. No test exercises all of them
together.

| Writer | Identity it writes | Public changefeed |
|---|---|---|
| Vector artifact catalogue, `rrd-vector` `catalog.rs`, `rrd-engine` `runtime/vector_catalog.rs` | `{projection}.{generation}`, `{projection}.{generation}.vector-bytes` | Fixed by this package and checked before staging |
| Quantization artifacts and lifecycle, `quantization_catalog.rs` | `{projection}.{generation}`, `{projection}.{generation}.quantized-bytes`, `revision-{n}` | Canonical, fixed by H-04q2 |
| Client transactions, `engine/model/lower.rs` | The caller's `DataReference`, already canonical | Canonical. Event targets use `cursor:{n}`, which `raise.rs` projects as a typed event target |
| Prompt flights, `engine/flight.rs` | `flight_id`, `{flight_id}.{rank}`; events `cursor:{n}` | Canonical, with the same typed event target |
| Tasks, `engine/task.rs`, and reasoning budgets, `engine/reasoning_budget/persistence.rs` | `task_id`, `flight.id` | Canonical ids |
| Memory estate, `engine/memory_estate.rs` | `seat_id` | Canonical id |
| Attunement inventory, `engine/attunement/inventory/records.rs` | A digest, `entry-{digest}`, `snapshot-{digest}` | Canonical. Members are checked with `is_digest` |
| Runtime traces, `rrd-core` `trace.rs` | Event kind `runtime_trace` | A typed event target; the changefeed oracles read scopes holding these spans |
| Embedding jobs, `rrd-inference`, `edge.rs` | `job.target` | Only `edge.rs` builds a job, in the offline edge runtime's own `instance:edge` scope, which is not `RrdEngine`'s changefeed |
| Context maintenance, `rrd-maintenance` | `run.id` (`CanonicalId`), `active` | Canonical, and in the `maintenance:` scope, which no client reads |
| Distributed authority, `rrd-cluster` `authority.rs` | The cluster id | Written to `distributed_scope`, which no client reads, so the id's grammar does not reach the public projection |
| Artifact transfer, backup, and the S3 and object stores | Object receipts and test fixtures | They carry objects a writer above already named, or they are test code |

No writer above was found writing an identity the public projection refuses.

## Links

- [Pull request #271](https://github.com/rrflow/rrflow-development/pull/271)
- [POAM-046](../../../poam/rrflow-1.0-alpha/poam-046.md)
- [HNSW projection](../../../reference/vector/hnsw-projection.md)
- [H-04q2 the public quantization lifecycle](h-04q2-public-quantization-lifecycle.md)

# E-04 vector planner and external-process observations

**Status:** active supporting evidence; E-04 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journals/gate-e/e-04-vector-planner-external-observations`
**Owner:** E-04 package driver

This record supplements [the bounded candidate selection journal](e-04-bounded-turboquant-selection.md)
and [draft PR #273](https://github.com/rrflow/rrflow-development/pull/273). It
does not independently change product or release status.

Research evidence, not a roadmap or release-status owner. Raw local artifacts
referenced below are under the original checkout `target/` and are not part of
the repository or candidate CI. Public main at inspection was commit
`e012a51d4c01975ddab70538df14333a70ba7aed`, tree
`95ebae426247adbe05a473e26ef5d6b17a496d6d`. The root checkout was older and
was not used as the current-code baseline. No production data was used.

## Coverage

An inventory of the clean detached checkout contains 2,143 available tracked
non-submodule files, including 676 `.rs` files. The reported total is 598,380
newline-delimited lines across these files, including non-code and possible
binary files; it is not a count of reviewed lines. Inventory file
`rrflow-research-line-inventory-e012.json`, SHA-256
`1afbb19dd0ecac9076848e64746fecccd676cee99fd8114d4ce90f8b462c6a1e`,
records paths and index blob IDs. Submodule contents, untracked files, and
any tracked path unavailable in this checkout are outside that count.

Actually read in full at the main revision for this pass:

- `crates/compute/rrd-vector/src/plan.rs`, 394 lines, Git blob
  `0936be944bdd208ffc33488a1ea6d246cbd463b0`.
- `crates/compute/rrd-vector/src/runtime.rs`, 1,196 lines, Git blob
  `37c08a33725324801ecf74d737803c96620a72b7`.
- `crates/compute/rrd-vector/src/filter.rs`, 316 lines, Git blob
  `1d150b87a58f5250efe11fb6e0289c5a1afad3a1`.

No whole-codebase line-review claim is made. The remainder of the inventory has
not been reviewed in full during this pass. Prior work also reviewed the
TurboQuant segment candidate file at candidate commit `6f23b764`, with its
separate journal in draft PR #273; that is a different source identity.

## Code-grounded findings

The planner checks request/stamp validity, source coverage, model/field/metric
compatibility, required filter-property coverage, approximation mode, and
selects the lowest caller-supplied estimated cost. It records rejected paths.
`VectorRuntime::prepare_search_at` currently assigns TurboQuant an estimated
cost equal to candidate-version count, and adds counted overlay versions. It
does not derive a filter-selectivity estimate at this boundary. This is not
proof that no other component has statistics. `execute_search` verifies the
request, catalogue revision, newly computed plan and published descriptor,
then exact-reranks TurboQuant's selected references plus covered delta
identities. This is a real component path, not proof of automatic model
routing or project indexing.

Qdrant describes payload-aware strategy choice and quantization/rescoring in
https://qdrant.tech/documentation/search-patterns/vector-search-filtering/
and https://qdrant.tech/documentation/manage-data/quantization/. A candidate
planner improvement must use measured, source-bound and scope-safe estimates;
exact reranking cannot recover an identity absent from the candidate set. The
comparison does not establish that copying Qdrant or forcing approximation is
correct for RRFlow.

## Real processes

The existing independent smoke harness was copied to a new local path before
execution, preserving its prior result. SurrealDB 3.3.0 image digest
`sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20`
committed two synthetic records and an edge in one transaction and returned
the same relationship after container restart. Qdrant 1.20.0 image ID
`sha256:9bfc89f9914fc14876a88c05270685921a55b8877b254a98bcc1cc2f0a424cd2`
accepted a bits4 configuration and returned the same filtered two results from
three vectors after restart. Output `rrflow-external-db-smoke-result-rerun.json`
has SHA-256 `f603ba8225d56aecaca5abfdccf3012e019715b8ac6766cca4072ef4b3991c73`.
This does not prove an optimized quantized execution path.

A separate Qdrant-only diagnostic inserted 3,000 deterministic random 64-D
vectors, with 1,000 in the queried scope, enabled bits4, waited for the service
to report 3,000 indexed vectors and green/ok status, and compared 12 filtered
approximate queries (rescore enabled, oversampling 2) with Qdrant exact search.
The mean top-10 overlap with exact was 0.9917 before restart and 1.0 afterward.
Exact ID order matched on all 12 queries before/after; approximate ID order
matched on 11 of 12, with one candidate differing on the remaining query.
The harness's strict equality assertion failed (exit 1). A cause is not known;
the experiment did not test repeated queries within a process, segment
optimization timing, or verify the exact planner/quantization path. Do not
interpret the difference as data loss or a product defect. No latency,
competitiveness, realistic embedding, relevance or RRFlow parity claim follows.

Raw `rrflow-qdrant-recall-research.json` SHA-256
`96a20a984eaea55280f1e693333df75ba409df7749c009855f6691b8f2456f22`;
script `rrflow-qdrant-recall-research.py` SHA-256
`47ebd75d4b49c348fa9d471820a56ad70b4c7dde59ed242e09b96631ea209c71`.
Disposable containers were requested for removal in finally blocks. The larger
run finished its cleanup before its failure was returned. No remote server or
external account was used.

A second fresh collection run of the same workload repeated all 12 queries
within each process and after restart. Both exact and approximate ID orders
matched across all four rounds, with mean overlap against exact 1.0 in each.
The service reported 3,000 indexed vectors but also yellow status and three
segments on first observation, yellow and four segments immediately before
restart, and grey and four segments immediately after the subsequent queries;
optimizer status was `ok` throughout the recorded snapshots. These observations
are not steady-state qualification and do not explain the first run's change.
The follow-up exited 0. Raw `rrflow-qdrant-recall-research-v2.json` SHA-256
`20f9b43c67dfc2197e98d49943a1fab2237807327a41ecad85e44b1f6a5638b1`;
script SHA-256
`576ba7dc8547b2aebcc34815e79652e3a8b0b612f503a01dc077009ddc9cc818`.

Host available physical memory fell to 3,084 KiB while other work was active;
the exact allocation attribution was not measured. The follow-up was started
after free physical memory recovered to roughly 17.5 GiB. Next experiment:
wait for a stable optimizer state, repeat across independent collections and
real embedding corpora, capture actual query-path instrumentation, and compare
with the authorized RRFlow path.
At the time of these observations, the installed RRFlow quantization test on
PR #273 was interrupted. The subsequent ARM64 installed test and its limits are
recorded in the [bounded candidate journal](e-04-bounded-turboquant-selection.md).
Candidate CI and merge remain unverified here; C-07 and alpha remain open.

## Evidence maintenance

The generated Gate E index was refreshed with
`python -X utf8 scripts/knowledge/render_navigation.py`; its `--check` and
`python -X utf8 scripts/ci/check_documentation.py` passed. The staged diff
passed `git diff --cached --check`, and the generated execution inventory was
rebuilt and checked. These checks validate repository metadata, not the
external products or the whole-codebase review. No runtime code was changed by
this research note and no roadmap or POA&M status changed.

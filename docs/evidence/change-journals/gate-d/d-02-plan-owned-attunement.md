# D-02: bind attunement progression to its plan

**Status:** active implementation candidate; integration and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-plan-owned-attunement`
**Owner:** OBJ-14 / D-02 prerequisite / POAM-039 / [PR218](https://github.com/rrflow/rrflow-development/pull/218)

## Brief and scope

Starting development main `6fe8b7590d913beda9b009551e5d2d2644e406d9`, tree `3bd842a6132a2815fab0133988898ec2483b6293`. The draft PR carried the bounded brief before repository edits and records its later versioning, fixture, schema and protocol-digest discoveries before those edits. Its initial base is the already merged `codex/ci-main-cache-independent`; retarget main after the implementation commit. [PR208](https://github.com/rrflow/rrflow-development/pull/208) changes the same contracts and must be integrated deliberately, with ordinary merge and fresh exact-revision CI.

Previously plan validation and job progression required the global eleven-entry `ATTUNEMENT_PHASES`. This candidate binds checkpoints, next-step selection and resume to the exact positions, recognized phase kinds, configuration hashes and plan digest of the stored plan. It supports repeated kinds and requires exactly one final Verify step, so omitted steps do not themselves produce a successful job. The default profile still supplies eleven entries. The only implemented attunement step remains Connect; Verify and other unavailable steps fail explicitly. There is no public custom-plan submission or per-prompt planner in this package. This is a bounded linear attunement contract, not a dynamic reasoning graph, source freshness check, automatic project integration, SWAG routing, handoff, or alpha completion.

Attunement envelopes now require version 2. Installation-result and restore-publication contracts retain version 1; the default plan content is unchanged but its identity and dependent installation digests change. Version-1 stored attunement plans and clients are not accepted by this candidate; no transition or upgrade path is provided. Do not assume an existing estate can resume its old job after this change. The resource cap of 32 remains an explicit finite bound, not a requirement that all projects execute the same steps.

## Research and decisions

[Temporal workflow definitions](https://docs.temporal.io/workflow-definition) require deterministic durable replay and put nondeterministic operations in activities. [Temporal task history](https://docs.temporal.io/tasks) describes replay using recorded activity results. [LangGraph's orchestrator-worker example](https://docs.langchain.com/oss/python/langgraph/workflows-agents#orchestrator-worker) describes runtime-created workers; [its persistence documentation](https://docs.langchain.com/oss/python/langgraph/persistence) describes checkpoints. These are design references, not evidence that RRFlow has those capabilities; no upstream code is copied. [Schemars field customization](https://docs.rs/schemars/latest/schemars/derive.JsonSchema.html) and [JSON Schema 2020-12](https://json-schema.org/draft/2020-12) informed the schema projection. Schema bounds/version do not authorize effects or prove digest, source or replay safety.

Existing engine authorization, leases, receipt and control-store persistence remain the trace and effect boundary. The installed-engine test exercises those boundaries and reopens the store. No resource or performance improvement is claimed, so no benchmark was introduced. C-07's native matrix and soak remain separate requirements; short characterization results are not qualification.

## Paths and structural traceability

Implementation and tests: `crates/transport/rrd-contract/src/attunement.rs`, `src/lib.rs`, `tests/attunement_contract.rs`, `examples/refresh_installation_contract.rs`, `crates/authority/rrd-engine/src/engine/{attunement,installation}.rs`, `src/engine/tests/attunement.rs` and `crates/transport/rrd-client/tests/rrflowdb_local.rs`. The `install-attunement-v1.json` characterization fixture moved to `install-attunement-v2.json`; its canonical owner remains the contract crate. The existing golden test and refresh example are its characterization and generator. Version-1 fixture content remains in Git history. The regenerated fixture changes attunement versions and dependent digests, retaining installation-result version 1. Replacement evidence is the golden test plus installed reopen tests; neither proves an upgrade path.

OpenAPI, SDK endpoint and signal outputs are generated from the contract exporter and existing generators under `sdks/{typescript,python,go,java,dotnet}` and `scripts/ci/check_generated_surfaces.py`. The public-contract reference and five SDK references record the new digest without rewriting earlier audit digests. `docs/roadmap/rrflow-1.0/gate-b-evidence.md` identifies its original fixed-order description and fixture as historical without changing its status. This journal and its generated navigation index are evidence, not another lifecycle authority.

Reviewed owners and boundaries include `README.md`, `docs/decisions/0002-adaptive-reasoning-governed-effects.md`, `docs/roadmap/rrflow-1.0.md`, `docs/roadmap/rrflow-1.0-execution/change-authoring.md`, `docs/poam/rrflow-1.0-alpha/poam-039.md`, the above implementation and tests, and the surface generators. The complete changed-path inventory and digests are derived by Git and `scripts/ci/build_execution_inventory.py`; a mechanical file inventory is not a claim of line-by-line semantic review of the whole repository.

## Evidence and failures

The first focused variable-plan contract test failed against the old global eleven-step rule, then passed after the change. A temporary moved-value compiler error in the test fixture was corrected. An initial restore filter matched zero tests and is not counted as evidence; the named restore test below actually ran. A schema projection check initially reported the old generated digest; the owning generators produced `fb2bf476a766d125f24b1b68617f42df7edaef75292c968da1cf43f6c73fb7b0`. Python generation initially lacked `ruff` on PATH, then succeeded using the locked offline `uv` environment. `pnpm install --offline --frozen-lockfile --ignore-scripts` failed in the local pnpm launcher; TypeScript generation used an existing dependency installation with the same lockfile and succeeded. The first navigation render rejected the journal because PowerShell wrote a UTF-8 BOM before its heading; rewriting it as UTF-8 without BOM corrected the record. The first full contract suite failed its OpenAPI-digest assertion: the actual advertised constant still had the old digest. The constant was updated to the reviewed exporter output; the assertion remains intact. The first staging command named the already staged old fixture path and Git rejected that pathspec without staging the remaining paths; the corrected command staged the new path and retained the recorded rename. `gh pr edit` hit a GitHub GraphQL error; the brief was successfully updated through the GitHub REST API before the following edit.

The new engine oracle inserts a test-only `[Connect, Connect, Verify]` plan directly into installed rrflowKV control storage; it does not represent a public plan creation API. Through `RrdEngine`, it starts, checkpoints the first step, pauses, closes/reopens, resumes, executes the second configured Connect and checks its digest chain; Verify then fails with `phase-unavailable` and no success. Existing native worker-kill/replay and receipt tests exercise the prior default path.

- `cargo test --offline --locked -p rrd-contract --test attunement_contract`: 16 passed.
- `cargo test --offline --locked -p rrd-engine --lib attunement`: 13 passed, one ignored native child fixture; the parent worker-kill/replay test passed.
- `cargo test --offline --locked -p rrd-engine --lib a_staged_restore_is_published_in_place_and_the_estate_opens_from_the_backup`: one passed.
- `cargo test --offline --locked -p rrd-client --test rrflowdb_local`: four passed, including installed relation/query atomicity and reopen, attunement read and cancellation replay, and rejected truncated/foreign Parquet export.
- `python scripts/ci/check_generated_surfaces.py --check`: passed, 62 operations and the digest above.
- TypeScript `tsx scripts/generate.ts --check` and `tsc --noEmit`: passed using the matching-lockfile local dependency tree.
- `cargo test --offline --locked -p rrd-contract`: passed in full after the advertised OpenAPI digest correction.
- `cargo test --offline --locked -p rrd-engine --lib -- --test-threads=2`: 247 passed, three ignored; includes the new plan/reopen test and existing TurboQuant tests. The ignored native child and privileged platform fixtures are not independent passing evidence.
- `cargo clippy --offline --locked -p rrd-contract -p rrd-engine -p rrd-client --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed before the final digest-only edit.
- `python scripts/ci/check_documentation.py`, `python scripts/knowledge/render_navigation.py --check` and `python scripts/ci/build_execution_inventory.py --check`: passed after the historical reference update.

Candidate and protected merge-group CI outcomes are pending at the time of this record. No production project, source mutation, external provider, native capacity matrix, 100,000-record result, long soak or release candidate was tested here. Full-file change review covers the authored diffs and generated projection changes; generated 40,000-line TypeScript output was checked by its generator, not manually audited line by line.

## Change checklist and status

- [x] Owner, starting revision, current and target behavior, exact scope and stop conditions recorded in PR before the corresponding edits.
- [x] First-failure oracle, installed engine reopen, error and unsupported-step behavior retained.
- [x] Research and schema/effect boundaries recorded; no source adaptation, provider state, or second engine authority.
- [x] Fixture move mapped to canonical replacement and characterization.
- [x] Contract package, engine library, generated-surface, lint and documentation/navigation checks ran as reported above. Authored diffs and generated changes were reviewed; a line-by-line review of generated TypeScript is not claimed.
- [ ] Candidate CI, protected merge-group CI and merge: pending.
- [x] Roadmap and POA&M status changes: none. POAM-039 remains open. No merge evidence exists yet.

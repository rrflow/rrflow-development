# D-02: replace fixed attunement defaults with engine selection

**Status:** active draft; selection is not implemented
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-adaptive-attunement-20261107`
**Owner:** OBJ-07/14, D-02/D-03/D-05, [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md), [PR #299](https://github.com/rrflow/rrflow-development/pull/299)

## Baseline and requested behavior

Baseline public main `81d2166311476ef51703532573908024a61b725a`; draft
bootstrap `b2e2216339f502d74c2ff303462edbbe4f9a4f2b` has no source changes.
The PR brief was written before the added local test. The push destination was
`development`, `https://github.com/rrflow/rrflow-development.git`, ref
`refs/heads/agent/adaptive-attunement-selection-20261107`, with no prior ref.

Both installation profiles currently contain the same eleven entries. The
installer copies that list into the default content-addressed attunement plan
and recomputes it when validating the sealed preview. The public contract
requires the last selected entry to be Verify. The installed dispatcher has
Connect and Inventory execution paths and fails other entries as unavailable.
This is neither dynamic project/prompt planning nor evidence of a working
project index. The user directed removal of the fixed default, not replacement
with another disguised fixed list.

Code inspected: both full profile assets; `engine/installation.rs` profile,
proposal, plan construction, sealed-preview and installed-binding sections;
`rrd-contract/src/attunement.rs` plan and job validation/transition sections;
`engine/attunement.rs` dispatch and checkpoint sections; and
`engine/attunement/inventory/publication.rs` group and final-commit sections.
This is a boundary review, not a whole-file or repository-wide review.

## Research and actual comparison observations

[SurrealDB's architecture](https://surrealdb.com/docs/learn/data-models/architecture)
describes document, relation and index changes under the same transaction;
its [v4 transaction API](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/kvs/src/api.rs)
also distinguishes a committed-but-not-yet-synced path from durability. The
same repository's [runtime commit owner](../../../../crates/persistence/rrd-store/src/repository/runtime.rs)
composes semantic and control transitions in one transaction. That code is a
real foundation, not proof that a multi-transaction project refresh is atomic:
the existing [#252](https://github.com/rrflow/rrflow-development/pull/252)
oracle exposes partial refresh facts via current reads. All relevant readers
must resolve the same completed generation, including after interruption and
reopen; an individually atomic batch does not make all batches atomic.

[SurrealDB Agent Memory](https://surrealdb.com/docs/agent-memory/architecture/traces-and-evolution)
describes provenance, input-sensitive response reuse and background evolution;
its [product page](https://surrealdb.com/agent-memory) says that memory layer
is closed source. These are observable design ideas, not source we inspected or
copied. Exact dependency invalidation and durable receipts are RRFlow design
requirements, not established RRFlow behavior.

[ArangoDB's indexing documentation](https://docs.arango.ai/arangodb/stable/indexes-and-search/indexing/basics/)
describes edge indexes; its [query statistics](https://docs.arango.ai/arangodb/stable/aql/execution-and-performance/query-statistics/)
distinguish execution time, scan counts, per-operator data and memory limits.
Its [ArangoSearch documentation](https://docs.arango.ai/arangodb/stable/indexes-and-search/arangosearch/)
describes eventual consistency and `waitForSync`; a workbench must disclose
freshness rather than assume every search index is transaction-current.

Local functional observations, not benchmarks: SurrealDB 4.0.0 image digest
`sha256:c35893410644e65fa90aa1bef446ac9cb0a0c2733b9d405ce3eb56754b85c330`
ran an in-memory, network-disabled three-record filtered HNSW query. It returned
two expected records; EXPLAIN reported `KnnScan` with the pushed predicate.
ArangoDB 3.12.13 image digest
`sha256:e48adfc58986a10e0316aa23ccae95ca580d880a41c76a8f0df68f9c29b470f2`
ran in a network-disabled, tmpfs-backed container with no host data mounts;
a two-document, one-edge traversal returned `Grace`, with `TraversalNode` in
the plan and `scannedIndex=2`, `scannedFull=0` in its profile. The owned
container was stopped successfully. These tiny synthetic observations do not
establish matched performance, full persistence, safety of draft adapters, or
an RRFlow alpha.

## First local oracle and current state

The uncommitted `rrd-contract/tests/attunement_adaptive_plan.rs` constructs
independently digested selected-effect plans without a Verify suffix and checks
that empty, malformed-sequence and tampered plans still fail. This isolates the
public structural constraint; it does not test capability discovery, permissions,
execution, project-indexing completion or absence of a disguised fixed list.
On Windows x86-64, `CARGO_TARGET_DIR=C:/Users/jessa/AppData/Local/Temp/rrflow-adaptive-attunement-target-20261107`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`, the command
`cargo test --offline --locked -p rrd-contract --test attunement_adaptive_plan`
failed: the no-Verify case rejected `[Connect]`; the malformed/tampered plan
test passed. The command exited 1. This failure exposes the forced terminal
step; it does not establish a working adaptive selection or project workflow.

Before relaxing the plan validator, inspection of
`AttunementVerification::validate_for` exposed an implicit dependency on that
old invariant: Passed checks only succeeded state and the last checkpoint's
manifest. Once the plan can end in another selected effect, a forged Passed
verification could bind an unrelated last checkpoint. PR #299's brief was
amended before production edits to require an actual Verify checkpoint and a
negative oracle. Final selected-effect completion must also atomically commit
its checkpoint and receipt, and must not assert full indexing or verification.
No production implementation, full package suite, candidate CI, merge, or
alpha qualification is claimed.

## Remaining scope and checklist

[done] PR brief and starting revision; [done] inspect actual profile, contract,
dispatcher and inventory publication boundaries; [done] primary-source
research and isolated competitor observations; [open] record first oracle,
define versioned capability and unavailable-reason ownership, preserve existing
installed plans and interruption recovery, implement source-bound selection and
honest terminal state, run real installed driver and package checks, review all
changed files and diff, regenerate navigation, and pass candidate and protected
CI. No roadmap or POA&M status has changed.

## Source review coverage and implementation handoff, 2026-11-07

This is supporting evidence for the existing owners, not a replacement roadmap.
The reviewed tree is `be16e108a1660459e1d64d1a50384e750bde49d6` at
`b2e2216339f502d74c2ff303462edbbe4f9a4f2b` (same source as public main
`81d2166311476ef51703532573908024a61b725a`). A Git-derived inventory of tracked
code/config extensions reports 1,315 files and 523,120 lines, excluding
submodules; the local file/digest inventory is
`C:/Users/jessa/AppData/Local/Temp/rrflow-source-inventory-b2e22163-20261107.json`.
This inventory is **not** a line review. The rest of those files and almost all
upstream source have not been individually reviewed. Continue the requested
exhaustive review by recording exact revision, file and line ranges, findings,
test oracles and remaining ranges in the relevant package records; do not infer
coverage from searches, filenames, generated summaries, or tests.

Newly inspected RRFlow source ranges and blob identities:

| Boundary | Source and observed behavior | Consequence |
|---|---|---|
| Plan contract | `crates/transport/rrd-contract/src/attunement.rs`, blob `0c5ee80aa3936cb63e31818f5c8f48acdcfa5429`, lines 578–603, 1266–1298, 1587–1654 | Validation mandates exactly one terminal Verify. Replan permits a new source but requires all other plan content to match. Passed verification relies on the first invariant rather than independently checking the last checkpoint phase. |
| Engine planning | `crates/authority/rrd-engine/src/engine/installation.rs`, blob `e6b17ab0ed9c4b47615ea6807c61fb0104eea748`, lines 141–205 and previously inspected plan/sealed-preview paths; both complete profile JSON assets | Both profiles supply the same eleven phases; proposal validation separately imposes terminal Verify. Removing one validator alone does not alter default selection or installed execution. Profile bytes are also bound into installed identity. |
| Replan | `crates/authority/rrd-engine/src/engine/attunement.rs`, blob `ed67bd070e3d6542524c2f91f9d29af1daccfea6`, lines 278–337 and dispatcher paths | Replan clones the old plan with a fresh source identity; it cannot reselect capabilities. The worker provides Connect and Inventory paths and reports other requested phases unavailable. |
| Transaction | `crates/persistence/rrd-store/src/repository/runtime.rs`, blob `039fc92bdfa3ba8c78f53a090443ebcc0ad77e59`, lines 277–323 | Semantic and control effects can be staged and committed in one authoritative transaction. Native I/O uncertainty still requires reconciliation. This does not make a multi-transaction inventory atomic. |
| Relationship writes | `crates/persistence/rrd-store/src/access/semantic_commit.rs`, blob `5e7f43f4ccc22d69a6ab47e6bd5e95ca14d67308`, lines 537–564 and 808–883 | A relation mutation stages current/versioned relation and incoming/outgoing adjacency entries. This is actual code, not proof of an efficient, complete traversal at scale. |
| Relationship reads | `crates/persistence/rrd-store/src/keyspaces.rs`, blob `92a9d0cd3fd7378d08dd313782bbe9c76051e9fc`, lines 600–665; `access/versioned_read.rs` lines 860–878 | Typed endpoint-and-relation prefixes select temporal adjacency ranges. Those ranges include historical versions and retirements; total traversal work depends on range seeks, versions, filtering and caching as well as returned edges. No unconditional O(E) or database-size-independent performance claim is established. |

`access/versioned_read.rs`, blob `010639ed875c126a241afce665d2f35806e62ee6`,
lines 297–344, 565–616 and 716–756 additionally
show a single read stamp and key budget, per-entry JSON decode and proof
authentication, owned key/value batches, and repeated bounded range scans of
up to 1,024 rows. These are concrete differences from the reviewed SurrealDB
borrowed streaming cursor; they do not by themselves quantify cost or prove
that RRFlow should remove authentication. Measure the installed path before
changing layout, batch size or ownership. The new replan deficiency is recorded
in the existing [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md); its
status remains open.

The upstream comparisons were against ArangoDB `v3.12.13`, commit
`aaf43d548368e421fbf57abf4add4f5e6cf25c6d`, and SurrealDB `v4.0.0`, commit
`284fe93afc851c3baec2c262c01289602021a70d`. Specifically reviewed upstream
ranges: ArangoDB [`RocksDBEdgeIndex.cpp` lines 517–564, 753–786 and 883–915](https://github.com/arangodb/arangodb/blob/v3.12.13/arangod/RocksDBEngine/RocksDBEdgeIndex.cpp):
bounded endpoint lookup, index writes and cache invalidation; SurrealDB
[`kvs/src/api.rs` lines 401–430 and 544–583](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/kvs/src/api.rs):
bounded borrowed cursor iteration, explicit durability distinction, and
conflict-tracked reads. These are design comparisons, not a complete review of
either source tree or permission to copy code; ArangoDB's file identifies a
Business Source License. RRFlow should measure its own bounded scans, history
amplification, read stamp, permission filtering and cache behavior before
choosing an optimization.

### Dependency-ordered work for an actual project journey

These are engineering packages with different owners, not fixed runtime
reasoning phases. A package is complete only when its owner accepts the named
evidence; the numbered list is not an attunement plan.

1. **D-02: capability-selected plan and effect boundary — PR #299.** Replace
   profile-list selection in `installation.rs` with engine-owned selection
   from discovered, actually executable capabilities, effective configuration,
   project observations and explicit authorization. Keep unsupported entries
   visible with reasons without scheduling a pretend success. Version and bind
   source observations, selection/configuration, budgets, allowed effects and
   plan digest; retain independent validation and receipts for each durable
   effect. Replan from current relevant inputs, not by cloning old phases; bind
   predecessor, generation, digest and authorization. Ordinary relevant edits
   create a reviewable successor instead of permanently trapping the project;
   irrelevant edits must not invalidate unrelated effects. Preserve old sealed
   plan/profile readback and interrupted installations with explicit tests.
   A job ending after inventory must not issue a verification claim: validation
   of Passed verification must independently require real verification evidence.
   Test different project/config/capability inputs, unknown effects, denied
   authorization, tampered digest, ordinary edits, interruption, replay,
   restart and historical audit. Remove both the profile-derived default and
   duplicated mandatory-terminal validators as one coherent change; do not
   replace eleven phases with another fixed runtime sequence.
2. **D-03: publish one complete inventory generation — PR #252 dependency.**
   Repair the observed intermediate-batch visibility defect in the canonical
   source/read-stamp boundary. Validate every reader, including historical
   snapshots, scans, relationships, exports and derived index/retrieval inputs,
   against the same completed generation, retaining unrelated committed data.
   Use bounded, authenticated non-omission; test multi-batch failure and kill,
   concurrent reads and writes, reopen, replay, source deletion, and stale
   publication. The known 2,048-file oracle must pass before claiming a full
   project generation is atomically visible.
3. **D-03/D-05: real source to recall.** Reconcile parser work in PR #241
   with the same authorized source identity and completed generation. Carry
   typed project documents and provenance through parsing, relation publication,
   lexical/vector projections and actual query/retrieval as available; report
   unsupported formats or adapters, partial coverage and freshness explicitly.
   Keep relational rrflowQL as the query surface and relationships behind the
   shared transaction/read model. Test a user-authorized real project, edit,
   delete, incremental refresh and restart, with independently read results.
4. **C-07 and Gate C/E: storage, relationships and analytical paths.** Profile
   the retained installed 100,000-record workload and complete independent
   readback without relaxing limits, then run required native capacity and
   actual soak gates. Compare matched endpoint traversals as database size,
   degree, historical versions and concurrency vary; record seeks, scanned
   versions, bytes, latency, memory and correctness. Separate durable WAL and
   transactional lookups from optional Parquet analytics; an in-memory buffer
   alone is not a crash-durable WAL. The bundled Parquet query export described
   by README is not evidence of cold-tier compaction. Add cold-storage behavior
   only through its owning storage contract with atomic publication, recovery,
   resource limits and independent readback.
5. **Connectome and optional integrations.** Use engine-owned typed operations
   and read-only diagnostics for progress, source coverage, configuration,
   durable receipts, provenance and relationship visualization. Keep the
   Connectome repository boundary and default engine independence intact. LFG,
   SWAG and other endpoints are explicitly configured, authorized adapters;
   missing services cannot be represented as successful reasoning. Prompt
   effort, resource budgets and ephemeral context are configurable; durable
   engrams have observable governed writes. No hidden provider interception.
6. **Matched workbench and release evidence.** Use isolated, version-pinned
   ArangoDB/SurrealDB/RRFlow instances and a permissioned, held-out real Rust/Go
   project corpus. Record dataset and adapter revisions, workload, hardware,
   configuration and data freshness; independently score grounding, task
   correctness and output quality alongside latency, resource use, tokens and
   cost. Do not compare a database traversal with an LLM task as if they were
   equivalent, or infer product superiority from the synthetic smoke probes.
   Only pass integration and C-07 gates after the installed joined driver,
   actual native matrix/soak, candidate CI and protected merge-group checks pass.

The negative verification oracle has since been added locally, but has not been
run. No production file was changed, no installed joined journey was completed,
and no candidate CI or merge is claimed by this record.

Local documentation verification: `python scripts/knowledge/render_navigation.py`
updated the Gate D journal index; `python scripts/knowledge/render_navigation.py
--check` passed (18 indexes, 1,022 nodes, 3,590 links) and `python
scripts/ci/check_documentation.py` passed (1,023 document statuses). `git diff
--check` passed for tracked changes. The current worktree also contains the
untracked test and this journal; their presence is not candidate CI evidence.

## 2026-11-09 absorption onto main

The two contract tests that describe the target behaviour fail on main as
intended: `AttunementPlan::validate` still requires exactly one final Verify
phase, and no selection exists. The owner directed that every authored line
be absorbed into main rather than left on a branch, so both tests land
marked `#[ignore]` with the POAM-006 reason in the attribute. They are the
red oracle for D-02; un-ignoring them is the first step of the package that
implements selection. The third test, which checks the unchanged rules,
runs. `cargo test -p rrd-contract --test attunement_adaptive_plan`: 1
passed, 2 ignored. Clippy, fmt and documentation policy pass.

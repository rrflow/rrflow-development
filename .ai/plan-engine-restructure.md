# Plan: restructure the workspace

**Written:** 2026-11-26. **Status:** planned, not started. **Replaces:** section 6 of `docs/architecture/code-structure-and-conventions.md` (see [lessons.md](lessons.md) for why).

Pre-release. No compatibility, no legacy names, no shims. A rename or move happens in one pass; the compiler and the tests show what broke; it is fixed in the same branch.

## 1. Target layout

Modelled on SurrealDB's repository: top-level folders named for the product, one flat level of crates under each, short folder names, and the package name is always `<group>-<folder>`. No tier folders (`foundation`, `authority`, `operations`, `persistence`, `transport`, `adapters`, `compute`, `storage` all go). No crate nested inside another crate's folder.

```
Cargo.toml              one workspace, one lock file, every first-party crate declared once
rrflow/                 the database's components        package = rrflow-<folder>
  core/                 shared value, record and schema types; identifiers; digests
  lsm/                  rrflowLSM  hot disk persistence
  cld/                  rrflowCLD  cold and warm columnar storage
  mx/                   rrflowMX   in-memory working state and memory indexes
  tx/                   rrflowTX   stamps, MVCC, commit, conflicts
  kv/                   rrflowKV   the one key-value port over memory, hot and cold
  vector/               ANN, TurboQuant, quantized segments
  ql/                   rrflowQL   parser, planner, execution
  cx/                   rrflowCX   cluster transport, consensus, replication
  cx-operator/          the Kubernetes operator for rrflowCX
  db/                   rrflowDB   the multimodal coordinator on top
  inference/            model endpoints and embedding calls
  attunement/           project discovery and inventory
  engram/               per-project engram lifecycle
  security/             principals, tokens, authorization
  observability/        signals and delivery                       (today rrd-observation)
  knowledge/            operator knowledge                         (today rrd-operator-knowledge)
  cli/                  the `rrflow` command
  mcp/                  the MCP adapter
  edge/                 the edge adapter
rrd/                    the daemon, RRD: the running service       package = rrd-<folder>
  engine/               composes the components into one running instance
  server/               the daemon's network surface and process
  contract/             the wire contract a caller and the daemon share
  client/               how a caller reaches the daemon
lab/                    measurement from outside the product       package = rrflow-lab-<folder>
  core/ systems/ datasets/ benchmarks/ evals/ definitions/ runner/ experiments/ qualification-storage/
```

### 1.1 Component dependencies (the owner's table, 2026-11-26)

| Crate | Depends on (components) |
|---|---|
| `rrflow-lsm` | none |
| `rrflow-cld` | none |
| `rrflow-mx` | none |
| `rrflow-tx` | lsm, mx |
| `rrflow-kv` | tx, lsm, cld, mx |
| `rrflow-cx` | kv, tx |
| `rrflow-ql` | kv, tx, mx |
| `rrflow-db` | all of the above |

`rrflow-core` sits under all eight and depends on no first-party crate. "None" in the table means no component; every component may use `core`. `rrflow-vector` depends on kv and mx and is used by db. Everything else in `rrflow/` depends on `db` or lower. `rrd-engine` composes them; `rrd-server` depends on the engine; `rrd-client` depends only on `rrd-contract`.

### 1.2 Rust structure rules, enforced by the checker

1. A crate is a folder with `Cargo.toml` and `src/`. Its folder is `<group>/<name>` and its package is `<group>-<name>`.
2. `src/lib.rs` holds module declarations and re-exports only.
3. A module is `name.rs`, or `name.rs` plus a `name/` folder holding its parts. No `mod.rs`, no `#[path]`.
4. A folder name says what is in it. No `utils`, `common`, `helpers`, `misc`, `kernel`, `foundation`, `shared`, `base`.
5. Unit tests sit beside their module as `name/tests.rs`. Tests that use only the public API sit in `tests/`.
6. A first-party dependency is declared once in the root manifest and used with `workspace = true`.
7. A crate's first-party dependencies are exactly those the manifest allows (section 1.1). An extra one is a hard error.

### 1.3 Keeping `core` from becoming a dumping ground

- `core` has a fixed list of top-level modules in the architecture manifest. A new one is a manifest edit, reviewed on its own.
- A module stays in `core` only while two or more crates use it. The checker counts consumers per module from the source and fails when a module has one.
- `core` depends on no first-party crate and on no I/O, clock, async runtime or storage library.
- Its line count is recorded and may only go down without a manifest edit.

## 2. Audit (measured 2026-11-26 on branch `refactor/storage-port-repositories`)

Method: `cargo metadata --no-deps` for packages and dependencies; line counts of `*.rs` under `src/` and `tests/`; item-level scans of `use` paths for who uses what. Scans miss glob imports, so every file-level assignment below is confirmed by the compiler when the file moves.

### 2.1 Crates today → target

| Today | src / tests lines | Target | Note |
|---|---|---|---|
| `crates/core` `rrflow-core` (was `rrflow-kernel`, renamed in the working tree, unverified) | 12,321 / 1,310 | `rrflow/core` | reorganized, section 2.2 |
| `crates/storage/rrflow-lsm` | 24,387 / 13,956 | `rrflow/lsm` | |
| `crates/storage/rrflow-cld` | 1,940 / 567 | `rrflow/cld` | takes `rrflow-arrow` in as its `arrow` module |
| `crates/compute/rrflow-arrow` | 624 / 0 | into `rrflow/cld` | used by cld and query only |
| `crates/storage/rrd-store` | 28,738 / 12,968 | split: `rrflow/mx`, `rrflow/tx`, `rrflow/kv`, `rrflow/db` | section 2.3 |
| `crates/compute/rrd-query` | 9,697 / 5,606 | `rrflow/ql` | |
| `crates/compute/rrd-vector` | 14,485 / 2,524 | `rrflow/vector`; in-memory index parts to `rrflow/mx` | |
| `crates/compute/rrd-inference` | 2,944 / 2,397 | `rrflow/inference` | |
| `crates/compute/rrd-attunement` | 2,106 / 1,741 | `rrflow/attunement` | |
| `crates/authority/rrd-engram` | 6,533 / 1,938 | `rrflow/engram` | |
| `crates/authority/rrd-security` | 1,461 / 690 | `rrflow/security` | |
| `crates/operations/rrd-observation` | 317 / 265 | `rrflow/observability` | |
| `crates/operations/rrd-operator-knowledge` | 2,221 / 714 | `rrflow/knowledge` | |
| `crates/operations/rrd-cluster` | 10,140 / 4,881 | `rrflow/cx` | today uses `rrflow-lsm` directly and seven raw commit functions of the store |
| `crates/operations/rrd-kubernetes` | 738 / 189 | `rrflow/cx-operator` | no consumer |
| `crates/operations/rrd-maintenance` | 1,556 / 0 | deleted | no consumer, no tests |
| `crates/authority/rrd-estate/` | empty folder | deleted | |
| `crates/authority/rrd-engine` | 104,079 / 7,210 (46,078 of the src lines are tests) | `rrd/engine` | section 2.4 |
| `crates/transport/rrd-server` | 6,638 / 5,322 | `rrd/server` | |
| `crates/transport/rrd-contract` | 27,016 / 10,592 | `rrd/contract` | |
| `crates/transport/rrd-client` | 3,448 / 7,236 | `rrd/client` | |
| `crates/adapters/rrflow-cli` | 2,800 / 21,390 | `rrflow/cli` | |
| `crates/adapters/rrflow-mcp` | 690 / 356 | `rrflow/mcp` | |
| `crates/adapters/rrflow-edge` | 88 / 145 | `rrflow/edge` | |
| `crates/evaluation/**` (9 crates) | 21,717 total | `lab/*` | `rrflow-evaluation-core` → `rrflow-lab-core`; `rrflow-lab` → `rrflow-lab-runner`; `rrflow-qualification-storage` → `rrflow-lab-qualification-storage` |

### 2.2 `core` today, by who uses each module

| Module | Lines | Used by (references) | Decision |
|---|---|---|---|
| `runtime/` | 3,352 | store 268, vector 113, query 102, engine 34, cluster 34, 9 more | stays; split into `record/` (model, ids, graph, encoding, validate, referential) and `commit/` (stamps, transaction, audit, data_snapshot, projection). `commit/` moves to `rrflow-tx` if nothing below tx needs it; the compiler decides |
| `schema/` | 1,795 | store 133, query 57, cluster 26, engine 19, 7 more | stays |
| `data.rs` | 537 | store 76, vector 73, 4 more | stays, as `value/` |
| `claim.rs` | 307 | 19 crates | stays |
| `digest.rs` | 249 | 12 crates | stays |
| `ident.rs` | 152 | contract, cluster | stays, as `identifier.rs` |
| `error.rs` | 71 | vector, store, inference, 3 more | stays |
| `result_identity.rs` | 139 | query 16, contract 4, store 2, server 2 | stays |
| `trace/` | 2,569 | cluster 12, store 7, engine 5 | stays |
| `authenticated_log/` | 1,180 | store only | leaves, to `rrflow-tx` |
| `temporal.rs`, `reference.rs` | 410 | store (query once) | leave, to `rrflow-kv` |
| `telemetry.rs` | 797 | contract only | leaves, to `rrd-contract` |
| `reasoning_tree.rs` | 654 | contract only | leaves, to `rrd-contract`; the contract's own 679-line copy and this one become one definition |

`core` drops to about 9,300 lines and every remaining module has at least two consumers.

### 2.3 `rrd-store` today → mx, tx, kv, db

The order below follows the owner's table: mx is a leaf, tx sits on lsm and mx, kv on tx.

| Target | Takes | Open point the compiler settles |
|---|---|---|
| `rrflow-mx` | the in-memory MVCC map inside `engine.rs` (today `RrflowMxStore`, without the port impl), in-memory vector and BM25 index structures from `rrd-vector` and `rrd-query/bm25.rs` | which vector files are purely in-memory |
| `rrflow-tx` | `transaction.rs` (write set, commit and rollback outcomes), `access/semantic_commit/` (plan, conflicts, constraints, effects, retirements), the read-stamp and change-page half of `access/runtime_state.rs` (line 202 on), `access/versioned_identity_ranges.rs`, `core`'s authenticated log; from the engine: `transaction/`, `transaction_admission.rs`, `session_transactions.rs`, `query_transaction.rs`, `rollback.rs`, `clock.rs` where they need no engine state | `semantic_commit` reaches keys and keyspaces; if it cannot sit below kv, the key codec and keyspace table move down to tx with it |
| `rrflow-kv` | the port (`StorageEngine`, `StorageTransaction`, `StorageProfile`, `PhysicalStoreEvidence`), `rrflow_kv/` (the LSM-backed store), an adapter over `rrflow-mx`, cold routing over `rrflow-cld` (today the engine's `cold_history/`), `key_codec`, `keyspaces`, the read half of `runtime_state.rs` (lines 1–200), `snapshot`, `gc`, `footprint`, `control`, `fjall_baseline`, and the six repositories (`repository/`) with the `Repositories` trait | QL (runtime 12 calls, projections 6, control 3), vector (control 7), CX (runtime 2), engram and security (control) all call repositories and all sit below db, so the repositories cannot be in db |
| `rrflow-db` | `access/{index, lexical_index, vector, referential, function, versioned_read, proof_nodes}`, `lexical`, `projection`, `ds`, `writer`, `invocation`, `outcome`, `object`, `s3`, `archive`, `backup`; coordination of ql, vector and cx | QL uses `versioned_read` types and `lexical_index`, which are listed here but QL is below db: those types move down to kv or to ql when the error appears |

In the working tree now, uncommitted: the port no longer declares the six repository accessors; a `Repositories` trait in `repository/` is implemented once for every port. About 250 files gained one import. Compiles in the default build; tests not run.

### 2.4 `rrd-engine` today

54 modules side by side in `src/engine/`, a 303-line `mod.rs`, and 46,078 lines of tests inside `src`. Largest: `attunement` 5,782, `installation` 5,351, `vector` 3,468, `reasoning_budget` 1,791, `flight` 1,613, `engram_control` 1,538, `token_key` 1,527, `function` 1,410, `diagnostic` 1,348, `retrieval_query` 1,308.

Target: one folder per component the engine composes, named for it (`kv/`, `cld/`, `tx/`, `db/`, `ql/`, `vector/`, `cx/`, `inference/`, `security/`, `attunement/`, `engram/`, `observability/`), plus `lifecycle/` (instance, registry, serving, control), `installation/` and `memory/` (knowledge, memory_engram, context, task, task_assessment, reasoning_budget, project_map). Logic that needs no engine state moves down into its component. Tests that use only the public API move to `tests/`.

## 3. Phases

One branch, `restructure/workspace`, cut from the current working branch so the work in the tree is kept. One commit per phase at minimum. Each commit compiles (`cargo check --workspace --all-targets`). The branch merges to `main` through one pull request and the merge queue when section 4 holds; if it runs past three days, `main` is merged into it.

| # | Phase | Done when |
|---|---|---|
| 0 | Verify and commit what is in the tree: the `Repositories` port change, and `rrflow-kernel` → `rrflow-core` | workspace compiles with all features; lint clean; tests of store, engine and core pass; two commits |
| 1 | Layout and names, one pass: every row of 2.1 that is a rename or a move. `git mv` each crate to `<group>/<folder>`; replace package and library names in code, manifests, workflows and scripts; delete `rrd-maintenance` and `rrd-estate/`; fold `rrflow-arrow` into cld. `rrd-store` becomes `rrflow/db` here and is carved down in phases 3–5 | no `crates/` folder; no `rrd-` package except the four daemon crates; workspace compiles and lints |
| 2 | Architecture manifest and checker rewritten for section 1: groups, the allowed-dependency table, the Rust structure rules, the `core` rules. The tier model and every `target_*` record are deleted | checker passes on the tree and fails on a seeded violation of each rule (tests) |
| 3 | `core` reorganized per 2.2: self-describing modules; single-consumer modules move to their owner; one reasoning tree | every `core` module has two or more consumers; the contract no longer has a second tree |
| 4 | `rrflow-mx` created | it has no component dependency; its own tests pass alone |
| 5 | `rrflow-tx` created from db and the engine | depends only on core, lsm, mx |
| 6 | `rrflow-kv` created from db and the engine's cold routing | depends only on core, tx, lsm, cld, mx; ql, vector, cx, engram, security use kv for repositories |
| 7 | `rrflow-ql`, `rrflow-cx`, `rrflow-vector` cut to their allowed dependencies; cx stops using lsm and the raw commit functions directly | checker's dependency rule passes with no exception recorded |
| 8 | `rrflow-db` is what remains plus coordination | db depends on all; nothing below db names it |
| 9 | `rrd-engine` folders per 2.4; tests out of `src`; `mod.rs` removed | no flat module list; `lib.rs` is declarations only |
| 10 | CI, scripts and records follow: workflow package names, the 391 hand-written paths in `build_execution_inventory.py` replaced by a lookup through `cargo metadata`, `selected_tests.py`, the code-structure record rewritten to point here, README crate map | every `scripts/ci` check and script test passes |
| 11 | Full proof and merge | section 4 |
| 12 | macOS: make the runner run on every nightly, then review the macOS code paths the owner does not trust | separate plan, written when this merges |
| 13 | Connectome mirrors the layout, after the ACID and SurrealDB-capability proofs | separate plan |

## 4. Proof before merge

- `cargo check --workspace --all-targets --all-features` and `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean.
- `cargo test --workspace --no-fail-fast` passes on Windows. Every failure is fixed at its cause; no test is deleted, ignored or weakened to get there.
- The list of test names before phase 0 and after phase 10, with paths mapped, is the same set.
- The OpenAPI digest and every generated projection regenerate with no difference.
- All `scripts/ci` checks pass.
- The candidate gate passes on the pull request, then a manual nightly on the merge commit with the macOS jobs actually run.

## 5. Decisions taken in this plan (strike any that is wrong)

1. Top-level folders are `rrflow/`, `rrd/` and `lab/`; `crates/` goes.
2. `core` stays, named `rrflow-core`, with the rules of 1.3.
3. The four daemon crates keep `rrd-`: engine, server, contract, client (owner, 2026-11-25).
4. The repositories live in `rrflow-kv`, because five crates below db call them.
5. `rrflow-vector` stays its own crate between kv and db; its in-memory indexes go to mx.
6. Inference, attunement, engram, security, observability and knowledge stay separate crates under `rrflow/`; they are not merged into the engine.
7. Evidence, history and research records keep their text. Only links that break are repointed.

## 6. Not known yet

- Whether `semantic_commit` can sit below kv without taking the key codec with it (phase 5).
- Which `rrd-vector` files are purely in-memory (phase 4).
- Which engine transaction functions need engine state (phase 5).
- How long `cargo test --workspace` takes on this host end to end; the LSM suite alone is about nine minutes.
- "These all trigger in ms" has no number and nothing measures it. It is not part of this plan.

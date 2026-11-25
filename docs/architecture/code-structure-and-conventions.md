# Code structure and conventions

**Status:** active standard; the owner ordered the alignment on 2026-11-24 and its complete plan on 2026-11-25; section 6 is that plan and governs every move
**Coordinate:** `rrflow://rrflow-instance/data/architecture/code-structure-and-conventions`
**Owner:** the canonical folder layout, naming, file-size, module, error, schema and dependency rules for the engine workspace and Connectome, and how each rule is enforced
**Measured at:** engine `main` `297bd98f`, Connectome `19ee075c`, 2026-11-13

The owner's words of 2026-11-13: "files that are far too long need to be clearly refactored", "establish canonical patterns", folders and files that "reflect platform industry terminology", Rust "orthogonal and based off existing patterns", the UI on "arktype" with "schemas based off the engine", and code aligned "to ensure it never drifts or comes back again". Every rule below is enforced by a check that fails CI. A rule without a check is not a rule.

## 1. What exists today

| Area | Measured |
|---|---|
| Engine size | 30 crates, 787 `.rs` files, about 204k lines |
| Long engine files | 61 non-test files over 1,000 lines and 147 over 500. The largest are `rrd-contract/src/lib.rs` (7,669 lines, 261 public types, an 850-line `endpoint_catalogue()`), `rrd-lsm/src/database.rs` (3,056), `rrflow-kernel/src/runtime.rs` (2,825) and `rrd-lsm/src/segment/mod.rs` (2,802) |
| Engine lint and layout rules | No `clippy.toml`, `rustfmt.toml`, `deny.toml` or `[workspace.lints]`; `too_many_arguments` allowed 132 times. `workspace_architecture.rs` enforces the package-to-path map, dependency sets for 7 crates, forbidden edges and vocabulary, but nothing on file size, module style, errors or tier order |
| Engine module, error and naming patterns | Both `mod.rs` and file-named modules, plus 23 `#[path]` attributes. Errors are hand-written in five styles, with about 125 `Result<_, String>` and 109 `Box<dyn Error>` signatures. About 35 SHA-256 validators and 13 hand-written hex encoders sit beside `rrflow-kernel::digest`. "engram" appears 3,100 times and "engram" 115, with miscased renames such as `InstalledengramRecord` |
| Engine layering | Group directories do not match dependency tiers: `transport/` holds both the lowest crate (`rrd-contract`) and the highest (`rrd-server`). `rrd-maintenance` and `rrd-kubernetes` have no consumers |
| Connectome size | 406 TS/TSX files (67,686 lines) and 41 Rust files (18,064 lines) |
| Connectome layout | Five organizing schemes at once: layer folders, `screens/{pages,views}`, a half-built `shell/shared/spokes`, the `rrflow/` client, and workspace areas. Modals live in 3 places, stores in 2, components in 3 and connection logic in 4. 45 imports break the written spoke rules |
| Long Connectome files | 26 product files over 500 lines (`workbenchLanes.tsx` 1,343, `quantization.tsx` 1,253, `session.ts` 1,196); Rust `view.rs` 1,733 |
| Connectome schemas | arktype used once and valibot twice, both on inherited code. 84 IPC view types are hand-written copies of Rust structs. Configuration loads with no schema check. 15 `JSON.parse` calls are unvalidated |
| Connectome enforcement | Ten `pnpm check` scripts; the only import gate is RRFlow isolation. No file-size or naming rule |

## 2. Engine rules

| Id | Rule | Industry basis | Enforced by |
|---|---|---|---|
| R1 | A non-test source file has at most 800 lines; a test file at most 1,500 | file-size budgets in large Rust codebases; a reviewable unit | `scripts/ci/check_code_shape.py` with a recorded **ratchet**: every file over the limit today is listed with its current length. A listed file may only shrink, and it leaves the list when under the limit. A new file over the limit fails |
| R2 | `lib.rs` and `main.rs` hold only module declarations, re-exports and the entry point, at most 200 lines | the facade pattern of tokio, serde and the Rust API guidelines | the same check |
| R3 | File-named modules (`foo.rs` with `foo/`). No new `mod.rs` and no `#[path]` | Rust 2018 module idiom | the same check, with the 10 `mod.rs` files and 23 `#[path]` attributes on the ratchet list |
| R4 | Crates form declared tiers, and a crate depends only on lower tiers | layered architecture, as in rustc's own crate graph | one tier table in `workspace_architecture.rs`; every edge checked, not just 7 crates |
| R5 | One `Error` enum per library crate in `error.rs` (`thiserror`), with `pub type Result<T>`. No `Result<_, String>` or `Box<dyn Error>` in library crates | Rust API guidelines C-GOOD-ERR | a source check on the ratchet: counts may only fall |
| R6 | One digest type: `Sha256Digest` (validated lowercase hex), with one encoder and one validator, in a tier-0 crate both `rrflow-kernel` and `rrd-contract` use | one owner per concept | the check refuses `Sha256::new` and hex encoders outside that crate |
| R7 | Configuration naming: `Options` for constructor inputs, `Settings` only for S1 settings, `Policy` for rules, `Limits` for caps. One `open(path, Options)` instead of `open_with_*` and `create_with_*` fan-out. More than three optional inputs means an options struct | builder and options patterns of the API guidelines | `too_many_arguments` stops being allowed: its 132 escapes go on the ratchet |
| R8 | `[workspace.lints]`: `unsafe_code` denied outside named crates, `clippy::undocumented_unsafe_blocks`, `unreachable_pub`, `missing_debug_implementations`; `cargo-deny` for licences and advisories | standard workspace hygiene | CI clippy and `cargo deny check` |
| R9 | Tests: unit tests in a sibling `tests.rs` (`#[cfg(test)] mod tests;`); integration and public-API tests in `tests/`. `rrd-engine/src/engine/tests/` (42.8k lines inside the crate) moves to `tests/` wherever it uses the public API | Cargo conventions | the shape check |
| R10 | Names: every workspace crate is `rrflow-<name>` and a component's crate carries the component's name (section 6.1); the `rrd-*` prefix is retired. Types carry no crate prefix except the facade (`RrdEngine`, `RrdClient`). No rename leaves an alias or shim | consistent vocabulary | the architecture checker of section 6.11 |
| R11 | No unconsumed crates or lab code in production crates: `rrd-maintenance` is removed and `physical_policy_lab.rs` moves to the Lab. `rrd-cluster`, `rrd-kubernetes` and the pgvector adapter are product scope (owner, 2026-11-13). They serve the shippin panel and cloud synchronization, so they move to the cloud tier and gain real consumers and tests (kernel package K15) | one current implementation | the architecture test lists every crate with a consumer or an owner decision |

**Target tiers:** the crate, folder and tier table is in [section 6.2](#62-target-workspace).

**First splits**, one PR each, with behavior unchanged:

1. **`rrd-contract/src/lib.rs` → about 15 modules:**
   - `protocol` (constants, limits)
   - `query`
   - `vector`
   - `retrieval`
   - `points`
   - `changefeed`
   - `backup`
   - `audit`
   - `catalogue` (with `endpoint_catalogue()` split per domain)
   - `engram_authority`
   - `error`
   - `identity`
   - `deployment`
   - `session`
   - `schema_values`
   - `transaction`
   - `commit`

   Proof: the OpenAPI digest constant is unchanged, every SDK projection regenerates with no diff, and all `rrd-contract` tests pass untouched.
2. **`rrd-lsm/src/database.rs`** → `database/{options, failpoints, open, transactions, reads, compaction, gc, snapshot}.rs`.
3. **`rrflow-kernel/src/runtime.rs`**, **`rrd-lsm/src/segment/mod.rs`**, **`rrflow-kernel/src/trace.rs`**, **`rrd-engine/src/engine/installation.rs`**, **`rrd-store/src/backup.rs`**, **`rrd-store/src/rrflow_kv.rs`** and **`rrd-vector/src/hnsw.rs`**, by the split points recorded in the 2026-11-13 survey.
4. The remaining files on the ratchet list, largest first.

## 3. Connectome rules

**Target layout.** This builds on the 2026-09 proposal (parking lot 004, option 2B), which was never committed, and replaces the five schemes:

```
src/
  app/                 entry, providers, router setup, shell layout
  routes/              TanStack file-based routes: thin, compose features only
  features/<feature>/  components/  hooks/  schemas/  services/  state/  index.ts
  devkit/              dev component library: inspectors, metric and leak panels, trace viewer,
                       component registry and gallery (MCPL concept applied to dev UI)
  shared/ui/           primitives, theme, tokens (teal and burnt orange)
  shared/lib/          pure utilities only
  contracts/generated/ arktype schemas and types generated from the engine OpenAPI
  contracts/ipc/       types generated from the Tauri Rust structs (ts-rs or specta)
  platform/            tauri and browser adapters
src-tauri/src/
  main.rs (thin)  lib.rs  commands/  application/  adapters/{rrd,config,filesystem,process}  ipc/
```

**Features:**
- `workspace`: projects, instances, pods and lanes;
- `connections`;
- `knowledge`: graph, schema, records, vectors;
- `reasoning`: tasks, plans, traces, the reasoning graph;
- `user-configuration`: User policies and preferences, with effective-value explanations from the engine ([ADR-0006](../decisions/0006-scoped-policies-and-preferences.md));
- `lab`: systems, datasets, benchmarks, experiments, reports;
- `settings`;
- `guides`;
- `diagnostics`;
- `query`: the inherited editor, re-authored.

| Id | Rule | Enforced by |
|---|---|---|
| C1 | Dependencies run one way: `routes` → `features` → `shared`, `contracts`, `platform`. A feature imports another feature only through its `index.ts`. `shared` never imports a feature | an import-boundary check (dependency-cruiser) in `pnpm check` |
| C2 | arktype is the only runtime schema library. Engine schemas are generated from the OpenAPI document, IPC schemas from Rust. Every `JSON.parse`, configuration load and import is validated. valibot is removed with the inherited Docker code | a check that refuses other schema libraries and unvalidated `JSON.parse` |
| C3 | The configuration store has one arktype schema with versioned migrations. A load that fails validation is reported, never silently patched | a test that loads every historical fixture |
| C4 | TS and TSX files at most 400 lines; Tauri Rust follows R1 | the shape check with a ratchet, as in R1 |
| C5 | Names: feature folders in kebab-case; components `PascalCase.tsx`; hooks `useThing.ts`; schemas `thing.schema.ts`; one component per file | biome `useFilenamingConvention` and the shape check |
| C6 | No screen is deleted before its replacement reproduces it (`tests/visual/PARITY.md`) | the existing visual capture and compare |
| C7 | Every setting is rendered from its schema: no hand-written settings form | the settings feature reads only generated S1 schemas |
| C8 | The dev UI shows the truth: each instance and lane shows RSS, memory events, handles against the limit, threads, cache bytes, queue depth, queue wait p95, rejections and pressure stall, and flags any value still rising after warmup | devkit panels fed by engine diagnostics; a leak soak test |

**Configuration categories** (Connectome settings, mapped to S1):

1. Workspace and projects: serving, instances, pods, lanes, credential mapping.
2. Connections: profiles, templates, authentication.
3. Engine: generated from S1 (storage, planner, vector, lexical, graph, reasoning effort, LFG, System One, endpoints).
4. Lab: systems, runs, templates, linked paths.
5. Appearance: theme, scale, sidebar, motion.
6. Editor and query.
7. Visualization: graph and diagram defaults.
8. Input and commands: keybindings.
9. Diagnostics and updates.
10. Experimental: feature flags.

Session recovery state is not configuration and moves out of the settings tree.

**First moves**, one PR each:

1. Enforcement with ratchets in both repositories, so nothing gets worse from the first merge.
2. `contracts/generated` with arktype schemas from the engine; the configuration schema and migrations; the engine client repinned to `main`.
3. TanStack file routes beside wouter, then one feature per PR in workspace-area order. Each moved file is split under the limit, with visual parity captured before and compared after:
   - `workbenchLanes.tsx` → `features/lab/`;
   - `quantization.tsx` → `features/knowledge/vectors/`;
   - `session.ts` → `contracts/ipc` plus `platform/tauri`.
4. Inherited views re-authored or retired under C6.

## 4. Pods and lanes

This implements S6 of the [kernel contract](../roadmap/kernel-contract.md#36-s6-project-engram-instance-and-pod), using the 2026-11-13 research and the [SurrealDB multi-project probe](../evidence/surreal-projects-20261113/README.md).

**Lab pod** (`eval/lab/pod.sh`, new on 2026-11-11):
- RRFlow runs as a sibling container under the same limits as every other system.
- Servers get cpuset and memory limits, but no CPU quota, which is measured to inflate tail latency.
- Build is separated from measurement by a quiet window, judged by pressure stall.
- Warmup is not scored; at least 5 interleaved repetitions run, reported as a median with a confidence interval.
- A run whose spread exceeds 5% is marked "inconclusive".
- Pressure stall and host load are recorded every second, including Windows-side CPU.
- `.wslconfig` is pinned and recorded, `autoMemoryReclaim` is off, and the VM clock is checked against the host.
- Every resource is labelled for reaper-style cleanup of orphaned containers.
- A socket proxy limits the runner to the lab's own containers.

**Instances and lanes:**
- One process or container per project, never a shared namespace (that is the probe's result).
- Per-instance CPU weight, memory high and max, I/O weight and a pids cap.
- Background compaction and indexing run at idle priority.
- Caches are per instance.
- Lanes, with defaults:
  - `count` 4;
  - `max_in_flight` 2 per lane;
  - `queue_depth` 16;
  - `queue_timeout_ms` 2,000;
  - deficit round-robin between lanes;
  - a refusal when a lane is full.
- Enforcement: cgroups v2 on Linux, Job Objects on Windows.

**Accepted when:**
- Another project's write storm raises a project's p95 by at most 1.25× across 10 interleaved repetitions.
- Jain's fairness index across equal lanes is at least 0.9.
- Canary records never cross projects.
- A soak of 30 minutes or more shows memory, handles and threads levelling off after warmup and returning to baseline after a lane closes.

## 5. Owner decisions

1. **Structure:** ordered by the owner on 2026-11-24 ("go") and 2026-11-25. Section 6 is the plan.
2. **`rrd-cluster`, `rrd-kubernetes` and `pgvector`:** decided 2026-11-13. They are product scope for the shippin panel and for synchronizing with the cloud offering, comparable to SurrealDB Cloud or ArangoDB's managed service and backed by shippinAI. They move to `crates/cloud/` under K15.
3. **Limits:** 800 lines for Rust and 400 for TS/TSX are the proposed caps. The default is these values.
4. **One repository:** decided 2026-11-25. The storage engines and foundation crates stay in this repository for now, each defined as a clearly bounded workspace crate. Section 6.12 is the rule that keeps each one able to leave later.

## 6. Alignment plan of 2026-11-25

The owner ruled on 2026-11-24 that no new code is written until the existing code is aligned without regression, and on 2026-11-25 that the plan is complete before any further move. This section is that plan. It replaces the tier table that stood in section 2 before this date. Every number below was measured at `7f16ed4be`; anything not measured is listed in 6.9.

### 6.1 Naming

One rule, no exceptions: **every workspace crate is `rrflow-<name>`, and a product component's crate carries the component's name.** The `rrd-*` prefix is retired. A rename replaces the old name everywhere in the same pull request; no alias, re-export shim or forwarding module is left behind.

| Component | Crate | Rust path |
|---|---|---|
| rrflowKV | `rrflow-kv` | `rrflow_kv` |
| rrflowLSM | `rrflow-lsm` | `rrflow_lsm` |
| rrflowCLD | `rrflow-cld` | `rrflow_cld` |
| rrflowTX | `rrflow-tx` | `rrflow_tx` |
| rrflowDB | `rrflow-db` | `rrflow_db` |
| rrflowQL | `rrflow-ql` | `rrflow_ql` |
| rrflowMX | `rrflow-mx` | `rrflow_mx` |
| rrflowCX | `rrflow-cx` | `rrflow_cx` |

Folders are named for the tier, in plain platform terms. A crate's folder name equals its package name. The engine crate is `rrflow-core`: the name formed from the product name and the word engine is a retired split-engine identity that the `workspace_architecture` test forbids in every tracked file, and that ban stays.

### 6.2 Target workspace

A crate depends only on crates in a lower tier, or in its own tier where the table gives the order.

| Tier | Folder | Crate today | Target crate | Lines today |
|---|---|---|---|---|
| 0 | `crates/foundation/` | `rrflow-kernel` | `rrflow-kernel` | 13,631 |
| 0 | | `transport/rrd-contract` | `rrflow-contract` | 37,718 |
| 0 | | `compute/rrflow-arrow` | `rrflow-arrow` | 624 |
| 1 | `crates/storage/` | `persistence/rrd-lsm` | `rrflow-lsm` | 39,473 |
| 1 | | `persistence/rrflow-cld` | `rrflow-cld` | 2,752 |
| 1 | | part of `persistence/rrd-store` | `rrflow-kv` (on `rrflow-lsm`) | see 6.3 |
| 1 | | part of `rrd-store` and of `rrd-engine` | `rrflow-tx` (on `rrflow-kv`) | see 6.3 |
| 1 | | the rest of `rrd-store` | `rrflow-db` (on `rrflow-tx`) | see 6.3 |
| 2 | `crates/compute/` | `rrd-query` | `rrflow-ql` | 15,291 |
| 2 | | `rrd-vector` | `rrflow-vector` | 17,751 |
| 2 | | `rrd-inference` | `rrflow-inference` | 5,433 |
| 2 | | `rrd-attunement` | `rrflow-attunement` | 3,847 |
| 3 | `crates/knowledge/` | scattered, see 6.3 | `rrflow-mx` | see 6.3 |
| 3 | | `authority/rrd-engram` | `rrflow-engram` | 8,471 |
| 3 | | `operations/rrd-operator-knowledge` | `rrflow-operator-knowledge` | 2,935 |
| 3 | `crates/governance/` | `authority/rrd-security` | `rrflow-security` | 2,150 |
| 3 | | `operations/rrd-observation` | `rrflow-observation` | 582 |
| 3 | `crates/cloud/` | `operations/rrd-cluster` | `rrflow-cx` | 15,018 |
| 4 | `crates/engine/` | `authority/rrd-engine` | `rrflow-core` | 110,976 |
| 5 | `crates/interfaces/` | `transport/rrd-server` | `rrflow-server` | 11,957 |
| 5 | | `transport/rrd-client` | `rrflow-client` | 12,034 |
| 5 | | `adapters/rrflow-cli` (binary `rrflow`) | `rrflow-cli` | 24,270 |
| 5 | | `adapters/rrflow-mcp` | `rrflow-mcp` | 1,046 |
| 5 | | `adapters/rrflow-edge` | `rrflow-edge` | 321 |
| 5 | `crates/cloud/` | `operations/rrd-kubernetes` | `rrflow-cx-operator` | 927 |
| — | `crates/evaluation/` | Lab and qualification crates | unchanged | |

`rrd-cluster` sits below the engine today (it depends on `rrd-lsm`, `rrd-store` and the kernel, and the engine composes it behind the `cluster-transfer` feature), so `rrflow-cx` is tier 3 and stays optional. The Kubernetes operator has no consumer and depends only on the contract and kernel, so it is an interface-tier crate.

Removed, each in the pull request named in 6.8: `rrd-maintenance` (no consumer; one file of 1,556 lines) and the empty untracked folders `crates/authority/rrd-estate/` and `engine/estate_control/`. `rrflow-lab-experiments` stays: no crate depends on it, but it is a binary with its own CI test (`provider_compatibility` in `ci-reusable.yml`).

### 6.3 Component boundaries, measured

**`rrd-store` splits three ways.** Its source is 28,824 lines in 25 top-level modules. A scan of `crate::` and `super::` references found no pair of modules that depend on each other, and found this order:

- `key_codec` and `keyspaces` depend on nothing else in the crate.
- `access/runtime_state.rs` (778 lines) is the root of the `access` layer: every other `access` module depends on it and it depends on none of them. It holds key checking, point reads, scans, sequences, read stamps and change pages.
- `rrflow_kv/` (2,661 lines) depends on `access` only through `runtime_state` (11 references), and on `key_codec`, `keyspaces`, `transaction`, `control`, `engine`, `error` and `lexical`. Nothing in `access` or `repository` depends on `rrflow_kv`. It makes 24 direct `rrd_lsm::` uses; `access` makes 3.
- `access/semantic_commit/` (1,998 lines: plan, conflicts, constraints, effects, retirements) depends only on `runtime_state`; `access/index` and `access/vector` depend on it.
- `repository` depends on `access` (16 references); `backup` depends on `access`, `archive`, `repository` and `snapshot`.

| Target | Takes from `rrd-store` | Takes from `rrd-engine` |
|---|---|---|
| `rrflow-kv` | `key_codec`, `keyspaces`, `error`, `transaction`, `engine` (the storage port, its profiles and the volatile store), `control`, `gc`, `snapshot`, `footprint`, `fjall_baseline`, `rrflow_kv/`, and the key, read, scan and sequence half of `access/runtime_state.rs` | nothing |
| `rrflow-tx` | the read-stamp and change-page half of `access/runtime_state.rs`, `access/semantic_commit/`, `access/versioned_identity_ranges.rs` | `engine/transaction.rs` and `transaction/` (890 lines), `transaction_admission.rs` (411), `session_transactions.rs` (120), `query_transaction.rs` (175), `rollback.rs` (219), `clock.rs` (271): the parts that do not need `RrdEngine` state |
| `rrflow-db` | `access/{index, lexical_index, vector, referential, function, versioned_read, proof_nodes}`, `repository`, `lexical`, `projection`, `ds`, `writer`, `invocation`, `outcome`, `object`, `s3`, `archive`, `backup` | nothing |

`RrflowMxStore` is renamed `VolatileStore` and its profile `volatile` when it moves into `rrflow-kv`; the seven records named in the system overview are corrected in that pull request.

**`rrflow-mx` is gathered.** Today the agent memory and knowledge behavior is in `rrd-engine` (`engine/knowledge.rs` 378 lines, `memory_engram.rs` 440, `context/` 1,136, `task.rs` 678, `task_assessment.rs` 616, `project_map.rs` 420) and in the contract (`knowledge_tree`, `knowledge`, `memory_context`, `memory_engram`). The crate takes the logic that does not need `RrdEngine` state; the engine keeps a `mx/` folder that calls it.

**The reasoning tree has one definition.** `rrd-contract/src/reasoning_tree.rs` (679 lines) and `rrflow-kernel/src/reasoning_tree.rs` (654) declare the same 22 public items; the contract adds `ReasoningReadStamp`. The contract depends on the kernel only in tests, and one test parses the same JSON into both. The two are not copies: the contract's types identify things with `CanonicalId` and report `ContractError`, and the kernel's use `RuntimeId` and `RuntimeType` and report the kernel `Error`. One definition therefore needs one identifier type first. Package 4 makes the kernel the owner of the identifier and of the tree, and the contract depends on the kernel and re-exports both with its schema derive. See 6.9 item 1.

### 6.4 The engine pulls each crate by name

`rrd-engine/src/engine/` holds 54 modules side by side, 45,916 lines of tests inside `src`, and a 303-line `mod.rs`. The target is one folder per thing the engine composes, each named for the crate it pulls:

```
crates/engine/rrflow-core/
  src/
    lib.rs            declarations and re-exports only
    engine.rs         RrdEngine: open, close, the composed parts
    error.rs
    kv/               storage_cursor, tail_repair, maintenance
    cld/              cold_history
    tx/               transaction, transaction_admission, session, session_transactions,
                      query_transaction, rollback, clock
    db/               data, changefeed, subscription, backup, restore_publication
    ql/               query, query_export, retrieval, retrieval_query, function
    vector/           vector
    mx/               knowledge, memory_engram, context, task, task_assessment,
                      reasoning_budget, project_map
    inference/        inference, model, endpoint_configuration, endpoint_proposal,
                      endpoint_worker, source_language
    cx/               distributed
    security/         security, token_key, invocation
    attunement/       attunement
    installation/     installation, engram, engram_control, engram_layout, scoped_configuration
    lifecycle/        instance, instance_registry, serving, control
    dev/              diagnostic, flight, observation_delivery, observation_source
    runtime/          the existing runtime module
  tests/              every test that uses only the public API, in the same folder names
```

All 54 modules are placed above. A folder imports its own crate and the foundation; it reaches another component only through `engine.rs`. Logic that does not need engine state moves down into the component crate (6.3), so the engine folders become wiring.

### 6.5 The golden layout inside every crate

```
<crate>/
  Cargo.toml
  src/
    lib.rs              module declarations and re-exports, at most 200 lines
    error.rs            one Error enum, pub type Result<T>
    <feature>.rs        the feature's public surface
    <feature>/          its parts, colocated: types.rs, <verb>.rs, tests.rs
    dev/                trace, debug and inspection code, behind the `dev` feature
  tests/                public-API tests
```

A feature folder holds everything that feature needs and nothing another feature owns. Shared helpers live in the lowest crate that every user can reach, never in a `utils` or `common` module. Rules R1 to R9 above apply unchanged.

### 6.6 Processes and lifecycle

Measured today: `rrd-server` has `process.rs` and `stop.rs`; the engine exports `ServingStopRequest`, `ServingStopState`, `EngineServingStop` and an instance registry. There is no pause, resume or drain operation anywhere in `serving.rs`, `instance.rs`, `instance_registry.rs`, `control.rs` or the CLI. POAM-088 records that managed instances are not registered or stoppable in every form.

Alignment moves the existing start, serve, register and stop code into `lifecycle/` and changes none of it. Pause for patching, resume and drain are new behavior. They are the first package after alignment, built in `lifecycle/` behind one state type (`starting`, `serving`, `paused`, `draining`, `stopped`), so that a service is patched or stopped through one path.

### 6.7 Development, trace and debug code

Measured today: trace and inspection code is in `rrflow-kernel/src/trace/` (2,569 lines) and `telemetry.rs` (797), `rrd-engine` `diagnostic/` (1,345), `flight/` (1,595) and `runtime/trace.rs`, and `rrd-observation` (582). No crate has a `dev` or internal-build feature; the features are `full`, `edge`, `crash-test`, `cld-parquet`, `cld-vortex` and `cluster-transfer`.

Target: each crate's development-only code is in its `dev/` folder behind a `dev` feature, and one workspace feature on `rrflow-cli` turns them all on for the internal build. Code that a released engine needs (the audit chain, the public diagnostic operations) is not development code and stays in its feature folder. See 6.9 item 2.

### 6.8 Order of work

One pull request per package. A package is complete when nothing of the old shape remains and the proof in 6.10 holds; a package that cannot be completed is not opened.

| # | Package | Removes |
|---|---|---|
| 1 | Repair `main` (pull request 511, POAM-094) | the red nightly |
| 2 | One architecture manifest and one checker (6.11) | the path tables repeated in two test files, four CI scripts, two script tests and a baseline, all of which the kernel move had to edit by hand |
| 3 | Dead code: `rrd-maintenance`, the empty folders, and the 29 files that still say "estate" | unconsumed crates and the retired name |
| 4 | Foundation: `rrd-contract` → `foundation/rrflow-contract`, `rrflow-arrow` → `foundation/`, one reasoning-tree definition | the duplicate type set |
| 5 | `rrd-lsm` → `storage/rrflow-lsm`; `rrflow-cld` → `storage/` | `persistence/` |
| 6 | `rrflow-kv` extracted, with the `VolatileStore` rename | the misnamed rrflowMX store |
| 7 | `rrflow-tx` extracted from the store and the engine | transaction logic in two crates |
| 8 | The rest of `rrd-store` → `storage/rrflow-db` | `rrd-store` |
| 9 | Compute renames: `rrflow-ql`, `rrflow-vector`, `rrflow-inference`, `rrflow-attunement` | `rrd-*` in tier 2 |
| 10 | `rrflow-mx` gathered; `rrflow-engram`, `rrflow-operator-knowledge`, `rrflow-security`, `rrflow-observation` moved and renamed | scattered memory code, `authority/`, `operations/` |
| 11 | `rrflow-cx` and `rrflow-cx-operator` | `rrd-cluster`, `rrd-kubernetes` |
| 12 | `rrflow-core`: the 6.4 folders, tests out of `src` | the flat 54-module engine |
| 13 | Interfaces: `rrflow-server`, `rrflow-client`, the CLI, MCP and edge adapters | `transport/`, `adapters/` |
| 14 | Connectome: `features/` named for the engine folders of 6.4, client repinned | the five organizing schemes of section 1 |

Packages 4 to 13 each exceed the 400-line warning because a rename touches every import. They are moves, reviewed as renames.

### 6.9 Not yet measured

Each item is settled by reading code at the start of the package that needs it, and the package's journal records the result. None is assumed.

1. The two reasoning-tree files differ in 269 lines, classified on 2026-11-25 as identifier types, error types, derives and comments. Not yet read: whether `CanonicalId` and `RuntimeId` accept exactly the same values. If they do not, package 4 records each difference and which rule the single type keeps.
2. Which trace and diagnostic code a released engine needs and which is development-only (packages 2 and 12).
3. Settled on 2026-11-25: `rrflow-lab-experiments` has a CI test and stays.
4. Which rows of `rrd-maintenance` the [context-maintenance record](../reference/context/context-maintenance.md) requires to be covered elsewhere before removal (package 3).
5. The file-level split of `access/runtime_state.rs` between `rrflow-kv` and `rrflow-tx`, and which engine transaction functions need `RrdEngine` state (packages 6 and 7).
6. Whether time-series and geospatial capability exist in code, as the system overview already notes.
7. The module graph in 6.3 comes from a text scan of `crate::` and `super::` paths. It misses glob imports. Package 2 replaces it with a graph from the compiler.
8. Whether `RrflowKvStore` and `StorageEngine` land in `rrflow-kv`. If they do, the `rrflow-cld` test of 6.12 conforms once it names `rrflow-kv`; if not, the test moves to `rrflow-db` (package 6).
9. Which of the kernel product types that `rrd-store` uses are needed by the `rrflow-kv` part, and so whether `rrflow-kernel` as a whole is a fit dependency for a standalone key-value store or its `digest` module must be separable (package 6).
10. Rule 3 of 6.12 has not been run for any crate (package 2).

### 6.10 Proof of no regression, every package

- `cargo check --workspace --all-targets --locked` is clean and `Cargo.lock` changes only in package names.
- The list of test names from `cargo test --workspace -- --list`, with crate and module prefixes mapped by the package's rename table, is identical before and after. No test is removed, ignored or renamed apart from its path.
- `cargo test --workspace --locked` passes on the author's host, and the candidate gate passes.
- The OpenAPI digest constant is unchanged and every generated projection regenerates with no difference.
- After the merge the nightly full matrix is started by hand on the new `main`, and the next package does not open until it is green.

### 6.11 Enforcement

One file, `crates/architecture.toml`, is the only place that lists each crate's tier, folder and allowed dependencies, the engine folders of 6.4, the golden layout of 6.5 and the size limits. One checker reads it and fails with a hard error on:

- a crate outside its folder, a crate name that is not `rrflow-<name>`, or a dependency on a higher tier;
- an engine folder importing another component's crate directly;
- a module named `utils`, `common`, `helpers` or `misc`; a `mod.rs`; a `#[path]`;
- a file over its limit, or `lib.rs` holding logic;
- development code outside `dev/`, or `dev/` code reachable without the `dev` feature;
- a standalone crate breaking a rule of 6.12, or a first-party dependency declared anywhere but the root manifest.

The existing `workspace_architecture` test and `check_code_shape.py` read the same file instead of their own tables. A violation that exists when a rule lands is recorded with its current measure and may only shrink. Adding a crate, an engine folder or a rule is one edit to the manifest, and the same checker runs in Connectome against the mirrored folder list.

### 6.12 One repository, standalone crates

The owner decided on 2026-11-25 that the storage engines and foundation crates stay in this repository for now. There is one Cargo workspace, one `Cargo.lock` and one CI. A crate that could later become its own repository is marked `standalone` in `crates/architecture.toml` and obeys four rules, each a hard error in the checker:

1. Its first-party dependencies are only other standalone crates.
2. Its dev-dependencies obey rule 1 as well, so its tests need no product crate.
3. `cargo test -p <crate> --locked` passes with no other workspace crate's features enabled.
4. Every first-party dependency in the workspace is declared once, in the root `[workspace.dependencies]`, and consumed with `workspace = true`. Moving a standalone crate out is then one edit: its root entry changes from `path` to `git` with a revision.

| Standalone crate | First-party dependencies today | Measured at `058f459a8` |
|---|---|---|
| `rrflow-kernel` | none | conforms to rules 1 and 2 |
| `rrflow-arrow` | none | conforms to rules 1 and 2 |
| `rrflow-lsm` | `rrflow-kernel` | conforms; all 7 kernel references are to `digest` (a 249-line module) |
| `rrflow-cld` | `rrflow-arrow`; dev-dependency `rrd-store` | breaks rule 2: `tests/stamped_history.rs` uses `RrflowKvStore` and `StorageEngine` |
| `rrflow-kv` | not a crate yet (package 6) | `rrd-store`, which holds it, uses kernel product types (`RuntimeLogicalModel` 18 references, `ScopeId` 14) |

Not standalone: `rrflow-tx`, `rrflow-db` and everything above them. `rrflow-vector` depends on `rrd-store` (8 references, 7 of them its `Error`), so the vector index cannot stand alone today.

Rule 4 is broken widely today: 84 first-party dependency lines name a `path` directly and 18 use `workspace = true`; the root manifest declares only `rrflow-kernel`. Package 2 moves all of them to the root, which changes no code and no `Cargo.lock` entry.

This decision agrees with the rule in `AGENTS.md` that all first-party engine code lives in this repository, so that rule does not change.

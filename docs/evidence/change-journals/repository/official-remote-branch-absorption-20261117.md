# Official-remote branches: read, absorbed or shown superseded: 2026-11-17

**Status:** active change journal; all six branches read and given a disposition; claims only what each row states
**Coordinate:** `rrflow://rrflow-instance/data/change-journal/official-remote-branch-absorption-20261117`
**Owner:** the record of the owner's 2026-11-17 direction to read every branch fully and prove each feature is carried or superseded; [AGENTS.md](../../../../AGENTS.md) owns the process

## Change brief

- **Owner direction (2026-11-17):** work the oldest branch first; read every file and every change; absorb each one or prove it was superseded; make no assumption and no blind comparison.
- **Scope:** the six non-`main` branches of the official repository `rrflow/rrflow` (remote `origin`). Nothing is deleted here. Every branch stays on `origin` until the owner decides.
- **Method:** for each branch, read its commits and every changed file in full, then read the code on `development/main` that implements each capability and name the file. Patch-id equality was not accepted as proof. Failures to find a counterpart are recorded as gaps.
- **Starting revision:** `development/main` at `d22de7d54`.

## Branch 1 of 6: `agent/connectome-temporal-runtime-visualizer` (tip `63bc2331f`, 2026-08-30)

**Disposition: absorbed.** The tip and its whole history are ancestors of `development/main`: this branch is the engine's August mainline, not a separate line of work.

Read: the tip commit (`feat: complete persistent estate authority`, 13 files, +1,524/-28) and the 12 most recent commits' subjects. The tip's feature, the persistent authority catalogue, lives today at `crates/authority/rrd-engram/src/authority.rs` (the `estate` to `engram` rename). All eight core types the commit defined (`AuthorityResourceKind`, `AuthorityStatus`, `AuthorityReceiptBoundary`, `AuthorityDesiredState`, `AuthorityObservedState`, `AuthorityResource`, `AuthorityReceipt`, `AuthorityIdempotencyBinding`) are defined there. Its test `authority_catalogue.rs` is now `crates/authority/rrd-engram/tests/authority_catalogue.rs`.

## Branch 2 of 6: `codex/consolidate-project-structure-and-context` (tip `73c75a852`, 2026-09-02)

One commit, `Define shape-first consolidation path`, four files: `README.md` (+5), `docs/consolidation-contract.md` (+91), `scripts/project_shape.py` (+110), `scripts/test_project_shape.py` (+39). Read in full: all four. **Disposition: carried in part, superseded in part.** The document did not exist on `main`; it is now preserved as [`docs/roadmap/consolidation-contract.md`](../../../roadmap/consolidation-contract.md), body unchanged.

### The generator and its test (`project_shape.py`, `test_project_shape.py`)

| Capability on the branch | Where it is on `development/main` | Evidence read |
|---|---|---|
| Entries sorted by path | `installation/inventory/scan.rs`: `entries.sort_by(... relative_path())` | scan.rs, 213 lines, read in full |
| SHA-256 content identity per file, one tree digest | `scan.rs` folds a per-file digest into one aggregate that also binds the policy, the git identity, each entry's kind, path length and size | scan.rs |
| Excludes `.git`, `.vyrm`, `node_modules`, `target` | `assets/install/default-profile-v3.json` exclusions: `.git`, `.rrflow` (the engram state, replacing `.vyrm`), `target/`, `node_modules/`, plus `vendor/`, `build/`, `dist/`, `coverage/`, `.cache/`, `__pycache__/`, `.venv/`, `.next/`, `.turbo/`, `*.tmp` | profile, lines 62 to 150 |
| Roles for `AGENTS.md`, `Cargo.toml`, `package.json`, `pyproject.toml`, `README.md` | `engine/project_map.rs` `start_here()`: all five, plus 17 more, with stack detection and language shares | project_map.rs lines 262 to 372 |
| Deterministic output | test `exclusions_and_creation_order_do_not_change_the_digest_or_admitted_count` creates two trees in different orders and asserts equal digests | inventory/tests.rs lines 312 to 349 |
| The digest changes when content changes | test `stale_attunement_can_be_replanned_without_reinstalling_or_losing_receipts`: after editing `README.md` the old plan is refused with `AttunementSourceChanged`, and the accepted replan has a different `plan_sha256` | tests/attunement/replan.rs lines 5 to 90 |
| Stale shape rejected | `attunement/inventory.rs` recomputes the project digest, refuses with `AttunementSourceChanged` if it differs from the plan's `source_sha256`, and checks again immediately before publishing | inventory.rs lines 150 to 200 |
| Shape persisted as a preflight input | the same function commits the proposal as records through `records::mutations` in one runtime commit | inventory.rs |
| Markdown rendering of the shape, `--output` file | **not carried** as a command. The Connectome installation preview shows the engine's plan facts (`check-installation-facts.mjs`). Convenience only | not read beyond the check's name |

The Python files themselves are not carried; the engine's Rust inventory is a strict superset. They remain readable at commit `73c75a852`.

### The contract (`consolidation-contract.md`)

| Claim | Disposition | Evidence read |
|---|---|---|
| One engine; CLI, hooks, MCP and UI are ports and keep no lifecycle logic | **carried** | `docs/architecture/engine-data-flow.md` "System boundary": `RrdEngine` is the "sole semantic transaction coordinator"; HTTP, SDK, MCP, CLI and Connectome "invok[e] public `RrdEngine` capabilities without recreating engine logic". The MCP adapter "cannot be a second context assembler, workflow engine". Lines 22 to 40 and 1369 to 1395 |
| Run the shape generator **before asking a model to plan** | **superseded** by [ADR-0002](../../../decisions/0002-adaptive-reasoning-governed-effects.md), accepted 2026-09-14: "Several records required a complete committed project inventory before exploratory reading or planning ... can make the system deterministic at the cost of making it unable to adapt." The inventory now binds persisted knowledge, mutations and evidence claims, not thinking | ADR-0002 lines 1 to 75; engine-data-flow.md lines 1192 to 1236 |
| Templates, instance bindings, generated runtime configuration, routine selection | **carried in the owner's later form**: immutable digest-bound routine definitions, runs pinned to a revision and input digest, skill packages, host hooks as stubs. Generation of per-project templates is kernel-contract package K2 (S1 settings and install templates), not yet implemented | engine-data-flow.md lines 1369 to 1395; kernel-contract.md K2 |
| Publish the same events to Connectome; Connectome keeps no ledger | **carried** | engine-data-flow.md "One canonical event path" and "Prompt-flight flow": "Clients render flights; none keeps a flight ledger, runner, or diagnostics protocol of its own" |
| Connectome becomes a separate repository and process | **done** | the former `connectome-ui` crate is absent from the engine tree; Connectome is `rrflow/connectome-development`, pinned as a submodule and consuming the public contract |
| Ordered recovery 1 (persisted shape, reject stale) | **done** | see the generator table |
| Ordered recovery 2 (one entry point returning context, route, plan, routine and hop ids) | **partly**: `flight-launch` is the observable entry point and returns context, ordered stage events and digests. Route, goal and routine are their own operations. The single-call collapse is kernel-contract package K0 (working skeleton) | engine-data-flow.md lines 554 to 606; kernel-contract.md K0 |
| Ordered recovery 3 (templates and bindings) | open, kernel-contract K2 | kernel-contract.md |
| Ordered recovery 4 (versioned protocol and event stream) | **done**: `rrd` protocol v1 with the `/v1/ws` live stream | prompt-flight section; Connectome's live view reads it |
| Ordered recovery 5 (UI moved to Connectome) | **done** | see above |
| Ordered recovery 6 (delete duplicate orchestration after differential tests) | open, kernel-contract K13 (repository convergence) | kernel-contract.md K13 |

The consolidation contract is **not** one of the seven records the kernel contract lists as superseded; this record gives it its own disposition.

## Branches 3 and 4 of 6: the two rrflowDB PostgreSQL branches (2026-10-19)

`backup/agent/rrflowdb-postgres-identity-20261029-1024` (tip `f661cc965`) holds one commit, `Clarify rrflowDB Rust database identity and PostgreSQL target`. `backup/agent/rrflowdb-postgres-plan-20261029-1024` (tip `d134734bd`) holds three: the same identity change again as `d1602b111` (same files, same sizes), `fb8b4db98` `Plan native rrflowDB PostgreSQL compatibility and independent release` (16 files, +674/-114) and `d134734bd` `Normalize plan line endings and record final verification` (5 files). All are documentation and one generator line. **Disposition: absorbed; every file was compared and every difference classified.** Read in full: the commits' messages and stats, and the diff of every file against `development/main`, ignoring line endings, with each removed branch line paired with its replacement on `main`.

| File | Result of reading |
|---|---|
| `repository/rrflowdb-postgresql-identity.md` (69), `repository/rrflowdb-postgresql-plan.md` (110), `repository/rrflowdb-postgresql-plan-format-verification.md` (33), `protocol/README.md` (19) | identical to `main`, apart from line endings |
| `reference/protocol/postgresql.md` (354) | six lines differ, all the `estate` to `engram` rename; carried |
| `architecture/system-overview.md` | 19 branch lines differ, each rewritten rather than dropped: the rename, the rrflowKV row split into KV, LSM and CLD rows, updated LFG, SWAG and RROFLOW status, and new rows for task, shippin.cloud, DevPulse, engram and second brain |
| `poam/poam-011.md` | no branch line is missing from `main`; `main` has 105 more |
| `poam/poam-013.md` | the branch's claim that no native Windows primary binary had run is replaced by newer evidence, and says so ("superseded by that evidence") |
| `objectives/rrflow-1.0-alpha.md` | rename, "ten" to "fifteen" objectives, expanded outcomes; nothing dropped |
| `roadmap/rrflow-1.0.md` | Gate D and Gate I titles updated (Gate I: "host hooks as alias stubs", the owner's 2026-10-28 decision); D-01 is now checked |
| `roadmap/rrflow-1.0/executable-dependency-spine.md` | the branch's twelve-wave order is **superseded** by the owner's later "one implementation order", which states that gate letters "are not a requirement to finish every storage benchmark before implementing a usable project" |
| `roadmap/rrflow-1.0/gate-h.md` | H-06 and H-07 expanded, H-08b and H-08d updated, H-09 to H-12 added; PostgreSQL stays H-08 and SQLite is H-12 |
| `roadmap/rrflow-1.0/gate-j.md` | J-01 and J-03 updated for alias stubs; 212 lines added |
| `roadmap/rrflow-1.0/current-alpha-objective-and-remediation.md` | the branch's Gate H "0 / 8" row is gone because `main` removed that hand-kept table on purpose; the parent roadmap's canonical checklist lists H-01 to H-12 |
| `README.md` | 32 branch lines differ, each replaced by later wording (D-01 complete, the rename, "Enter RRFlow" retitled "Find your way", the C-04 status) |
| `roadmap/rrflow-1.0-file-plan.jsonl` (1,193 lines) | on `main` it is a retired tracked projection (one descriptor line); the output is generated into untracked `target/` by `build_execution_inventory.py` |
| `scripts/ci/build_execution_inventory.py` | the branch's only change, planning `postgresql.md` under H-08, J-02, J-03 and J-05, is on `main` at line 1692 |
| `repository/README.md` index rows | all three journals are indexed on `main` |

**Defect found on `main` while reading.** `docs/architecture/system-overview.md` defines the term engram as "the owner's term that replaces "engram"". A bulk rename turned the word "estate" into "engram" inside the sentence that defines the rename. Fixed in this change.

## Branch 5 of 6: `backup/agent/c07-backup-scheduling-20261029-1024` (tip `2665f9911`, 2026-10-24)

One commit, `Retain integrated native growth failures and captured reopen admission`: a new journal, additions to POAM-035 and the Gate C-07 chapter, and two index rows. **Disposition: absorbed, with one instruction superseded by a later owner decision.** Read in full: the commit and every changed file against `development/main`.

| File | Result of reading |
|---|---|
| `gate-c/c-07g-integrated-native-growth.md` (104) | identical to `main`, apart from line endings |
| `gate-c/README.md` | no branch line is missing from `main`; `main` has 199 more |
| `poam/poam-035.md` | five branch lines differ, all the `estate` to `engram` rename |
| `roadmap/rrflow-1.0/gate-c-07.md` | the rename throughout; the sustained-run row updated with newer evidence (a frozen Desktop candidate acknowledged all 100,000 corpus operations); the DuckDB boundary replaced because the owner withdrew that direction; PostgreSQL row widened to PostgreSQL and SQLite; hooks stated as alias stubs |
| the branch's three-step "next packages" list (preserve each failed campaign; attribute and remove growth costs at their owner; freeze one candidate and run preflight) | **carried and extended** by `main`'s "Joined rrflowDB evidence sequence" (6 steps, lines 158 to 215): "Retain each failure and diagnose its actual boundary before another repair", "Grow in measured stages" over small, 1k and 10k fixtures before the 100k profile, and "Freeze each candidate" |
| the branch's instruction "Do not start new UI/provider/configuration packages while these storage blockers remain" | **superseded** by the owner's later "one implementation order" (see branches 3 and 4): a usable project comes first, and only a correctness failure stops the slice it affects |
| `roadmap/rrflow-1.0-file-plan.jsonl` | retired tracked projection, as above |

## Branch 6 of 6: `backup/agent/c07-run-progress-20261029-1024` (tip `65385004d`, 2026-10-25)

Two commits: `bec6e0d86` `chore: open C-07 comparable run progress package` and the merge `65385004d` of `d0c446df`. **Disposition: absorbed; nothing to carry.**

- `bec6e0d86` changes no file: its tree equals its parent's.
- `d0c446df1d7244212fe56e9daff8c82ec98d6d4b` is an ancestor of `development/main`.
- The branch tip's tree is byte-identical to `d0c446df`'s (`git diff` between them is empty).
- The phrase "comparable run progress" does not appear in `main`'s documents. The commit was an intent opened without content. The nearest real text is Gate C-07's sustained-run driver with its "progress/result receipts" and the sequence step "Record completion and per-stage progress"; this record does not claim more than that proximity.

## Summary

| # | Branch | Disposition |
|---|---|---|
| 1 | `agent/connectome-temporal-runtime-visualizer` | absorbed (ancestor of `main`; feature verified at its renamed path) |
| 2 | `codex/consolidate-project-structure-and-context` | carried in part, superseded in part; document preserved as history |
| 3 | `backup/agent/rrflowdb-postgres-identity-...` | absorbed |
| 4 | `backup/agent/rrflowdb-postgres-plan-...` | absorbed; the wave order is superseded by the owner's later order |
| 5 | `backup/agent/c07-backup-scheduling-...` | absorbed; one instruction superseded |
| 6 | `backup/agent/c07-run-progress-...` | absorbed; empty commit |

No branch was deleted. All six remain on `origin`; deleting them, and what to do about the official repository's `main` (which holds the commits I pushed there by mistake on 2026-11-16), are the owner's decisions.

## Gaps left open by this record

- The kernel-contract package K0 (one end-to-end call returning context, route, plan, routine and hop ids) and K2 (install templates) are the open homes for the consolidation contract's recovery steps 2 and 3. Neither is implemented.
- The markdown rendering and `--output` file of the old shape generator have no command on `main`.
- Where this record says "identical", the comparison ignored line endings.

## Not run

- No test was executed. Every row above comes from reading source, tests and documents on `development/main`; the documentation and navigation checks run before commit are listed in the pull request.

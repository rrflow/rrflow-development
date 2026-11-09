# Existing-project Windows admission characterization

**Status:** active failing characterization; installer repair incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/existing-project-admission-windows`
**Owner:** Gate D existing-project installation, POAM-006

Starting main revision `e6f3632b32f1c9be33c1951886a9d9a2be648ddf`, tree `85c2e6e27c77e0dc02abc7c31c509e4fa2655bdc`; draft [PR #247](https://github.com/rrflow/rrflow-development/pull/247). The PR brief owns the proposed repair. This record describes an intermediate characterization and does not claim the package or roadmap gate is complete.

## Scope and current result

The installer currently rejects existing inherited `.rrflow` directories before preview. The intended repair must preserve project-owned files, directories, and ACLs, bind any retained directories and collisions to the approved plan, retain strict protections for newly created private objects, and preserve interruption/replay and existing installed-estate security. Do not adopt PR #236's permission modification or relocation of existing entries. PR #236 remains open; no consolidation or closure is claimed.

The engine now includes the rejected estate-relative directory in its storage error, while retaining the same admission checks. A focused native Windows test confirms the error identifies `.rrflow` and preview has no file effects. This diagnostic does not make installation succeed.

This candidate adds a native object-identity digest on the existing `SourceObservation` and tests its stability when a child is created and its difference after replacing the directory while retaining the old object. This is host-local metadata only. IDs can be reused and the digest is neither content freshness nor permission authorization. It is not yet used by installation. A Windows installer test creates an existing inherited estate and sentinel files and requires preview, installation, reopen, replay, preserved bytes, and unchanged `icacls` output. Its preview currently fails, so later assertions have not run.

Changed paths: `crates/authority/rrd-engine/src/engine/estate_layout.rs`, `crates/persistence/rrd-lsm/src/filesystem/source.rs`, `crates/persistence/rrd-lsm/tests/filesystem.rs`, `crates/authority/rrd-engine/src/engine/tests/installation.rs`, this journal, the POAM-006 detail, and the generated journal index. No existing user project or ACL was mutated. No implementation was moved or deleted.

Relevant reads included `README.md`, the alpha objective, Gate D roadmap, POAM-006, change-authoring and traceability procedures; `engine/estate_layout.rs`, `engine/installation.rs`, `installation/admission.rs`, `installation/inventory.rs`, `installation/recovery.rs`, `installation/bootstrap.rs`, `token_key/private_file.rs` and Windows policy; `rrd-lsm` filesystem and source-observation code; the `rrd-contract` installation plan; and installation/observation tests. Some large files were examined by targeted sections, not reviewed completely. Full-file review of all modified files and the finished diff remains required before candidate readiness.

Research: [Windows file security and access rights](https://learn.microsoft.com/en-us/windows/win32/fileio/file-security-and-access-rights) describes inherited descriptors and access checking. No external code was adapted. Trace decision: preserve existing installation stage and recovery observations; no new tracing was added in this intermediate candidate. The filesystem tests characterize native identity, not durability across process death or protection against native ID reuse. No capacity/resource qualification is claimed.

## Commands and results

All commands used native Windows from the isolated worktree. `CARGO_TARGET_DIR` was `D:\localdev\devlabs\rrflow\target\d04-source-spans\target`.

- `cargo test --locked --offline -p rrd-engine --lib existing_project_preserves_inherited_estate_and_opens_after_replay -- --exact --nocapture`: built in 1m42s but selected **zero tests**, because `--exact` did not include the module path. This is not a pass of the oracle.
- `cargo test --locked --offline -p rrd-engine --lib existing_project_preserves_inherited_estate_and_opens_after_replay -- --nocapture`: **one failed**, at plan creation: `Storage("private object requires the current owner and a protected non-null DACL")`. No installation effects were attempted.
- `cargo test --locked --offline -p rrd-lsm --test filesystem native_directory_identity_survives_child_creation_but_detects_replacement -- --nocapture`: **one passed**.
- `cargo test --locked --offline -p rrd-lsm --test filesystem`: **12 passed**.
- `cargo test --locked --offline -p rrd-lsm`: passed the full package suite; the unit target reported 59 passed and two explicitly ignored, and the integration and documentation targets completed successfully.
- `cargo test --locked --offline -p rrd-engine --lib inherited_estate_preview_reports_the_rejected_directory -- --nocapture`: **one passed**.
- The installation-preservation test was rerun after the diagnostic change: **one failed**, now naming `estate directory .rrflow`; the preview still did not succeed.
- `cargo fmt --all --check`: initially failed on formatting in the new test; `cargo fmt --all` applied the formatting, and a second `cargo fmt --all --check` passed.
- `git diff --check`: passed after formatting.
- `python scripts/knowledge/render_navigation.py`: initially failed because PowerShell wrote a UTF-8 byte-order mark before the journal title; removed the mark and reran the generator successfully; `python scripts/knowledge/render_navigation.py --check` also passed.

The complete engine suite, contract/generator checks, restart and interruption cases, Linux, macOS, candidate CI, real-project install, Connectome, C-07 capacity, and soak were not run. The installation test's intentional failure and incomplete ownership/preview/recovery implementation keep this PR draft and prohibit merge. No alpha, gate, or POAM status changes.

## Checklist and next action

- [x] Reproduced a concrete before-oracle at the engine installation boundary.
- [x] Recorded research, affected boundaries, relevant risks and failure.
- [ ] Represent preexisting ownership, preconditions and retained native identity in the sealed preview and recovery contract; avoid weakening previously private estates.
- [ ] Implement collision-safe admission and recovery with inherited parent permissions and private owned descendants; test changes between preview/apply, interruption, restart and replay.
- [ ] Verify installed Windows flow and existing-state/ACL preservation, then run owning suites and the required candidate CI.
- [ ] Complete full-file and diff review, update the package record, and merge only after all required checks pass.

## 2026-11-09 absorption onto main

Merged `development/main` cleanly. The preservation oracle
`existing_project_preserves_inherited_estate_and_opens_after_replay` still
fails, as this package recorded: no in-place preservation repair exists. The
owner directed every authored line onto main, so the oracle lands marked
`#[ignore]` with the POAM-006 reason; the preview-reporting and native
directory-identity tests run and pass (63 passed, 1 ignored). #236 lands the
relocating adoption separately; whichever behaviour the owner finally wants,
this oracle is the test to un-ignore.

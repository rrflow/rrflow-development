# Archived project installation integration

**Status:** active integration evidence; full existing-project qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-project-installation-integration-20261121`
**Owner:** Gate D / POAM-006 existing-project installation and source recovery

## Before-edit brief

Baseline d4a5ca068ac5d35de2a0b13cd8d1adb72e93302b, green PR480.

Integrate the original existing-project installation and project-map sources through ordinary merges while retaining the current canonical implementation.

Outcome: Gate D / POAM-006 existing-project installation and source-derived project navigation; source-recovery prerequisite, not alpha completion.

Sources: 420f35e6695c1b257573affca1a2040ec9d3f5be (old adoption contract), d76fee09699b9eca81aedbd18452c8e4b5760e64 (retained-directory installation), 0976c4ed1ff1599043d6429b93ebe021e7ddbdda (project-map projection). Driver: this integration session. Dependency: green PR480; subsequent worker/vector/parser archives remain separate work.

Before implementation: read the complete source patches for the old adoption contract, retained layout/install/publication/security/test changes, project-map source and its CLI/inventory/tests; current engram_layout admission, installation/storage publication, retained_engram tests, project-map implementation/tests, POAM-006 retained-admission paragraph and its Nov15/Nov18 evidence. Inspect remaining conflict regions before resolving them.

Traceability: old relocation adoption is superseded by the Nov15 retained-directory owner decision: keep project-owned entries and ACLs in place. Current engram_layout and installation/{planning,admission,apply,open,validation,storage,bootstrap} own retained identities, publication proofs and recovery. Keep current macOS ACL support, strict fresh-directory privacy, and canonical source-language/inventory policy. Map old test monoliths into their existing split tests. Project-map's notes/no-rewrite behavior and CLI install/serve consumer remain in engine/project_map.rs and CLI installed.rs. No obsolete second implementation, public schema field, private thought schema, or directory migration is restored.

Scope: these three source merges; existing canonical owners only if a missing behavior is established; linked repository journal and generated index. Preserve current submodule pin and source-independent package behavior. No feature parity or redesign claim follows a merge.

Failure oracles: retained installation preserving files/ACLs across replay and process death; identical replacement and unproven staging refusal; current attunement contract; project-map notes/no-change and reopening; installed lifecycle driver. Commands: cargo test -p rrd-engine --lib retained_ --locked, engine --lib installation, engine --lib project_map, contract --test attunement_contract, CLI --test installed_lifecycle, engine --test workspace_architecture; documentation and diff checks; candidate and queue CI. Dedicated CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target, four jobs.

Research: original in-repo provenance and current native-security owners suffice; no new native APIs or upstream algorithms proposed. Existing failure-stage hooks and daemon receipts are the diagnostic boundary; no benchmark/visual comparison, new tracing, or performance claim. Stop on unmapped behavior, dropped assertion, red CI, or unrelated uncommitted work. Record exact results and remaining gaps before ready.


## Source resolution

The three originals were merged separately: 420f35e6 in ba4002be7,
d76fee096 in 8943c515, and 0976c4ed in 8f9463f9. All source objects remain
in ordinary ancestry. No implementation or test delta remains against the
starting main: the current owners already carry the reusable behavior, with
later corrections preserved. This package records source integration, not a
new product capability.

- The old adoption contract would relocate project-owned entries into a
  previous directory. Its contract field and tests are superseded by the
  retained-directory decision in the Nov15 existing-project journal. Preserve
  the current RetainExistingDirectory contract and proof validation, rather
  than restoring relocation or its backward-compatible digest branch.
- The retained-directory source's layout maps to engram_layout.rs. Current
  admission distinguishes installing, retained and fresh private directories;
  an unproved non-private directory is not automatically accepted. Existing
  identity/security preconditions, private staging and publication ownership
  proofs are in installation/{planning,admission,apply,open,validation,storage,
  bootstrap}. The old installation monolith is not restored.
- Source native-security functions and tests remain in token_key/private_file
  and installation/retained_engram, configuration_and_adoption, concurrency,
  and recovery. Keep the subsequent macOS ACL descriptor implementation, the
  corrected Windows fixture, and the rejection of an already installed locator.
  The source's duplicate digest import was removed during conflict resolution.
- Original source contract tests map to attunement_contract/{validation,
  operations,golden}; their identity/security/proof oracle remains executable.
  The original intermediate journal already exists as explicitly superseded
  history; its old active-status text was not restored.
- The project map still renders the source census and retains owner notes,
  skips writes when unchanged, and is called by the installed CLI. Its old
  custom walk and language match now map to the profile-governed inventory
  and shared source_language owner. Preserve Git observation/retry/budget
  checks and source exclusion tests. Do not reintroduce the old engine ignore
  dependency, collect_project_entries re-export, or obsolete test monolith.

Read original patches for the adoption contract and tests; retained layout,
installation, admission, bootstrap, platform security and test additions; the
complete original project_map.rs and its CLI, inventory and test changes.
Current reads covered layout admission, planning, publication storage, retained
installation tests, project-map tests and implementation changes, current
native macOS descriptor section, relevant contract symbols, source inventory
oracles, and the Nov15/Nov18 owner journals plus POAM-006. This is a
package-specific review, not an exhaustive whole-codebase line-review claim.

## Verification

At 8f9463f9, using the dedicated target directory and four jobs:

- cargo test -p rrd-engine --lib retained_ --locked: 15 passed, including
  retained-directory process-death/recovery and unchanged user files/ACLs.
- cargo test -p rrd-engine --lib installation --locked: 84 passed, one
  intentionally ignored subprocess fixture.
- cargo test -p rrd-engine --lib project_map --locked: 4 passed.
- cargo test -p rrd-contract --test attunement_contract --locked: 24 passed.

- cargo test -p rrflow-cli --test installed_lifecycle --locked: all 17 passed,
  including native process restart/replan/readback, fresh restore, concurrent
  instance registration, isolated temporal configuration and clean stop.
- cargo test -p rrd-engine --test workspace_architecture --locked: 35 passed.
- Documentation/navigation and diff checks run before the candidate commit.

 Source-inspection commands initially named two moved paths and one
PowerShell-incompatible brace glob; they failed without changing product
files and were corrected using the tracked path inventory. No external
research, hosted sync, UI, visual comparison or capacity qualification was
performed. POAM-006's remaining installation qualification gaps stay open.

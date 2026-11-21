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

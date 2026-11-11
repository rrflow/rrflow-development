# D-02 P1: capability catalogue and evidence-derived successor plans

**Status:** active implementation evidence; D-02 and OBJ-07 remain open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-evidence-derived-attunement-20261111`

**Owner:** OBJ-07 / OBJ-14; [Gate D project-workflow packages](../../../roadmap/rrflow-1.0/gate-d.md#project-workflow-build-packages); package P1 of the [D-02 evidence-driven attunement plan](../../../roadmap/d02-evidence-driven-attunement-engineering-plan.md)

**Starting revision:** main `ab32ec16`, after #324 (the plan) and #325 (P0).

**Current behavior.** Both bundled profiles declared eleven attunement phases; `plan_installation_with_options` copied them into every install plan; the engine executes Connect and Inventory and failed every other phase with `phase-unavailable`; a replan (`make_attunement_replan`) could change nothing but the plan id and source digest, and `AttunementReplanPreview::validate_for` required the successor's phases to equal the predecessor's; discovery refused to list past a succeeded predecessor. Nothing selected work from evidence.

**Target behavior.**

- The contract gains a capability catalogue (`ATTUNEMENT_CAPABILITIES`): each phase with a version, the capabilities it requires and a precondition over `AttunementEvidence` (committed inventory counts, files per language, bound grammars, an applied embedding binding, and the executable set). `derive_attunement_phases` is a pure function of evidence; `derive_attunement_plan` builds the successor plan whose step digests bind the evidence digest and carries a default parse policy for bound grammars or the predecessor's operator-selected one.
- `AttunementReplanPreview` carries optional `evidence`; when present the preview validates only against the plan the evidence derives, so a client can recompute and verify it. Without evidence the previous rebind rule holds.
- The profiles plan only Connect and Inventory and declare `available_capabilities`; the engine intersects them with `EXECUTABLE_PHASES` (Connect, Inventory today), the one owner of what can run. A capability without an executor is never selected, so a derived plan never reaches `phase-unavailable`. An explicit operator proposal still selects the initial plan by hand within the offered capabilities. A replan after a committed Inventory derives from evidence and replaces that selection, including a selected Parse step that has no executor; before an Inventory checkpoint a replan still rebinds the operator's plan.
- After an Inventory checkpoint, `make_attunement_replan` reads the committed records at the head the checkpoint published (refusing if the head moved), derives the plan, checks derived parse policies against the installed limits, and previews it with its evidence. Discovery accepts a succeeded predecessor at its own revision.
- A shared extension-to-language table (`engine/source_language.rs`) feeds both the evidence and the project map; the parser crate exports its supported language ids.

**Changed paths.** `crates/transport/rrd-contract/src/attunement/derivation.rs` (new), `attunement.rs` (module, preview field, validation), `lib.rs` (exports), `tests/attunement_derivation.rs` (new), `tests/attunement_contract.rs`; `crates/compute/rrd-attunement/src/parse.rs` (`SUPPORTED_LANGUAGES`); `crates/authority/rrd-engine/src/engine/{source_language.rs (new), mod.rs, project_map.rs, installation.rs, attunement.rs, attunement/evidence.rs (new), attunement/discovery.rs}`, `assets/install/{default,clyffy}-profile-v3.json`, `engine/tests/attunement.rs`, `engine/tests/attunement/derivation.rs` (new), `engine/tests/installation_bootstrap.rs` (profile digests); `crates/adapters/rrflow-cli/tests/installed_lifecycle.rs`; `docs/reference/deployment/installed-lifecycle.md`, `docs/reference/agent-bootstrap.md`; regenerated contract projections and storage goldens; this record.

**Oracles, written after the module.** The plan's rule is a red oracle committed before the code; this package did not follow it. `derivation.rs` was written first and its tests second, and neither was observed failing against the previous behaviour. `crates/transport/rrd-contract/tests/attunement_derivation.rs` (six cases: catalogue order, executable-only selection, preconditions, evidence binding and default policy, inconsistent evidence, derived preview validation) and `engine/tests/attunement/derivation.rs::default_plan_completes_after_inventory_and_derives_the_successor_from_evidence`. The two engine tests that characterized `phase-unavailable` now opt into it through an explicit proposal, since the default plan no longer contains an unavailable step. The installed-lifecycle driver's `expect_parse` branch is kept as the D-04 oracle: when the Parse executor lands, the evidence makes it available and the derived plan must select and checkpoint it.

**Acceptance.** ACCEPTANCE_PLACEHOLDER

**Real run on this repository.** REALRUN_PLACEHOLDER

**Research decision:** the catalogue-with-preconditions shape follows the plan's research (SCIP incremental indexing, Nx graph inference and tag boundaries, LlamaIndex content-hash caching); no upstream code.

**Checks not run:** CHECKS_PLACEHOLDER

**Roadmap/POA&M change:** none. POAM-059 stays open: the product path now plans and completes without a hand-written plan, but parses, indexes and links nothing until P3 to P5.

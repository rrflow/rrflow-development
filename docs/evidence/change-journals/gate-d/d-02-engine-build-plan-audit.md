# D-02 engine build-plan audit

**Status:** active planning evidence; runtime implementation held and API admission unresolved
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-engine-build-plan-audit`
**Owner:** OBJ-07 and existing D/E/F/G/H/J build and evidence requirements

## Brief and baseline

Starting revision `337d136210ff59b2359f898e717166c087dd5bfc`, tree `f4008995c7e17884ec23c0a71f8648fe2961f7b0`; development main
`dd225072bcc639c89b0eef968625cc73303ea998` was fetched before work.
[PR 137](https://github.com/rrflow/rrflow-development/pull/137) published the
brief before edits. It depends on the priority correction in
[PR 136](https://github.com/rrflow/rrflow-development/pull/136), incorporated by
fast-forward from a branch created at development main. Runtime work is held
under the owner's current instruction until planning and necessary decisions
are captured. This package performs research and documentation only.

Before: objectives and gate tables existed, but next-package admission was not
resolved and the design rationale/Gate G contradicted the corrected order.
After: existing owners define the observable engine journey, package readiness,
D-02a/b/c and D-03/D-04/D-05 inputs/outputs/authority/failure oracles, consumed
E/F paths, a joined real-source fixture, precise evidence limits and remaining
API/configuration admission blockers. Full thirteen-objective alpha remains
unchanged. This is build-plan clarity, not an implemented worker or alpha.

## Code-grounded findings and coverage

Complete focused files reviewed in this package: Gate E, Gate F and the
current-alpha-objective-and-remediation portal. Targeted sections and source
searches: README identity/status; objectives/journey/exit; dependency spine;
Gate D requirements and package context; Gate G package prerequisites; Gate H
operation/UI requirements; Gate J capacity/quality profiles; POAM-006 observed
deficiencies; engine-scale-and-contracts next-cut/research sections; engine
transaction/data-flow architecture; installed-lifecycle/schema/PostgreSQL owner
headings. All changed passages and the complete diff were reread. This is not
a claim of full-file or full-codebase implementation audit.

Exact source boundaries inspected:

- `crates/transport/rrd-contract/src/attunement.rs`: phase plans, estimates,
  installation action enum, job transition, lease and checkpoint definitions.
- `crates/transport/rrd-contract/src/lib.rs`: exported attunement types,
  `endpoint_catalogue` and attunement occurrence search. No advertised operation
  for attunement status/resume/cancel was found.
- `crates/transport/rrd-contract/tests/attunement_contract.rs`: search confirms
  contract-fixture/validation use, not worker execution.
- `crates/authority/rrd-engine/src/engine/task.rs` and public task contract:
  open/close named durable records; explicitly no orchestration.
- `crates/transport/rrd-server/src/http/handlers/task.rs`: existing authenticated
  task handlers do not dispatch attunement.
- `crates/authority/rrd-engine/src/capabilities.rs`: catalogue projection;
  source searches across engine/transport show no attunement request handlers.
- `crates/adapters/rrflow-cli/src/installed.rs`: installed command dispatch and
  verification, not a worker entry point.
- `engine/installation/inventory.rs`, configuration enforcement and existing
  deployment contract were checked by relevant source searches and prior
  reviewed baseline. Installation precondition inventory remains distinct.
- Generated file plan identifies planned `engine/attunement.rs`,
  `tests/attunement_resume.rs`, and `crates/compute/rrd-attunement/` modules;
  these were not presented as existing source or runnable tests.

The missing action mapping, sealed-configuration limitation and estimated versus
enforced budget distinction are recorded in existing POAM-006. A clarification
was requested about preserving current API behavior versus freezing the entire
operation set. No answer is inferred, and no API source change is made. The
plan explicitly blocks public control/configuration admission pending resolution.

## Research and traceability

The engineering architecture skill was used to evaluate requirements,
dependencies, alternatives and falsifying evidence; existing repository owners
were retained rather than creating another ADR/roadmap authority.
Primary sources were inspected online on 2026-10-25: Google SRE pipelines,
PostgreSQL 18 WAL, Qdrant hybrid queries, BEIR v4 and GraphRAG v2. Their links,
limited applicability, existing gate mapping and comparative experiments are
recorded in [the owning rationale](../../../architecture/engine-scale-and-contracts.md#evidence-based-build-decisions).
The OpenTelemetry site explicitly points to its separate GenAI conventions
repository; pinning that changing specification is deferred to H-05, not
silently adopted. No upstream code/dependency or vendor superiority is claimed.

Trace/resource/debug decision: no new runtime instrumentation. Define real
source/configuration/model coordinates and independent readback, distinguish
measured provider/local-model tokens from retrieval work and estimates, and
include ingestion/index/maintenance costs in future comparison. Hidden model
reasoning is not a proof artifact.

Structural map: old scheduling text remains at its existing owner and is
replaced by the current dependency link and explicit decision boundary. Existing
objective status rows, gate checkboxes, runtime implementation, API and wire
contracts, version, binary artifacts, Connectome gitlink and historical journals
are preserved. Only owning docs, this evidence node and generated projections
change. No semantic source consolidation or deletion is claimed.

## Acceptance, errors and remaining work

First failure/characterization: source search found attunement declarations and
tests without the needed handlers; `task.rs` prohibits treating task state as
orchestration; Gate G and the rationale retained obsolete scheduling. These
mismatches are now explicit, with D-02a/D-02c remediation and no fake API route.
Initial documentation validation passed after edits: 609 statuses and 607
classified coordinates. Final results follow below after journal/index refresh.

Command errors retained: literal wildcard path arguments failed under Windows;
guessed `engine.rs`, `public.rs` and CLI `cli.rs` files did not exist. Subsequent
`rg --files` and source-directory searches located the actual modules. Some
oversized read output was truncated; changed diffs were reread in bounded groups.
An unquoted `HEAD^{tree}` in PowerShell was misparsed; the revision/tree was
retrieved correctly with Python subprocess argument arrays above. No such error
is counted as accepted evidence or used to infer missing behavior.

No Cargo/native/runtime tests or new benchmark/soak were run: there is no runtime
edit, and the owner requested planning before further implementation. Proposed
phase tests are deliverables, not fictitious passing commands. Existing runs
retain their prior identity; this package makes no current soak-status claim.

Checklist: brief before effects; reviewed baseline and targeted coverage stated;
user outcome, exact scope and stop conditions bound; missing capabilities and
ambiguities explicit; source research and limitations linked; no duplicate
engine/plan authority; full changed diff reviewed; tests not run disclosed;
existing owner statuses unchanged; generated index and inventory regenerated.
Stop on API mutation, invented availability, lowered acceptance or implementation
before the required decision. PR 137 remains draft for the unresolved admission
and dependency, with no merge or candidate-CI pass claimed.

Final local acceptance commands all passed:

- `python -X utf8 scripts/ci/check_documentation.py`: 610 statuses,
  608 classified coordinates, ownership/index/local-link validation.
- `python -X utf8 scripts/knowledge/render_navigation.py --check`:
  17 indexes, 609 nodes, 2421 edges, zero drift.
- `python -X utf8 scripts/ci/build_execution_inventory.py --check`:
  1732 current/generated/planned records.
- `python -X utf8 scripts/ci/check_workflow.py`: 3 workflows, 9 substantive
  jobs, 5 engine suites, 20 default/6 optional packages, 19 pins.
- `git diff --check`: no whitespace errors.
- Inline exact baseline comparison: objective status rows and roadmap checkbox
  ID/status pairs unchanged; scope is README/docs only; runtime/API/scripts,
  version/lockfile, Connectome and retired active-change record unchanged.

Generated inventory was refreshed after recording these results. The proof is
planning consistency and explicit blockers, not runtime correctness. The API
clarification remains pending at commit; elapsed time is not authorization.

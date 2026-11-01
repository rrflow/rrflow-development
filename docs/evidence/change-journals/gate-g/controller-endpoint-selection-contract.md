# Controller endpoint-selection capability preparation

**Status:** active contract preparation; no executable selection or dispatch
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-endpoint-selection-contract`
**Owner:** G-01b and POAM-009; OBJ-12 learning prerequisite
**Pull request:** [196](https://github.com/rrflow/rrflow-development/pull/196)

## Baseline, brief and observed gap

This package branches from public main
`291fee511887932572ecc9935e9786e098727677`. An empty opening commit made the
draft PR and its change brief reviewable before repository edits. It adds no
fake implementation or data. The SWAG working session is the package driver;
PR188's collector is independent and is not a runtime dependency.

The public catalogue currently exposes endpoint configuration read/preview/apply,
and the engine persists those configurations. The inference execution boundary
provides embedding operations. `RouteStepDecision` is the separate three-member
recipe/branch/context union. Neither is an executable learned endpoint selector
or authorization to call the chosen provider. A local BF16 endpoint and offline
proposal are not substitutes for that missing capability.

## Inspected owners and exact change

Reviewed README, AGENTS, the PR template, change-authoring procedure, dependency
spine, Gate G, POAM-009, endpoint-binding reference, LFG/SWAG and learning owners.
Implementation inspection covered endpoint binding/configuration types and
validation, public catalogue configuration entries, the route-decision union,
engine endpoint configuration methods, embedding dispatch and router-model
pre-load admission. These code sections establish the boundary, not a claim
that unrelated files or all runtime paths were audited.

The change extends the existing LFG/SWAG owner with a distinct proposed selection
capability: configuration-derived eligibility, separately authorized controller
and worker calls, finite packet/proposal semantics, explicit unavailable facts,
relevant source/configuration identity, successor-plan behavior and real-provider
acceptance. Names in that design are not registered operations or available SDKs.
The learning owner records the user's real-data requirement; Gate G links the
implementation handoff and POAM-009 retains the missing runtime work.

Changed paths: `docs/reference/inference/lfg-swag.md`, its learning owner
`learning/controller-training.md`, `docs/roadmap/rrflow-1.0/gate-g.md`, and
`docs/poam/rrflow-1.0-alpha/poam-009.md`. This journal is new; its nearest index
is generated. The ignored execution inventory is regenerated from Git. No
source/type/schema/SDK or historical journal is rewritten, moved or deleted.
The endpoint profile and the three-decision router retain their existing owners.

## Design choices, proof and limits

Use a distinct selection capability rather than overloading route decisions or
embedding calls. Reuse engine configuration, task budgets and external-activity
authority rather than a Python/UI provider loop. This makes a missing prerequisite
visible and keeps proposal, effect authorization and learning admission distinct.
It requires real adapter/public-operation implementation before a usable workflow
can be claimed; the document does not register placeholder operations to imply it.

No external algorithm or provider library is selected or adapted. Primary provider
documentation and actual capability/cancellation evidence will be needed for the
chosen adapter package. No GPU experiment, resource benchmark, runtime trace or
synthetic dataset is applicable to this documentation-only boundary decision.
Preserve unknown cost, confidence and outcomes rather than populate missing data.

The falsifying oracle is the current absence of the actual selection/dispatch
operation: documentation must continue to report it as missing. Validate local
links/ownership, generated navigation, workflow policy, generated inventory and
the complete diff. Full candidate CI still applies before protected landing.
No runtime tests, real-task acceptance or model qualification are claimed here.

Next handoff: implement the distinct contract validators and public admission
action with the necessary engine handler, then the configured real adapter at
`rrd-inference`/`rrd-engine` in bounded packages under the existing prerequisite
order. The owner's first actual task and success rubric have been requested;
that input is required for the user-visible trial, not for this source preparation.
No route ranking is hardcoded and no training data is invented while waiting.

## Local validation results

`python scripts/ci/check_documentation.py` passed with 731 document statuses
and 729 classified coordinates. `python scripts/knowledge/render_navigation.py`
and `--check` passed with 18 indexes, 730 nodes and 2,830 edges.
`python scripts/ci/build_execution_inventory.py` and `--check` passed with
1,885 records. `python scripts/ci/check_workflow.py` and
`git diff --cached --check` passed. The complete six-file staged diff contains
only the four owner/reference changes, this journal and the generated index.
No runtime code, data, model or public catalogue entry was added.

The inventory is refreshed after this final evidence paragraph. Full candidate
CI and protected landing remain pending; skipped draft runs are not acceptance.

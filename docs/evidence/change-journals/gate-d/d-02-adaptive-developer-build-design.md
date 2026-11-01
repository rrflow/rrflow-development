# D-02 adaptive developer-build design evidence

**Status:** active documentation candidate; no runtime capability accepted
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-adaptive-developer-build-design`
**Owner:** [Gate D project-workflow build packages](../../../roadmap/rrflow-1.0/gate-d.md#project-workflow-build-packages)

## Package and starting point

OBJ-06/07/09/14 and Gate D D-02 through D-05, with H-06/H-08 and
POAM-006/038–041 as related owners. Main started at
`2cadd3a9045e3c1a2d9f873fe4215343680ab18b`. Draft
[PR 243](https://github.com/rrflow/rrflow-development/pull/243) contained the
change brief before the first repository edit. The empty bootstrap commit was
`14fb92abba99dfb64d895e7b74b790680c02fe06`.

The package records the fixed default and a proposed replacement/acceptance
path. It changes no runtime, schemas, SDKs, Connectome source or status. PR 241
owns a separate unmerged Parse draft and also touches POAM-006; integration
must preserve both records. PR 242 is a separate rrflowQL repair candidate.

## Review and decisions

Read in full: the engineering system-design skill and the repository's
`docs/roadmap/rrflow-1.0-execution/change-authoring.md` procedure. Inspected
the relevant sections of README, ADR-0002, the alpha objective, Gate D,
implementation navigation, CI policy, POAM-006/038–041, Connectome and
PostgreSQL contracts; main's two install profiles, attunement contract,
installation proposal/fallback, engine dispatcher/replan, inventory
publication, capability projection and attunement tests; and the pending
Parse draft. The Connectome worktree inspected in another checkout was at
`c1cbff203ecee89013878609d11aed04e89d0e71`, not main's pinned
`38142b77110a8f1db77e306e7d1ad375fd918267`; its contents are not evidence for
the pin. A pinned-revision client audit remains necessary.

Subsequently inspected `src/rrflow/attunement.ts` and targeted project,
native SDK client and estate source at the exact pin, via GitHub's contents
API and `git grep` against the existing submodule object. The phase module
defines eleven static views and a 30–45 minute estimate; searches for the two
constant names found only their definitions and a module-import search found no
matches. The native client has authenticated SDK session/query code and the
estate source has preview and digest-bound installation. These observations
do not establish an end-to-end desktop test or a complete UI audit.

Main's profiles name eleven phases. Caller-supplied ordered plans are
supported; the default fallback, closed enum, terminal Verify rule and linear
checkpoint chain still prevent calling the default dynamic. Connect and
Inventory have execution paths; the installed dispatcher reports unavailable
for the other phases. Existing replan is limited to specific locally fenced
states and copies the old phase sequence. These are source inspections, not a
new full-process test.

Research used the pinned SurrealKV commit ring and WAL crash tests, Turso's
PostgreSQL compatibility record and differential simulator, Pinecone Nexus
curation/evaluation description, and OpenTelemetry GenAI conventions, linked
from the design. No upstream code was copied. The design specifies required
receipt/trace/resource evidence; no runtime instrumentation was added.

## Change checklist and evidence

- Brief, baseline, owners, scope and stop conditions: PR 243 before editing.
- Changed: Gate D navigation and POAM-006 deficiency. Created: supporting
  design and this journal. Navigation indexes are generated from the records.
- Replacement map: included in the design; no implementation was removed.
- Smallest characterization: source inspection of both profiles, installation
  fallback, plan/checkpoint validation, dispatcher and replan. It establishes
  the named design gap but does not exercise the installed product.
- Documentation validation, generated navigation and full diff review:
  results recorded below before commit.
- Tests not run: engine, native process, Connectome, PostgreSQL, C-07 capacity
  and soak. This documentation-only package cannot establish their results.
- Remaining: exact existing-estate compatibility decision, capability-owner
  design, safe parser execution, pinned UI audit, and implementation/real
  process acceptance. No roadmap or POA&M completion status changes.

## Validation

`python scripts/knowledge/render_navigation.py` generated 18 indexes;
the final `--check` passed with no drift, 911 nodes and 3272 edges.
`python scripts/ci/build_execution_inventory.py` and its `--check` both
passed with 2297 current, generated and planned records. `git diff --check`
passed before staging.

`python scripts/ci/check_documentation.py` first rejected the two new records
because their statuses did not begin with `active` or `historical`; after that
correction it rejected their missing owner fields. Both metadata defects were
corrected. The final `python scripts/ci/check_documentation.py` passed with
912 document statuses, 910 classified coordinates, parent indexes and local
links checked. The changed records and complete staged diff were reread;
`git diff --cached --check` passed. Candidate CI and merge remain pending.

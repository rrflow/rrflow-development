# One-engine execution and handoff plan

**Status:** active proposed planning evidence; no implementation or gate completion claimed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/one-engine-execution-handoff-plan`
**Owner:** OBJ-14, existing D/C/E/F/G/H/I/J owners and repository change-authoring procedure

## Brief and baseline

[PR176](https://github.com/rrflow/rrflow-development/pull/176) carried the bounded brief before file edits. Starting public development main `9dea4eae5f904ba479a2df1898987b807e276172`, tree `2b047ed69ae93206124cdbead8ff4651a4fe16f0`; empty branch bootstrap commit `f53c82a4708335912648afb6d1f237e51adf32f7`. This session is the driver. The owner requested a complete researched plan, a schema for auditable change handoffs, and questions remaining after review. PR172 was verified merged with candidate CI passed; PR148 and PR171 were verified open drafts and conflicted. No implementation, release status or promotion is claimed.

Before: the existing roadmap and PR/journal policy describe requirements; no deployed handoff validator enforces an agent's plan per change. After: a supporting execution plan, proposed machine-readable schema and fictional example identify package ownership, gates, independent oracles, trust and failure boundaries, and a bootstrap path for actual future enforcement. Neither the schema nor this document enforces product effects or repository CI. The example is not a real approval or project plan.

## Source review and research

Read README identity and product warp points; the dependency spine, Gate D project packages, C-07 characterization and native matrix, Gate I provider observation, OBJ-14, POAM-006/035/038-041, ADR-0002 and change-authoring. Inspected `rrd-engine` attunement control/connect, retained root, installation inventory, knowledge binding and hybrid retrieval; the operation catalogue and HTTP read/cancel dispatch; source-bound design PR148 and its branch; the existing candidate-admission policy tests; PR172 candidate and merge status and PR171 status. Read the plan, schema, example and architecture index in full and reviewed their diff. No implementation was moved or deleted.

Primary references are linked in the plan: JSON Schema conditionals and RFC 8785, Bazel Skyframe, AWS idempotency, in-toto attestation, Qdrant quantization, Google TurboQuant, Claude Code monitoring, and Grok headless streaming. No upstream source adapted. No runtime trace/resource result is asserted; this is a design and schema example, not a benchmark.

## Validation and failures

`uv run --offline --no-project --python 3.11 --with jsonschema python D:/localdev/devlabs/rrflow/target/one-engine-schema-check.py` checks the Draft 2020-12 schema, validates the fictional engine example and an in-memory fictional repository receipt, and rejects ten structural invalid examples. The test confirms a deliberately mismatched approval digest is still structurally valid: future semantic authorization must reject it. The first run unexpectedly accepted malformed expiry because the available JSON Schema format checker lacked its optional date-time dependency; an explicit UTC shape pattern was added and the rerun passed. The future verifier must also validate the actual calendar and expiry. The test helper is a local planning check under `target/`, not a shipped runtime dependency or enforcement mechanism.

An initial oversized PowerShell write request was rejected by the command runner; the plan was authored in smaller patches. The first documentation-policy check rejected the coordinated plan's status because it lacked the required `active` or `historical` marker; the status was corrected to `active proposed`, which does not claim implementation. `python scripts/knowledge/render_navigation.py`, its `--check`, `python scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py`, both JSON parse checks and `git diff --check` passed after that correction. Final execution inventory is regenerated and checked before commit. User choices for the real project and observation content scope remain questions, with conservative assumptions only; per-estate credentials and grants are never inferred.

## Checklist and limits

- [x] Bound owner, baseline, exact scope, evidence and stop conditions in the draft PR before edits.
- [x] Reviewed current source/evidence and primary research; preserved roadmap and engine authority.
- [x] Validated schema structure and examples, including a demonstrated semantic-validation limitation.
- [x] Navigation, documentation, workflow, JSON/schema examples, inventory and diff checks.
- [ ] Candidate CI and merge are pending; this plan does not activate a validator.
- [ ] P0 trusted repository CI verifier, engine-owned plan/question/replan, actual source ingestion, complete memory/audit effects, provider adapters, C-07 native matrix and 24-hour soak remain open.

Created `docs/architecture/one-engine-execution-plan.md`, `docs/architecture/schemas/one-engine-handoff-v1.schema.json`, `docs/architecture/examples/one-engine-handoff-v1.json` and this journal; changed the architecture index and generated navigation/inventory. No objective, roadmap or POA&M status changed. Keep the PR draft for owner review of this plan and its decisions.

# H-06 turnkey project and capability audit

**Status:** active audit evidence; complete turnkey flow remains unqualified
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-06-turnkey-capability-audit`
**Owner:** OBJ-07/09/11, D-02/D-05/D-06, G/H and I learning prerequisites

## Scope and baseline

Engine source baseline `689ccca60ec90d4132daa8351febdd669a48518b`.
Connectome gitlink/source `38142b77110a8f1db77e306e7d1ad375fd918267`;
its native SDK dependencies pin engine `4860a253663c9c757800aafeb40890b76114c4f0`.
[PR 137](https://github.com/rrflow/rrflow-development/pull/137)'s bounded audit
brief was extended before these edits at the owner's request. No engine, API,
UI, submodule, model, service or runtime configuration is changed. The earlier
D-02 audit journal remains immutable; this is the successor evidence for the
turnkey/competitor/effort/service questions, not a new roadmap.

## What the evidence actually supports

| Requested behavior | Inspected evidence | Conclusion and owner |
|---|---|---|
| Add project and provision DB/KV/QL | Accepted D-01 installation owner; current Connectome workbench contract and native SDK pin; child CI confirmed below. | Installed estate creation and UI project management exist in bounded profiles. rrflowDB is the logical estate, rrflowKV its persistence and rrflowQL its query interface, not three servers. Current-engine turnkey integration remains H-06 work. |
| Automatically index project source into graph/vector knowledge | `connectome/src/rrflow/attunement.ts` marks inventory through verify contract-pending. Engine attunement handlers remain absent in the prior code-grounded audit; installation persists the pending job. | Not demonstrated. D-02/D-03/D-04/D-05 plus consumed E/F paths must execute and retain real source/model/readback evidence. Manually seeded vectors do not prove it. |
| Clear progressive UI with current engine | `workspace.tsx` has nested project workspaces and explicitly unavailable connector, replay, learning and cloud controls. `src-tauri/Cargo.toml` still pins the older engine/contract. | Existing navigation/data lenses are real source, but the current engine/SDK/install adapter must be qualified together. No live desktop was driven in this audit. |
| Effort and AI inheritance | `engine/reasoning_budget.rs`, contract and socket evidence in G-02a; `lfg-swag.md` exposes task configure/read/execute/Stop/attempt-read. | Bounded task embedding and durable Stop exist; complete AI/global/project preference inheritance, governed reasoning and consuming Connectome controls are not proven. FeatureHash is a CPU reference baseline, not an LLM. |
| Parquet | `engine/query_export.rs::export_query_scoped`, `rrd-contract/src/query_export.rs`, installed lifecycle `verify_parquet_export`, D-06 export receipt. | Real bounded export exists: 1024 rows, 64 fields plus identity, 256 KiB input and 512 KiB artifact ceilings. Full import/streaming exchange and dedicated UI control remain D/F/H work. |
| rroflow.ai integration | Existing service/learning contracts and workspace unavailable controls; source search for rroflow/rrflow.ai in Rust crates returned no matches. | No integrated learning/account/transfer demonstration identified. A string search alone is not a complete absence proof; real authenticated-service evidence remains H-07/G/I acceptance. |
| Pinecone/SurrealDB/Qdrant capabilities folded in | Official docs and RRFlow E/F/G/H requirements compared at the existing rationale. | Selected behaviors are required; no complete feature parity, competitive performance or qualified external connector follows from these plans. |

### Independent checks of prior evidence

Read-only GitHub checks on 2026-10-25 confirmed:

- RRFlow PR38 (bounded Parquet export) is merged as
  `b5772374c1ea8ab388fa0c4ac13558d2e3125775`.
- RRFlow PR103 (public task effort) is merged as
  `dd225072bcc639c89b0eef968625cc73303ea998`.
- Both revisions are ancestors of this audit baseline (`git merge-base
  --is-ancestor` returned success for each). Ancestry establishes inclusion,
  not current runtime correctness.
- [Connectome candidate run 40055042635](https://github.com/rrflow/connectome-development/actions/runs/40055042635)
  is completed/success at `0aca893cd5e52ca596483645de4972a61bb17ca0`.
  The existing [pin receipt](h-06-project-workspaces-pin.md) maps its tested tree
  to the pinned merge. This is prior exact-pair evidence, not current engine/UI
  qualification or a new audit execution.

The [Parquet receipt](../gate-d/d-06-bundled-parquet-export.md) and
[public effort receipt](../gate-g/g-02a-public-task-effort.md) record their
actual commands and bounded results. These tests were inspected, not rerun here.
No all-green current-product inference is made from merged source or prior CI.

## Owning-record corrections

The existing Connectome contract now defines minimum input (project source,
reviewed defaults/effects and only necessary missing decisions), automatic
estate/schema/model-bound collection provisioning, and the nested basic-to-
advanced views. Installation, indexing, retrieval and learning must have separate
truthful readiness evidence. The first demonstration must begin with source,
not manual data seeding, and include Stop, edit/delete, restart and isolation.

The LFG/SWAG owner defines explicit, typed opt-in caller effort inheritance,
field origins, fallback and hard-policy clamping; source text and hidden model
state cannot become configuration. The cloud owner separates account binding,
selected dataset transfer, training, evaluated artifact receipt and promotion.
Local indexing remains account-independent. Knowledge-path and Parquet owners
distinguish native behavior, external adapters and interchange capabilities.
Gate D/H point to these requirements without adding another lifecycle authority.

## Research and adaptation

Official sources reviewed online: Pinecone index creation, current integrated
embedding and namespace isolation; SurrealDB RELATE, indexes and transactions;
Qdrant hybrid search and multitenancy; Apache Parquet overview. Exact links and
mapping to canonical gates are in the existing
[capability comparison](../../../architecture/engine-scale-and-contracts.md#turnkey-capability-comparison-and-proof-obligations).
The proposed comparison is a scoped engineering interpretation, not adoption of
vendor storage architecture, automatic support for every vendor feature or a
performance claim. No upstream code, dependency or model asset was copied.

Primary documentation distinguishes file formats, databases and hosted services;
RRFlow must do the same. A vendor namespace/collection strategy does not grant
project sharing. An integrated embedding service does not remove the need to
bind model/dimension/source identity. An external source adapter cannot become
a second RRFlow persistence or transaction owner.

## Coverage, failures and verification

Source review: `query_export.rs` contract constants and relevant
engine export implementation; complete phase definitions in pinned Connectome
`attunement.ts`; targeted workspace labels, native Cargo pins, effort handler/
test declarations, installed export helper, and the referenced prior journals.
Documentation review: relevant Connectome native/journey sections; LFG/SWAG
configuration, effort and public task operations; knowledge-path/export limits;
service roles and learning boundaries. All changed passages and diff reviewed.
This is a targeted source/evidence audit, not a full code audit or live UI test.

`ATTUNEMENT_ESTIMATE_MINUTES` still contains a static 30-45 minute estimate in
Connectome source. No consumption was found in the searched workspace screens;
this audit does not claim that estimate is currently displayed. Future UI must
use qualified engine evidence or show unknown, not promise an unmeasured time.

A guessed SurrealDB transaction URL was inaccessible; the official RELATE
navigation resolved the correct language-primitives transaction page. Rust
source search with no rroflow matches returned status 1 (no matches), not a
runtime service failure. Some oversized read output was truncated; bounded
source and changed-diff reads supplied the relevant evidence. No errors are
converted to passes.

Runtime builds/tests, live desktop, account creation, provider activation,
service calls and new soak campaigns were not performed. The owner's planning
hold and unresolved API admission remain in place. Documentation checks and
baseline scope checks are the acceptance for these edits, not turnkey proof.

Checklist: user outcome and PR scope bound before edits; code/previous evidence
and external sources inspected with limits; each behavior mapped to existing
owners; missing capabilities explicit; no runtime/API/threshold change; old
journals preserved; one new evidence record and generated index/inventory;
exact local results appended below. No objective or gate is closed. Stop before
runtime work, implicit AI configuration, invented service availability or parity
claims. PR137 remains draft; no new candidate-CI or merge pass claimed.

Local acceptance results:

- `python -X utf8 scripts/ci/check_documentation.py`: passed, 611 document
  statuses, 609 coordinates, ownership/local links/indexes valid.
- `python -X utf8 scripts/knowledge/render_navigation.py --check`: passed,
  17 indexes, 610 nodes, 2433 edges, zero drift.
- `python -X utf8 scripts/ci/build_execution_inventory.py --check`: passed,
  1733 current/generated/planned records.
- `python -X utf8 scripts/ci/check_workflow.py`: passed, 3 workflows,
  9 substantive jobs, 5 engine suites, 20 default/6 optional packages, 19 pins.
- `git diff --check` and exact baseline scope/checkbox comparison: passed.
  No runtime, API, scripts, version, Connectome pin or objective status changed.

Inventory regenerated after this result entry. These checks validate the audit
records and ownership chain; they do not demonstrate the missing turnkey flow.

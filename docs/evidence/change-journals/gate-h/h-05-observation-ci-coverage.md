# H-05: observation candidate CI corrections

**Status:** active immutable correction evidence; candidate acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-05-observation-ci-coverage`
**Owner:** H-05 observation delivery prerequisite and candidate CI package coverage

## Bound correction

[PR235](https://github.com/rrflow/rrflow-development/pull/235), driven by the SWAG
controller working session, published observation delivery at
`19c3eb67b2f1125156ef031adf0b34d138a83117`, tree
`cfdf721c8755c22d03bda87de5449096f22c3e46`. Its
[implementation journal](h-05-engine-observation-delivery.md) retains runtime,
fault, installed-binary and author-review evidence. This record corrects CI
coverage, generated storage-test identities and an existing recovery-test clock
assumption. Production behavior and original binary evidence are unchanged.

Candidate [run 41262802879](https://github.com/rrflow/rrflow-development/actions/runs/41262802879)
failed Linux verify job `124719330131` at `python3 scripts/ci/check_workflow.py`:
`engine suite coverage mismatch: missing=['rrd-observation'] unexpected=[]`.
The native platform was the Linux verify runner. The raw completed-job log is
retained as `candidate-41262802879-verify-124719330131.log` under
`D:/projects/swag/.swag/evidence/20261031/observation-delivery/`, SHA-256
`30861737a535ae620ed8aab8e2a4f871d304200873fde1ad33f669d105887b56`.
The CLI initially refused log access while sibling jobs were running; the
completed-job API supplied the retained log. No job was declared terminal from
that observation failure. Local execution reproduced the same policy failure.

Before editing, the PR brief recorded this failure, exact correction scope and
acceptance oracle. `.github/workflows/ci-reusable.yml` now includes the bundled
adapter in the existing `product-operations` package selection. That suite runs
locked all-target tests and strict Clippy. No policy assertion, runner, timeout,
action pin, feature lane, permission, public contract or production source changes.
The missing package is included once; none is removed from another suite.

## Review and checks

The full 804-line reusable workflow was read, including every suite and the
always-evaluated gate. The CI operations owner, change-authoring routine and
`check_workflow.py::verify_engine_suites` supplied the canonical boundary.
The complete workflow delta, test-clock delta and this correction record were
reviewed. Storage fixtures are compared as complete JSON values and refreshed
by their owning generator; no workload/result values are authored. No external
research, source adaptation, new runtime trace or resource measurement was needed
for an explicit package-selection omission.

`python scripts/ci/check_workflow.py` now passes: five workflows, nine substantive
jobs, five cohesive suites, all 22 default-feature packages, six optional-feature
packages and 19 development-container pins. The existing policy regression scripts
also pass: `test_candidate_admission.py` (11), `test_native_sample_policy.py` (6),
`test_sdk_attunement_policy.py` (3), `test_rrflowdb_standalone.py` (7) and
`test_cache_warm_policy.py` (3), all under `scripts/ci/`. The 30 checks preserve the
existing admission and native/SDK policy requirements. The combined log
`ci-coverage-corrected.log` has SHA-256
`b36b90b67700ae62bcc4bb08d98d240c82a6c09ecf2c64d1c9e52ac0bc780aa2`.

## Storage projection and recovery-clock failures

The same candidate's Linux product-operations job `124719330087` failed the
storage evaluator's fixture validation and golden checks because the public
catalogue digest changed with the three observation operations. Raw log
`candidate-41262802879-product-124719330087.log`, SHA-256
`71c6cfc8d8a00e27e8159b83fba6ed9a6b9d94d284348b72e9ff5138b5c45cc0`,
is retained beside the verify log. Before regeneration, the PR brief bound the
correction to canonical projections and unchanged synthetic evidence class.
The complete `crates/evaluation/rrflow-eval/examples/refresh_storage_contract.rs`
was read; it refuses non-synthetic fixture manifests and generates schemas and
linked fixture identities from their owner.

`cargo run -p rrflow-eval --example refresh_storage_contract --locked` completed.
Its only file changes are `tests/fixtures/storage/{manifest,result,report}.json`
inside `rrflow-eval`. Complete JSON comparison confirms only four identity fields
changed: manifest public-catalogue digest, result manifest digest, and report
manifest/result digests. Workload values, outcomes and synthetic classification
are unchanged; all generated schemas remain byte-identical. The first comparison
script mistakenly allowlisted `candidate_sha256` instead of the report's
`result_sha256`; correcting that local comparison assertion verifies the actual
four-field delta without changing generator output or product validation.
These are pre-existing protocol fixtures, never model evaluation/training data
or native qualification results.

`cargo test -p rrflow-eval --all-targets --locked` passes 57 tests with none
failed or ignored. Log `evaluation-projections-corrected.log`, SHA-256
`235a48d4e07bb18610e052ca9e4be29d63514ca2c50ec32b42a24d5859802ef1`.
Strict all-target Clippy passes; `evaluation-projections-clippy.log` SHA-256
`9afabd88dbcd76025ab67bf99af0ad4945972d94aa000330535ec834c52eb8b0`.

The Linux engine-authority job `124719330184` also failed two existing interrupted
inventory tests at `partial_inventory_recovery`: fixed request times relative to
reopen could predate recovery's actual committed completion. The production
checks correctly returned `attunement command predates the job snapshot` and
`attunement transition time must advance`. Raw log
`candidate-41262802879-authority-124719330184.log`, SHA-256
`9fccbaa8b84fe04da87e8a5f726544acf41bb345af8da56b47b85763e450c443`,
is retained. Its suite reported 300 passes, two failures and four ignored helpers.

The PR brief recorded the additional test-only correction before editing.
The complete `partial_inventory_recovery` and its `call`/`resume` helpers were
reviewed with engine control/replan elapsed-time handling. Subsequent test
requests now derive from `recovered.status.job.updated_at_unix_ms`, preserving
all interruption, stale-source, successor, checkpoint and no-duplicate assertions.
No production clock, source rule, admission check or timeout is weakened.
On Windows, `cargo test -p rrd-engine --lib inventory_ --locked` passes all
eight focused tests, including the two failed Linux scenarios. Log
`recovery-clock-focused.log`, SHA-256
`003ff04b3937454f162ba66c1446c898510ad4a49ea5a6b6761ba0203c594abf`.
`cargo test -p rrd-engine --all-targets --locked` then passes 365 tests with
zero failures and four existing subprocess helpers ignored, including all
33 architecture checks. Log `recovery-clock-owning.log`, SHA-256
`011d99208830f0bdf57137159af8a7eee0aab571f0b7408027bd018aa1e19c0a`.
Strict all-target engine Clippy passes, `recovery-clock-clippy.log` SHA-256
`315eb5251090777fb1eb0fd42a7c44ed1e83023f2c8803c97ef2f7ce6efb8865`.
Workspace formatting also passes. Linux confirmation remains candidate CI's
responsibility; Windows success does not clear the original Linux failures.

## Handoff and limits

Checklist: same bounded PR and owner; pre-edit correction brief; reproduced
first failure; exact inclusion repair; existing policy regression checks; full
workflow/diff review; separate immutable journal and generated index. Regenerate
and verify documentation/inventory before committing the correction, then push
the exact source to the public development branch. A corrected full candidate
run is required; passing local policy checks cannot clear the original failed
candidate. The evaluator and affected engine checks cover these test/projection changes;
installed binary evidence remains bound to its original executable.

No roadmap, alpha, POA&M or release status changes. Actual development-task/UI
correlation, independently reviewed learning and public release remain outside
this correction. Stop if coverage repair would weaken a required gate or require
changing unrelated runtime behavior. The package driver owns the corrected
candidate and ordinary merge controls.

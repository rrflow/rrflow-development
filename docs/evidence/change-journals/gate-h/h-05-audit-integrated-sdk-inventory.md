# Audit read repair: SDK and inventory candidate integration

**Status:** active draft integration; both dependencies and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-05-audit-integrated-sdk-inventory`
**Owner:** H-05 / POAM-030 / [PR144](https://github.com/rrflow/rrflow-development/pull/144)

Starting audit head `75cea1de53da882161fb809ca26b172cb3a0b644` failed its
[candidate verify job](https://github.com/rrflow/rrflow-development/actions/runs/40874396924)
on execution-inventory drift. The PR brief was extended before normally
merging its explicit SDK prerequisite [PR177](https://github.com/rrflow/rrflow-development/pull/177)
candidate `1ab64897bcee1440159d19846e1bf0a6eb3d02b2`. That candidate itself
contains [PR193](https://github.com/rrflow/rrflow-development/pull/193)
`bdb954e48d2eb751b4839ed6efa761cc1e294071` and current landed main. Neither
prerequisite has landed; this PR remains draft pending both landings and its
own exact candidate and protected merge-group CI.

The only merge conflicts were the generated Gate H index and retired tracked
inventory. Retain the retirement descriptor and regenerate combined navigation
and the ignored Git-derived current inventory. The `audit_since` fix continues
to declare a tail only when a short control-journal page was fully consumed;
the existing returned-record limit, ordering, authorization, chain, corruption and
durable-head validation remain. Its prior prefix/page-size, persistent reopen,
engine read/export parity and HTTP tests are retained. The SDK native private
manifest protections, all landed engine/SDK behavior and immutable journals
remain. No public model, permission, threshold or gate status is changed.

Reviewed the README identity, H-05 / POAM-030, previous audit/SDK integration
journals, actual audit diff, security tests, real HTTP fixture and generated
conflicts. The complete source diff against the SDK candidate was inspected;
no upstream implementation is adapted. Existing chain digests and independent
prefix comparisons remain the failure and trace oracle; no benchmark or
memory-effect receipt is implied by this page-tail correction.

On Windows, the SDK private-manifest suite passed four native cases with the
POSIX case skipped; `python scripts/ci/test_execution_inventory.py` passed ten,
`python scripts/ci/test_candidate_admission.py` passed 11,
`python scripts/ci/check_workflow.py` passed, and `git diff --cached --check`
passed. Fresh isolated Rust checks are recorded below. Earlier green
candidate results cannot be transferred to this integrated head. No H-05,
POA&M, C-07 or alpha completion is claimed.

## Stop condition and additional finding

Review of `crates/authority/rrd-security/src/lib.rs::audit_since`,
`crates/persistence/rrd-store/src/repository/control.rs::journal_since` and
`crates/transport/rrd-contract/src/lib.rs::AuditPage` found that the record
limit bounds each control fetch and the number of returned audit records, but
not the number of fetches when non-audit control records intervene. The public
page has a sequence cursor but no explicit completion flag. The pre-existing
unbounded-paging stop condition is active; this PR remains draft. Updated
POAM-030 with the limitation and left its status unchanged. No runtime
measurement of this deficiency or implementation of a scan budget is claimed.
A separate scoped contract and implementation must preserve continuation,
chain validation and read/export behavior with sparse audit records.

## Isolated Windows test evidence

Starting revision `75cea1de53da882161fb809ca26b172cb3a0b644`, tree
`1a41b9a731ca7061cb74c22c51a137e130a1b3ce`; staged source and initial
journal tree at test start `4b53bd44a58d936490ab32d9fa9fed7d1f3e8e3b`.
The subsequent changes are this evidence record and the POAM source-review
addition; no tested Rust source changed. `CARGO_TARGET_DIR` was isolated at
`target/audit-144-sdk193-build`, with offline locked dependencies and two build
jobs. Commands and outcomes:

- `cargo test --offline --locked -p rrd-security --test security_authority`:
  7 passed.
- `cargo test --offline --locked -p rrd-engine --lib engine::tests::security`:
  11 passed, 234 filtered out.
- `cargo test --offline --locked -p rrd-server --test http_process initialized_security_authority_binds_sessions_and_denies_ungranted_routes -- --exact --nocapture`:
  1 passed, 19 filtered out.
- The same HTTP target with
  `authenticated_storage_diagnostic_trace_correlates_store_without_secrets`:
  1 passed, 19 filtered out.

Local retained logs under `target/audit-144-sdk193-` have SHA-256:
- `security.log`: `2cd9b120c53be495c66d588dd131d3df164bd91e8cc7a974c866d1531e6ab988`.
- `engine-security.log`: `353250bcd448f8309a6da3bd590d90f72f0e3274f5b55eabae31527405c9c157`.
- `http-audit.log`: `d8de3d009970e102e864ff50bedd3d4bb10318f0d9fa6a690a31f06a75e64307`.
- `http-diagnostics.log`: `4adffd9b502cb376cddce30f55ca1ee5e3ba946b3af669c827ad5db6c4642873`.

These tests do not exercise an adversarial sparse journal or establish a
per-call scan budget. The complete server/workspace suite, candidate CI and
protected merge-group CI have not run on this integrated tree. Dependencies
and the scan-bound stop remain open. The full source diff relative to PR177
was inspected, and no roadmap or POAM lifecycle status changed.

## Review checklist and remaining checks

- `cargo clippy --offline --locked -p rrd-security -p rrd-engine -p rrd-server --lib --tests -- -D warnings`: passed; retained `target/audit-144-sdk193-clippy.log` SHA-256 `1d9168cd6b9b9b65c1d04c0dc0d38e12689b35cf002df1c59c2faa97a264d5c3`.
- `cargo fmt --all -- --check`, documentation policy, navigation check,
  inventory generation/check and `git diff --check`: passed locally.
- Read the current README and H-05/POAM-030, the affected audit function and
  security tests, engine audit read/export and tests, the HTTP fixture and
  control-journal paging, the SDK integration journal and the PR's complete
  source diff against its dependency. No source adaptation is involved;
  chain digests and known prefixes are the diagnostic oracle.
- Reviewed the dependency merge conflict resolutions and staged changes.
  The independently unbounded scan activated the PR's stop condition; no
  source or contract fix for it is included. The POAM detail records the
  finding without closing or reprioritizing an owner status.
- Candidate and merge-group checks are not run for this head; PR193 and PR177
  still need their own landing evidence. No installed multi-platform, C-07,
  actual soak, memory-effects, source-inventory or alpha claim is made.

The driver is the current Codex session; this draft is held for the separate
scan-bound design and its named dependencies.

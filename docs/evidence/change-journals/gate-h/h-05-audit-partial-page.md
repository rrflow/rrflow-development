# H-05: preserve valid audit prefixes in partially consumed journal pages

**Status:** active implementation evidence; candidate CI and full H-05 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-05-audit-partial-page`
**Owner:** OBJ-10 / H-05 / POAM-030 / PR144

## Brief and implementation

Public development main `04fae27665c3081ef6bc193d54d8c1c9c8d3f2e0`, tree
`0effb6f859ddd0a99e24efd6016f80e06405cd40`, is the starting baseline. Empty
opening commit `3a9f9caf5e3be95eead62ad2641746254ba5c359` opened
[PR144](https://github.com/rrflow/rrflow-development/pull/144), whose brief
declared the scope, failure oracle, trace and stop conditions before edits.

`SecurityRepository::audit_since` previously marked a short final control
page as the journal tail even when the requested audit count stopped processing
inside that page. It now records how many control entries it actually consumed
and recognizes that tail only after consuming the whole short page, or after an
empty page. All existing chain-head, continuity, authorization and size checks
remain. No public schema, API, storage format, source movement or dependency
changes. The existing authentication/audit repository remains the authority.

Changed code: `crates/authority/rrd-security/src/lib.rs::audit_since`; its
existing `tests/security_authority.rs` characterization now requires valid
prefixes and covers every page size from 1 to 14 across memory and persistent
stores, including persistent reopen. `crates/authority/rrd-engine/src/engine/
tests/security.rs` compares engine audit read and export over the same
committed prefix for page sizes 1 through 14 while authorization can append
later audit entries. It verifies records, page boundaries and chain digests.
This is audit-record evidence, not complete memory-creation activity UX.

## First oracle and checks

On Windows, after inverting the existing characterization but before changing
the implementation, `cargo test --offline --locked -p rrd-security --test
security_authority a_record_limit_inside_the_final_partial_page_returns_a_valid_prefix`
failed with `Invalid("audit chain does not reach its durable head")`. The
same test passed after the fix, including the expanded page-size/reopen cases.

`cargo test --offline --locked -p rrd-security` passed seven tests, including
substituted durable-head rejection. `cargo test --offline --locked -p
rrd-engine --lib engine::tests::security::` passed eleven tests, including
read/export parity and the existing test that still characterizes POAM-029's
unrelated diagnostic conflict as a failure. These are local component results,
not installed UI or candidate-CI qualification. `cargo clippy --offline --locked
-p rrd-security -p rrd-engine --all-targets -- -D warnings` passed. Commands
used the existing isolated Windows build target directory. Exact candidate CI
remains required before merge.

`python scripts/knowledge/render_navigation.py` and its `--check` passed
(17 indexes, 637 nodes, 2510 edges). `python scripts/ci/build_execution_inventory.py`
and `--check` passed (1768 path records). `python scripts/ci/check_documentation.py`,
`python scripts/ci/check_workflow.py`, `cargo fmt --all -- --check` and
`git diff --check` passed. Inventory is refreshed after this journal update.

## Review, research and limits

Reviewed the README bootstrap; H-05, POAM-030 and change-authoring/CI owners;
the audit read implementation, existing fixture and head-corruption test;
engine read/export authorization and serialization; and the new complete test
diff. Existing control journal sequence and chain digests supply the oracle;
no external algorithm, upstream code, dependency or new trace authority is
introduced. The first attempted test patch failed because its context did not
match the existing comment; the corrected patch applied before the failing run.
No correctness, resource or security check was relaxed.

Not yet run: exact-candidate CI and installed-client proof. POAM-030 remains
open until its owning exit and candidate evidence are reconciled; POAM-029 and
H-05 remain open independently. This journal does not change roadmap status.
Checklist: PR brief precedes implementation; canonical owner and scope bound;
baseline failure reproduced; direct correction and multi-size/reopen oracles
added; full diff, owning checks, generated navigation and exact-candidate CI
required before merge. No merge or release claim is made.

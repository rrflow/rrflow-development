# D-04 parser draft: reconcile the public contract with main

**Status:** active draft integration; installed Parse and candidate CI remain blocked
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-04-parser-main-contract-reconciliation`
**Owner:** Gate D source parsing, POAM-006 and [PR #241](https://github.com/rrflow/rrflow-development/pull/241)

The starting draft was `10864770a9a62d5fcdc4826e704654080eeacdf7`; public main was `3a72a6b4adc43c842f67a59f4fc91d725eac7580`. The PR brief was extended before this ordinary main merge. The existing [parser journal](d-04-source-parsing-in-progress.md) records the earlier implementation and its unresolved installed-process failure. This reconciliation changes no parser execution or lifecycle authority.

The read-only conflict inspector identified 19 conflicting paths: the contract owner, six reference documents and 12 declared generated projections. Each of the seven non-generated conflicts was an OpenAPI digest citation. Main also added observation delivery to the same contract source while the draft added parse-policy exports; the merged source retains both. The complete combined export contains the three observation prepare/deliver/read routes and 15 `parse_policy` occurrences. Its OpenAPI SHA-256 is `ab55887653d7499ced47f6be542b2db17baa3321ca0b3ce6846796d06e77efaf`. Seven source/reference citations were updated to this computed value; all 12 conflicting projections were regenerated from the combined owner. These checks establish projection consistency, not parser safety or correctness.

For this reconciliation, reviewed the PR #241 brief and existing D-04 journal; `crates/transport/rrd-contract/src/lib.rs` from both parents; the conflicted protocol and SDK references; `sdks/python/scripts/generate.py`, `sdks/typescript/package.json` and `scripts/generate.ts`, `sdks/go/cmd/generate/main.go`, `scripts/ci/check_generated_surfaces.py`, `scripts/ci/inspect_merge_conflicts.py`, and the generated-projection registry in `scripts/ci/build_execution_inventory.py`. The original package journal records the earlier code review and source references. No new upstream algorithm was adopted during this merge; Tree-sitter and native isolation research and stop conditions remain as documented in the PR. Trace, resource limits and failure semantics are unchanged.

On Windows, the following completed successfully on the reconciled tree:

- `cargo run --offline --locked -q -p rrd-contract --bin rrd-contract-export`, followed by SHA-256, route and policy-occurrence checks.
- `python sdks/python/scripts/generate.py`, `python sdks/java/scripts/generate.py`, `python sdks/dotnet/scripts/generate.py`, `go run ./cmd/generate` in `sdks/go`, `pnpm generate` in `sdks/typescript`, and `python scripts/ci/check_generated_surfaces.py --write`. The declared Python generator formatter `ruff==0.17.1` was installed from local cache into an ignored build-tools directory.
- `python scripts/ci/check_generated_surfaces.py --check` verified 74 HTTP operations and seven signal projections; each SDK generator's corresponding check also passed.
- `cargo test --offline --locked -p rrd-contract --test public_contract` passed 46 tests; `cargo test --offline --locked -p rrd-contract --all-targets` passed; `cargo test --offline --locked -p rrd-attunement` passed 49 tests.
- After correcting the status noted below, `python scripts/knowledge/render_navigation.py`, `python scripts/ci/check_documentation.py`, and `python scripts/knowledge/render_navigation.py --check` passed.

The initial documentation-policy check rejected this journal because its status did not contain the required `active` or `historical` classification; the status was corrected before the final check. The native parser supervisor, platform memory and cancellation enforcement, authorized publication, restart/replay, and the real installed Parse oracle remain incomplete. This merge does not satisfy those requirements, C-07 native capacity/soak, or developer-alpha acceptance. Candidate and merge-group CI, cross-platform tests, and full installed process tests have not been run for this candidate. No roadmap or POAM completion status changes.

Checklist: PR brief updated before changes; parent revisions recorded; conflicts inspected and owner sources reconciled; projections regenerated and checked; relevant contract and parser tests run; linked journal added; navigation and final diff checks required before commit. No implementation was deleted, moved, or rewritten by this reconciliation.

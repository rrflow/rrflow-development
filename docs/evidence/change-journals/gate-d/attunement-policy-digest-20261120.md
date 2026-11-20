# Bind automatic attunement policy into configuration identity

**Status:** active repair evidence; candidate and merge-queue checks pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/attunement-policy-digest-20261120`
**Owner:** OBJ-14 / [Gate D](../../../roadmap/rrflow-1.0/gate-d.md) / [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md)

## Brief and source map

[PR447](https://github.com/rrflow/rrflow-development/pull/447) was opened with
the repair brief before implementation. Baseline main:
`adee61f32a6c2f87157043ed1d7d679685e60036`. Driver: the owner's ongoing
historical-code recovery session. The defect was found while comparing the
recovered installation-clock/configuration work with the current implementation;
this is not a verdict that all those stashes have been absorbed.

`EngramConfiguration::with_attunement_follow` recomputed a digest that omitted
the policy it had just changed. Disabled, enabled and differently timed automatic
attunement therefore shared a configuration identity. `validate()` also failed
to detect edits to that field when its old digest was retained. The installation
plan has its own broader digest; this finding does not claim that all plan or
authorization checks were bypassed.

The canonical `engram_configuration_sha256` now includes `attunement_follow`,
including `None`, with the other configuration fields. No alternate hash,
compatibility reader, migration or version change is introduced. This changes
the configuration identity even for disabled follow policies. Existing installed
state sealed with the old digest is not silently accepted by the repaired
validator. No user's installed state is changed or deleted by this package.

| Boundary | Source and proof |
|---|---|
| One configuration identity | Complete `crates/transport/rrd-contract/src/deployment/configuration.rs`; the repair adds the omitted field to the existing digest tuple. |
| Mutation rejection and canonical reconstruction | `tests/deployment_configuration_contract.rs::automatic_attunement_policy_is_bound_to_configuration_identity` compares disabled, 1,000 ms and 2,000 ms policies, rejects all six unsealed transitions and verifies disabling through the builder returns the original configuration. |
| Installation consumer | `rrd-engine/src/engine/installation/planning.rs` lines 78-112 applies the explicit follow choice through the same builder; `installation/open.rs` validates the persisted configuration and binds its digest to the locator. |
| Runtime consumer | `rrd-server/src/http/server.rs` lines 264-307 starts the bounded background follower only for an enabled policy; `rrd-engine/src/engine/attunement/follow.rs` lines 207-240 refuses disabled follow. No second policy owner is added. |
| Joined behavior oracle | Existing `a_followed_project_is_attuned_and_relearned_by_the_engine`: install with the policy, derive source facts, query them, verify an unchanged pass, edit one file, and prove only that file is reparsed. This is an engine test, not a native desktop or whole-project qualification. |
| Derived examples | Existing `refresh_installation_contract` and `refresh_storage_contract` examples regenerate synthetic digests through the typed owners. They do not create workload evidence. |

Read the complete current configuration module and new test, the existing
configuration tests, the consumer ranges above and the existing follower test
at `engine/tests/attunement/parse_step.rs:700`. The prior
[code-knowledge receipt](d-02-code-knowledge-slice-20261112.md) was inspected
at its policy implementation, oracle and verification sections. Owners read:
README, AGENTS, agent session standard, change-authoring procedure, relevant
Gate D and POAM-006 sections. Large owner modules are not claimed fully reread.

## Failure and acceptance evidence

Scratch: `D:/.usr/Temp/rrflow-selective-recall-audit-20261120`.
`CARGO_TARGET_DIR` is its existing `target` directory and
`CARGO_BUILD_JOBS=4`. No new dependency, third-party adaptation or external
research is needed for this omitted local field. Retained test output supplies
the debugging trace; runtime instrumentation is unchanged.

Before the repair, the new regression failed with exit 101: disabled and enabled
policies both hashed to
`67ac500465419a8ee9fee6f4ad99fb77d7f438d914d615c84cf935d67d1f1a09`.
`attunement-policy-before.log` retains the exact failure. After adding the field,
all ten deployment-configuration tests passed (`attunement-policy-contract.log`).

The first complete contract run then failed its installation golden comparison.
The existing `cargo run --locked -p rrd-contract --example refresh_installation_contract`
updated only configuration, installation-plan and installation-result hashes
in `fixtures/install-attunement-v3.json` and `fixtures/public-contract-v1.json`.
The complete fixture diff was read; no expected behavior or assertion was
removed. The next `cargo test --locked -p rrd-contract` passed all 205 tests,
zero failed or ignored (`attunement-policy-contract-suite-after.log`). The
original golden failure remains in `attunement-policy-contract-suite.log`.

The first engine installation run passed 83 tests but failed
`shared_sdk_corpus_identity_matches_bundled_installation`: the typed planner's
derived instance ID changed with the configuration digest. The existing
`refresh_sdk_installation_identity` generator then refused its own incomplete
update with `SDK cold-history scope differs from the corpus instance`. It
omitted that scope and five vector-index scopes. Before editing the generator,
the PR brief was extended to name this projection prerequisite.

The same generator now updates those six scopes in its typed corpus and its
existing JSON projection. Its strict validator remains intact. The resulting
corpus diff changes only two identity fields and thirteen instance scopes;
authored query, vector and expected-result content is unchanged. Both generator
execution and `--check` pass. Logs: `attunement-policy-sdk-refresh.log` for the
failure, and `attunement-policy-sdk-refresh-after.log` / `attunement-policy-sdk-check.log`
for the repair. The complete generator was read before editing.

`cargo run --locked -p rrflow-qualification-storage --example refresh_storage_contract`
regenerated only the configuration/manifest/result digest values in the three
synthetic storage fixtures. Schema output was unchanged. The complete diff was
read. `cargo test --locked -p rrflow-qualification-storage` passed all 54 tests
with zero failures or ignored cases (`attunement-policy-storage-suite.log`).

`cargo test --locked -p rrd-engine --lib installation` then passed 84 tests;
one subprocess helper is intentionally selected by its parent tests rather
than run directly. The failed and passing logs are
`attunement-policy-installation.log` and `attunement-policy-installation-after.log`.
`cargo test --locked -p rrd-engine --lib a_followed_project_is_attuned_and_relearned_by_the_engine`
passed the joined engine oracle in 0.54 seconds (`attunement-policy-follow.log`).
It verifies source-derived facts and selective edit handling, not a native UI.

`cargo clippy --locked -p rrd-contract --all-targets -- -D warnings` and
`cargo clippy --locked -p rrd-engine --example refresh_sdk_installation_identity -- -D warnings`
both passed. Their logs are `attunement-policy-contract-clippy.log` and
`attunement-policy-generator-clippy.log`.

`cargo test --locked -p rrflow-cli --test installed_lifecycle installed_engine_plans_applies_serves_discovers_authenticates_and_reopens`
passed the real CLI lifecycle on Windows in 18.84 seconds: plan, apply, serve,
discover, authenticate and reopen. The log is
`attunement-policy-cli-lifecycle.log`. This does not qualify the Connectome UI.

`python scripts/ci/check_generated_surfaces.py --check` passed
(`attunement-policy-surfaces.log`). `cargo fmt --all`, navigation rendering,
execution inventory generation, `python scripts/ci/check_documentation.py`
and `git diff --check` completed successfully; documentation checked 1,284
document statuses and 1,280 classified coordinates. No six-language SDK run
or manual desktop comparison is claimed by these local checks.

## Checklist and limits

This package replaces the defective identity calculation at its existing owner.
Its PR brief binds outcome, baseline, exact scope, production consumers, first
failure, acceptance commands, research/resource/trace decisions and stop
conditions. Any unexplained digest change, authorization difference or failed
oracle blocks readiness. Changed source and complete fixture diffs are reviewed;
the linked journal and nearest index are regenerated before commit.

No evidence establishes native Linux/macOS behavior, competitor performance,
full UI parity or complete historical recovery. POAM-006 and the broader
attunement objective remain open. The next integration step is the exact
candidate and merge-queue gate, followed by continuing the recovered-source
reconciliation. No unrelated worktree, process or installed project is removed.

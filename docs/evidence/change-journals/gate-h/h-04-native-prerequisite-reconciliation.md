# Native client reconciliation with current engine prerequisites

**Status:** active local integration evidence; dependency and candidate landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-native-prerequisite-reconciliation`
**Owner:** H-04 / POAM-011 native client; G-01 proposal semantics; PR222; SWAG controller session

## Source and change checklist

The pre-edit [PR222 brief](https://github.com/rrflow/rrflow-development/pull/222)
binds ordinary reconciliation from native client
`c376e27126c0fd513f8ffeb1e8e17a8befe87602` with controller dependency
`780c1fba60052ee0a93c4eed9f1bd8de89819af4`. Merge
`f957e06d44b77fb0130f0ad017df1d3307d97449`, tree
`d3a762d2ed9bc85f17984b469818bba9445dd0c3`, retains both parents without
conflicts and includes current main `906bec82` plus startup candidate `974c9d65`.
PR224 and PR228 have landed. PR227's full candidate run is active; PR206 is
still draft. No dependency is declared landed by this composition.

The incoming dependency supplies its existing Git-aware Inventory, bounded
publication/checkpoints, source-replan/startup binding, installed CLI/SDK cases,
storage repair, locked dependencies and owner evidence. It also removes the
obsolete public phase-order constant/sequence helper through its existing
contract change; the native client does not recreate a phase-order authority.
Incoming source and tests stay at their canonical owners, rather than being
rewritten or claimed freshly qualified in their entirety here.

Byte-exact parent comparisons pass for the complete native client and proposal
identity validator against `c376e271`, and for engine, persistence, compute,
CLI, SDKs, Cargo manifest and lockfile against `780c1fba`. The composed workflow
retains the native generator test plus incoming attunement and two-case Python
SDK checks. The generated marker metadata, sealed proposal validator and one-send
transport remain unchanged. Parent comparison receipt SHA-256 is
`6dab42f4b596c34bd74cb68ff4dd2e67127cf6a08572619f38a721bc271479f6`.

Reviewed README/AGENTS ownership, the prior native integration journal,
H-04/POAM-011 scope, incoming controller/startup journals, both parent change
lists and retained call/transport/generator/contract/workflow diffs. Focused
review covered call/transport validation, generator and its tests, proposal
identity, imported contract export removal and composed CI. Existing full-file
reviews remain attributed to the immutable owner journals. No unrelated source
is authored, moved or deleted by this increment; it adds this record and
regenerates the Gate H index after the ordinary merge.

## Fresh composed validation

All commands below passed at clean merge source `f957e06d` on Windows, with
two Cargo jobs and dev/test debug information disabled. The existing compiler
cache is `D:/localdev/devlabs/rrflow/target/controller-policy-configuration-worktree/target`;
that output directory is not a sibling source dependency or shipped artifact.

| Command | Observed result |
|---|---|
| `cargo test --locked -p rrd-client --test generated_catalogue --test controller_proposal` | Nine passed |
| `cargo test --locked -p rrd-client -p rrd-contract --all-features` | 182 passed across 31 targets, none failed or ignored |
| `cargo clippy --locked -p rrd-client -p rrd-contract --all-targets --all-features -- -D warnings` | Passed |
| `cargo test --locked -p rrd-engine --test workspace_architecture --test direct_read_source_closure` | Both targets passed, including all 32 architecture checks |
| `cargo fmt --all -- --check` | Passed |
| `python scripts/ci/check_generated_surfaces.py` | 71 HTTP operations and seven signal projections; unchanged OpenAPI digest `f1856602eb3b5ba419f600b3be0e9626d62ad533032fe220a2728795bd28048a` |
| `python scripts/ci/check_workflow.py` | Passed |

The client suite includes actual in-process engine/server authorization,
installation/replan/reopen, retained-response validation, lost-response single
send and unsupported-operation compile denial. Test inputs remain correctness
fixtures, not new provider observations or training records. No local validation
failure occurred in these commands. Raw logs, command/source/exit/hash receipts,
parent preservation and the pre-edit brief are retained under
`D:/projects/swag/.swag/evidence/20261031/native-client-prerequisite-reconciliation/`.
Owner-suite log SHA-256:
`7f2b42f794eda283fed9e07df8f3cce34675b3fa739101373c90cbb6ad1aa5f7`.
Strict-lint log SHA-256:
`a0ebe6c2d6a7f547b6a559b543ef50b635283addd1be78cac9c9ac67bacf0c4e`.

Final repository checks pass: `python scripts/knowledge/render_navigation.py`
reports 850 nodes and 3,080 edges; regenerated execution inventory passes its
`--check` with 2,188 records. `python scripts/ci/check_documentation.py` reports
851 statuses and 849 classified coordinates. `git diff --check` passes; final
review contains only this new journal and its generated index beyond the
declared ordinary merge. AGENTS.md remains unchanged.

## Limits and next acceptance

No new research, upstream algorithm, production trace, resource measurement,
provider call, label or consent is needed or inferred for this reconciliation.
Stop conditions remain changed admission/retry behavior, missing semantic
validation, duplicate authority or an unexpected functional repair; none was
observed. Prior source-specific failures and receipts remain unchanged.

Imported engine suites keep their original attribution; they were not all rerun
locally solely for this client composition. No new installed executable, UI pin,
model-quality result, container or public release is claimed. Earlier live
client/engine trials remain bound to their named pairs. Full project effects,
uncertain-work reconciliation, RROFLOW and reviewed learning remain open.

Keep PR222 draft until PR206 actually lands, then reconcile that destination
and mark ready to start the complete candidate pipeline. Candidate and protected
acceptance are required before landing. No H-04, roadmap or POA&M status closes;
product version remains `1.0.0`.

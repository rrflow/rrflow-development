# C-07g: retain the read-admission failure boundary

**Status:** active implementation evidence; native qualification and candidate CI pending  
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-read-denial-evidence`  
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR162](https://github.com/rrflow/rrflow-development/pull/162)

## Brief and baseline

[PR162](https://github.com/rrflow/rrflow-development/pull/162) recorded its bounded brief before edits. The candidate starts at `eb8dbbf25550958dd5196657358f034917111e1c`, tree `2cd18a5671b35b61ac226d075d6daa81f86337b0`, an ordinary merge of the PR158 shared-pin candidate into the new branch. Its dependencies, candidate CI and merge are pending. The driver is this working session.

The retained Windows warm-only run of PR158 `90bb821ee360b4979f61f1e8203efff8569594ad` acknowledged and independently read back 100,000 records and completed two kill/reopens with matching original/replay receipt digests, then failed at 47,016 post-reopen reads with a pinned-byte admission error. No measured segments began. [PR151](https://github.com/rrflow/rrflow-development/pull/151) holds its exact retained evidence and POAM-035 update. The previous harness exposed only a closed classification and hash, and cleaned the owned estate on this failure; its numeric limit, observed value and cause cannot be recovered from that result.

The test harness now emits only the numeric limit and observed value for the exact canonical native pinned-byte admission error grammar, rejecting malformed text and private suffixes. When the lifecycle reopen stage fails, it retains its own private estate path and reports that path, rather than deleting the state needed for diagnosis. Successful lifecycle cleanup and the original error are unchanged. Changes are limited to `crates/adapters/rrflow-cli/tests/support/sustained.rs` and `sustained/workload.rs`; no production engine limit, retry, workload, native evidence or storage authority changed. A retained directory may contain private project state and requires deliberate local handling. No file was moved or deleted.

## Decisions and review

Read README and linked C-07/POAM owners; codebase-grounded authoring and CI procedure; the PR158 pinned-generation diff and tests; the retained Windows run result, progress and terminal logs; complete sustained harness, lifecycle and workload paths at their relevant boundaries; existing owned-directory and client error handling; changed files and full diff. The old result cannot establish a causal fix. No external algorithm or code is adapted. Diagnostic capture is justified by the observed denial and emits validated numbers only; the failed estate is retained under its existing ownership boundary. Do not increase admission ceilings, weaken the oracle, or declare qualification from this instrumentation.

## Oracle and checks

On Windows in the isolated `target/c07-denial-evidence-build` target with four build jobs and test debug information disabled:

- Initial focused compile failed because the planned diagnostic helpers were absent; implementing the helpers resolved it.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen pinned_read_denial_reports_only_exact_numeric_resource_values`: passed, including actual `rrd_store` error wrapped as engine errors and malformed/sensitive suffix rejection.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen lifecycle_failure_retains_its_owned_estate_but_success_cleans_it`: passed.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen`: 17 passed, seven explicitly ignored.
- `cargo test --offline --locked -p rrflow-cli --test installed_reopen sustained::installed_capacity_workload_smoke -- --ignored --exact`: one installed smoke passed before the final parser-only tightening.
- `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings` and `cargo fmt --all -- --check`: passed after that tightening.

The first documentation-policy check failed because the journal status did not declare an active record; the status was corrected and the generators and checks were rerun.

The read grammar and retention tests are local deterministic oracles, not a native 100,000-record result. A new exact integrated candidate run must establish whether this diagnostic is reached and what it reports; a cause and successful recovery remain unknown. No native capacity matrix, complete profile, or 24-hour soak was run by this package. Candidate CI, integration, review and merge remain pending. C-07 and POAM-035 stay open, with no roadmap status change.

Checklist: owner, baseline, effect scope, first oracle, dependencies and stop conditions were bound in the PR; source and retained failure were reviewed; diagnostic, resource and research decisions and concrete checks are recorded here; the complete candidate diff, generated navigation/inventory and candidate CI must pass before landing. No claim of product-level completion is made.

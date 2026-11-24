# Nightly full matrix repair after the 2026-11-22 landings

**Status:** active repair evidence; one failure remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/nightly-matrix-repair-20261124`
**Owner:** [CI and runner operations](../../../operations/ci.md) and [Gate C, C-07](../../../roadmap/rrflow-1.0/gate-c.md); alarm [issue 505](https://github.com/rrflow/rrflow-development/issues/505).

## Change brief

Baseline `ee3d308f29ab3110a851c88d5b538c4073afc01d`. The nightly full matrix passed on `b08542f7e` (2026-11-22) and failed on this revision (run 45212399534, 2026-11-23). Pull requests 496, 497, 498, 500 and 503 landed between the two runs. The failing jobs were `installed 100-cycle reopen` on all three operating systems, `optional feature (rrflow-cli)` and `optional feature (rrd-engine)`. None of these lanes runs on a pull request, so every one of the five candidates was green when it merged.

Scope: the installed conflict oracle's expectation for a repeated commit, the nightly feature selection for `rrflow-cli`, the workflow policy check and the CI reference. No engine, storage or protocol behavior changes. Stop if a repair would weaken a product assertion, hide a failing test or change what a served engine does.

Files read: the failed job logs, `transaction.rs`, `transaction/outcomes.rs`, `engine/error.rs`, `installed/serving.rs`, `rrd-lsm/src/fault_io/crash/installed.rs`, `tests/support/running_engine.rs`, `tests/support/sustained/campaign.rs`, `attunement/linux_supervisor.rs` and its tests, `ci-reusable.yml`, `check_workflow.py`, `database-semantics.md` and `public-contract.md`.

## Findings and changes

**Repeated commit on a refused lease.** PR 496 added `finish_rejected_transaction`, which closes a lease as aborted after a definite refusal. `docs/reference/data/database-semantics.md` owns that rule: a subsequent commit on the lease is closed, and an explicit abort acknowledges it. The installed campaign still required the repeated commit to return `conflict` again and received `failed_precondition`. The first refusal, its `transaction-snapshot-stale` details, the acknowledged abort and the absence of the refused write were already correct and are still asserted. `campaign.rs` now requires the repeated commit to be a non-retryable `failed_precondition`.

**`rrflow-cli` under every feature.** PR 503 gave `rrflow-cli` the `cld-parquet` and `cld-vortex` features, and `check_workflow.py` then required the package in the `--all-features` matrix. That flag also enables `crash-test`, under which `serve` reads one control line from standard input before opening the engine. The ordinary installed suites send nothing, so the engine exited with `invalid diagnostic control` before announcing readiness; thirteen tests failed that way or on the equivalent `installed_reopen` refusal. The lane had never run before it landed. The matrix now runs `rrflow-cli` with `--features cld-parquet,cld-vortex`, and the policy check derives that list from the manifest, so a new non-diagnostic feature fails the check until the lane names it. `crash-test` keeps its dedicated native job.

**`rrd-engine` under every feature: not repaired.** Three Linux supervisor tests start a child from the test executable under a 256 MiB address-space limit and received exit status 139 instead of the expected results. The same tests pass in the default-feature suite, passed in this lane on 2026-11-22, and a sibling test whose first child runs under 512 MiB still passes. The only feature difference since the passing run is the two cold codecs. The likely cause is that an image linked with both codecs cannot start inside 256 MiB of address space. This is an inference from the logs and has not been reproduced: the WSL host has no `libclang` for the Vortex build and the session was not permitted to complete the build with a downloaded copy. The parser worker in production is started the same way, from the engine's own executable under `policy.max_memory_bytes`, so the question is whether a codec-enabled engine can start its worker at the configured limit. No code was changed for this failure, and the alarm stays open for it.

## Verification

- `cargo test -p rrflow-cli --test installed_reopen --locked -- sustained::campaign::installed_disjoint_writes_from_one_stamp_conflict_without_partial_effect --exact --ignored --nocapture` on Windows, debug profile: passed. Read cursor 16, commits 17 and 18, overlapping writer HTTP 409, two rows and head 18 after reopen. The nightly runs the same test in the release profile on three operating systems; that run is not yet available.
- `python scripts/ci/check_workflow.py`: passed, 9 optional-feature packages.
- The `rrflow-cli` feature lane was not run locally. Its diagnosis rests on the job log (`stdout_bytes=0 stderr_bytes=85`, readiness channel disconnected) and on the code path above. The next nightly run is its first execution.
- Not run: the release-profile installed lane, any Linux job, the Vortex build, the full workspace suite.

No objective, roadmap or POA&M status changes. The `rrd-engine` failure needs a reproduction before it can be recorded as a verified deficiency.

# CI: Windows reopen cache-miss characterization

**Status:** active CI characterization; no candidate or alpha gate passed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-windows-cache-miss-20261107`
**Owner:** repository candidate CI in [CI operations](../../../operations/ci.md).

At public main `838c85120621d5d9dde989d7677cc6ec89d20f03`, PR #270's
[candidate run](https://github.com/rrflow/rrflow-development/actions/runs/42392206110)
attempt 1 timed out after 90 minutes in its Windows release build, before
running installed reopen tests. The
[job](https://github.com/rrflow/rrflow-development/actions/runs/42392206110/job/128461942678)
reported `No cache found` and GitHub's check annotation identified the job
timeout. The final 20 minutes had no new Cargo compilation messages; neither
the log nor the annotation identifies whether compilation, linking or runner
resources caused that quiet interval. No reopen artifact was produced.

The [PR #293 protected merge group](https://github.com/rrflow/rrflow-development/actions/runs/42396236992)
ran the same Windows release-build command with no cache and finished that
build in 35 minutes 18 seconds; both 100-cycle histories and its warning
check passed. Comparing the two candidate heads showed no changes under
`crates`, `Cargo.lock`, `Cargo.toml`, `rust-toolchain.toml` or `.cargo`.
Their workflow, documentation and Connectome gitlink differ. This comparison
supports a bounded retry after a timeout; it does not prove the cause.

At inspection, Actions cache enumeration showed no entries on `main`, while
PR-scoped entries consumed much of the approximately 10.2 GiB listed cache
data. The existing manual trusted-main warmer targets macOS only. GitHub
documents [cache scope and eviction](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching)
and that [standard hosted runners are free for public repositories](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
These observations do not prove a future cache hit or speedup; additional
cache usage can displace other caches or incur charges depending on settings.

The original failure remains retained. PR #270 received one targeted rerun;
candidate and protected merge-group checks must pass on its exact revisions.
No engine behavior, check requirement or roadmap status changed.

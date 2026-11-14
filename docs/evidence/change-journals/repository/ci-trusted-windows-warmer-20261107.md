# CI: trusted Windows reopen cache warmer

**Status:** active draft candidate; warm run and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-trusted-windows-warmer-20261107`
**Owner:** repository candidate CI in [CI operations](../../../operations/ci.md), [PR #296](https://github.com/rrflow/rrflow-development/pull/296); C-07 candidate prerequisites and OBJ-10/11.

The starting public main revision was `838c85120621d5d9dde989d7677cc6ec89d20f03`;
the evidence-only branch revision before implementation was
`c1b36b19623eff64dc00d5b30bd7d7cd932047f7`. The
[preceding characterization](ci-windows-cache-miss-20261107.md) records
the exact timed-out cold candidate, successful comparable cold build, cache
scope and uncertainty. The PR brief named the target, owners, paths, safety
boundary and checks before the workflow or policy was edited.

Created `.github/workflows/ci-windows-cache-warm.yml`. It is a separately
dispatched, main-ref-only job on the fixed GitHub-hosted Windows runner. Its
locked release build, two workers, profiles, recursive checkout, toolchain,
action pins, shared key and failed-cache policy match the Windows reopen
candidate. A 120-minute timeout applies to this preparatory job alone. The
existing candidate and merge-group jobs, their 90-minute limit and every
required test are unchanged. No self-hosted runner, additional token permission
or runtime dependency is added.

Changed `scripts/ci/check_workflow.py` and
`scripts/ci/test_cache_warm_policy.py` to enforce that envelope. Negative
tests reject non-manual events, non-main admission, duplicate or extra jobs,
runner and environment drift, extra permissions or steps, untrusted checkout
configuration, action/key changes, failed-cache saves, and candidate build
drift. Changed `docs/operations/ci.md` with dispatch and evidence guidance.
This record and the generated repository journal index are added. The
complete changed files and diff were reread; the existing macOS warmer,
candidate job and action inputs were checked as the comparison boundary.

Local Windows checks:

- `python scripts/ci/test_cache_warm_policy.py`: 7 tests passed.
- `python scripts/ci/test_candidate_admission.py`: 21 tests passed.
- `python scripts/ci/check_workflow.py`: passed for six workflows and the
  unchanged required candidate partitions.
- The new YAML parsed with locally installed PyYAML 6.0.3 using `BaseLoader`;
  its manual trigger, Windows runner, timeout and four-step shape were checked.
  This supplemental local parse is not a GitHub execution test.

The [GitHub cache-scope reference](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching),
[hosted-runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
and pinned action configuration informed the design. No upstream source was
copied. The cache is a replaceable build acceleration, not engine state or a
test receipt. A successful manual build is not candidate acceptance. Cache
key derivation, save availability, eviction, variable runner performance and
storage billing still require observation; a valid workflow does not prove a
hit or a speedup. `actionlint` was not installed locally.

After the exact candidate and protected merge-group CI pass and this workflow
is on public main, dispatch that main revision, inspect its cache-save result,
then inspect a later candidate's actual restore log and build timing. The
existing PR #270 retry and PR #294 candidate are separate. C-07 capacity,
project-source visibility and alpha remain open. No roadmap or POA&M status
changed.

Change checklist: owning CI failure and exact starting revision identified;
PR brief published before implementation; explicit trusted execution and
unchanged gates specified; source and diff reread; negative policy tests and
local YAML parse passed; this linked record retains the checks and the missing
live evidence. No trace or resource sampler is added to product behavior.

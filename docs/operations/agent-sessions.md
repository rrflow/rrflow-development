# AI agent session standard

**Status:** active operational contract; enforced where a check is named, otherwise a rule a session must follow and report against
**Coordinate:** `rrflow://rrflow-instance/data/operations/agent-sessions`
**Owner:** how any AI session starts, works, claims results, uses machine resources and ends in this repository; indexed by `docs/operations/README.md`

[`AGENTS.md`](../../AGENTS.md) owns the repository rules, including the pre-release process of 2026-11-16: one short-lived branch and one pull request per package, merged through the queue on a green gate, with no compatibility and no legacy. This record turns them into one procedure a session can be held to, so the owner does not have to rediscover the rules each time. It adds no lifecycle, status or planning authority. The release roadmap owns gate status and the POA&M owns deficiencies.

## Why this exists

By 2026-11-14 the repository held 35 worktrees, 172 local branches, 8 stashes and 81 archived refs, and a green test run had been reported as proof of work it did not cover. Absorbing that took three days of the owner's time. Every rule below answers one observed failure.

## Start

1. Run `python3 scripts/ci/check_session_end.py`. A session starts from the state it must end in. Anything it reports is work from an earlier session; say so before beginning new work.
2. Read the open `Main is red` issues. Repair them first.
3. Read the owning records for the package (`README.md`, then the owner it names). State the package's current behaviour, target behaviour, oracle and stop conditions before changing a file, and record them in the package journal.

## Work

- **One package, one branch, one pull request.** Branch off `main` in the repository's own checkout, push to `development` only, open the pull request, enable auto-merge, and let the queue merge it on a green `pipeline / ci-gate`. No direct push to `main`, no stash, no archive ref, no second worktree. Park nothing: finish the change, or make it a recorded POA&M item. Pre-release means no compatibility, no legacy and no second implementation.
- **Prove on the real path.** Run the smallest relevant test first, then the owning suite, on the platform the claim covers. Read exit codes. A pipeline that hides a failing command is not a result.
- **Never weaken a probe.** A failing test that exposes a defect is fixed or kept failing and recorded. Deleting it, ignoring it, filtering its symptom or loosening its oracle is reward hacking.
- **Fix causes.** A rerun is not a fix. A policy check that fails is satisfied by correcting the content, not the check, unless the owner approves changing the rule.

## Claims

State a result at the level the evidence supports, and say which level it is:

| Level | Evidence | May be called |
|---|---|---|
| Carried | The name, route or line exists on `main` | "present" |
| Read | A person or session read the code against the intent and recorded the comparison | "reviewed" |
| Tested | A named test failed before the change or would fail without it, and passes now, on the stated platform | "verified on that platform" |
| Measured | A recorded run with commands, inputs and raw output, reproducible from a clean clone | "measured" |

Ancestry, a green compile, a mock, an agent summary and a self-written script's verdict are not evidence for "absorbed", "done" or "competitive". End every substantive report with what is unbacked, what was not run and the one next step.

## Absorbing old work

Before deleting anything that holds work (a branch, stash, ref, folder), in order:

1. Save its tip in a bundle and run `git bundle verify`.
2. Compare every added line and every declared name with `main`, then read what is absent. A name present does not prove the behaviour or its documentation is.
3. Record the verdict per item in a ledger under `docs/evidence/`.
4. Delete it, then run `check_session_end.py`.

## Machine resources

Sessions share the owner's machine with other projects. A session removes only what it created.

- Create temporary files under the session scratchpad, never beside the repository or under `D:\localdev`.
- Start every container with `--rm`, a name and `--label rrflow.session=<id>`, and remove it in an exit trap. The lab scripts (`eval/lab/pod.sh`, `run_system.sh`) are the pattern.
- Build output goes to one out-of-tree `CARGO_TARGET_DIR` per checkout, with `CARGO_BUILD_JOBS=4`.
- Never remove a container, image, volume or folder this session did not create. List it, say whose it appears to be and leave the decision to the owner.
- Before ending, list what the session created and removed.

## End

1. Push the package branch, open or update its pull request, and enable auto-merge. Read the gate result for that exact commit.
2. After the merge, delete the branch locally and on the remote.
3. `python3 scripts/ci/check_session_end.py` passes.
4. Report: what ran and passed, what failed, what was not run, what is assumed, and the next step.

## Enforcement today

| Rule | Check |
|---|---|
| Clean tree, no stash or archive ref, every branch pushed with an open pull request, none merged and left behind | `scripts/ci/check_session_end.py`, tested by `scripts/ci/test_session_end.py` in the scripts lane |
| Red candidate or red main repaired first | The `merge-alarm` workflow opens `merge-blocked` and `Main is red` issues |
| Pull request, `pipeline / ci-gate` and the merge queue required on `main`; force push and deletion denied | The `main` ruleset on `rrflow/rrflow-development` (applied 2026-11-17) |
| Code shape does not regress | `scripts/ci/check_code_shape.py` |
| Every crate sits in its tier, folder and name, and depends only downward | `scripts/ci/check_architecture.py` over `architecture.toml` |
| Records link, classify and name their owner | `scripts/ci/check_documentation.py` |

Not yet enforced, so a session must follow and report them: claim levels, container labels, scratch location and the absorption order. Turning them into checks is POAM work, not a promise made here.

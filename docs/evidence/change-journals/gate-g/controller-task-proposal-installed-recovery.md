# Installed controller trial and retained uncertainty

**Status:** active partial installed evidence; successful inference and fault cases pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-task-proposal-installed-recovery`
**Owner:** G-01a/G-01b and G-02/G-05; OBJ-12 and POAM-009 remain open
**Pull request:** [206](https://github.com/rrflow/rrflow-development/pull/206)

## Brief and inspected baseline

Baseline `d365d02891350590aa4495b46fc665ef313aec24`, tree
`38617606a687d16dfdfea42998ce4d003c40cd6f`. The existing complete-operation PR
brief includes installed execution, denial, restart and uncertainty evidence.
This stage changes its repository-owned proof/recovery tooling, not engine Rust
behavior, API schemas, training data or the installed binary.

Read README/AGENTS, the existing admission journal, execution-map evidence
requirements, POAM-009, generated Gate G journal index, inventory generator and
`scripts/inference/installed_controller_proof.py`; inspected the actual build
receipt, running operation catalogue, installation announcement, task/proposal,
failure and recovery records. Operator input files and cluster-access instructions
remain local; credentials were accessed in memory and are not journal content.

## Actual run and correction

The clean Windows development build from the baseline has executable SHA256
`7fe07a7198b864a2642c5a9a521e8e13ac7a7d935fd62e8477f2c075fd7680b9`.
It was copied and installed in a separate project. The first driver run stopped
at a task-open 404 before proposal dispatch: its guessed path was wrong. The
replacement uses the installed announcement to discover the actual catalogue,
then resolves operation method, path and mutation declaration from that owner.

A fresh trial opened the task from an explicitly bounded projection of the real
PR206 title/body, applied the operator's experimental profile, and verified
unauthenticated 401 plus missing-credential 412 with no reserved budget. The
authorized operation then returned 412, `retryable: false`, and
`EndpointCallFailed` with `execution_may_continue: true`. No successful model
proposal was observed. Later connection to the private forwarded port was refused;
SSH then required a Tailscale identity check. Those later observations do not
establish whether the earlier request reached the provider.

The proof driver now accepts `--recover-proof EXISTING --output NEW`. It verifies
the retained binary hash/build identity, reopens that same installed estate,
authenticates and reads only task-attempt/task-budget operations. It does not
pass provider credential configuration, replay a proposal, reset the store or
create a replacement task. Mixed recovery/dispatch arguments are rejected before
output/process creation. Output directories are exclusive; prior evidence stays.

Two actual recovery runs returned identical attempt and budget state: stopped
attempt, one reserved step, 2,057 ms charged effort, `attempt_failed`, and
`execution_may_continue: true`. All owned server processes were stopped. The
public readback proves retained uncertainty and effort; it does not expose or
independently measure the private capacity index or prove remote termination.

## Checks, trace and limits

Executed against the identified native binary:

```text
python scripts/inference/installed_controller_proof.py --recover-proof <failed-proof> --output <fresh-recovery>
python scripts/inference/installed_controller_proof.py --recover-proof <failed-proof> --output <fresh-repeat>
```

Exact commands, source/run identities and raw public responses are retained under
the operator's `.swag/evidence/20261029/installed-controller-proposal{,-discovery,-recovery,-recovery-repeat}/`
and `controller-operation-plan/`. Comparing both recovery JSON files passed exact
attempt/budget equality. A subprocess argument-rejection check returned exit 2 and
created no output directory. Python syntax compilation passed. Runner SHA256:
`77e25e637ae49beed29aafe00ef396c2f2ffc5a4237db962876ad3501bd60950`.
No invented provider response or quality label was needed for these checks.

Navigation, documentation, inventory, workflow and staged whitespace checks
passed: 18 indexes, 745 document statuses and 1,980 inventory records. Commands
were `render_navigation.py --check`, `check_documentation.py`,
`build_execution_inventory.py --check`, `check_workflow.py` under their existing
script owners and `git diff --cached --check`. No Rust/SDK behavior changed; the preceding passing package suites
are not rerun solely for this driver/doc correction. Existing journals and failed
attempts remain unchanged. No new research algorithm or performance claim applies.
Trace uses actual request/operation IDs, binary identity and before/after public
receipts. Recovery starts only its owned local server and cannot spend provider
capacity. POAM-009 records the unresolved reconciliation limit.

The PR stays draft. Successful installed inference, accepted-result replay,
Stop/revocation/configuration-change cases, candidate CI and protected landing
are still required. No new GPU training, admitted dataset, worker effect,
ordinary-edit successor plan, Connectome/RROFLOW journey, release image or
29 GB qualification is claimed. Preserve the uncertain estate; restoring a tunnel
alone is not authority to clear it or retry the unknown effect.

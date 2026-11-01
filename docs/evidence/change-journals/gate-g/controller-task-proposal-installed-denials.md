# Installed controller admission denials

**Status:** active installed predispatch evidence; live completion and in-flight cases pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-task-proposal-installed-denials`
**Owner:** G-01a/G-01b and G-02/G-05; OBJ-12 and POAM-009 remain open
**Pull request:** [206](https://github.com/rrflow/rrflow-development/pull/206)

## Brief, baseline and reads

Baseline `bda5321b79d98cfd704edf01f08d786a98819260`. The existing PR brief covers
installed denials/revocation/configuration change and recovery. This stage extends
only `scripts/inference/installed_controller_proof.py`; the binary and Rust engine
source remain `d365d02891350590aa4495b46fc665ef313aec24`.

Read the current proof driver, endpoint-proposal admission, endpoint-configuration
and reasoning-budget contracts, public task-budget implementation, session
handlers/authorization, service error mapping, the offline endpoint-selection
example and retained installed catalogue/profile/task inputs. README/AGENTS,
previous journals and the change-authoring rules remain applicable. No source
adaptation, external research algorithm, runtime dependency or gate change applies.

## Behavior and smallest oracle

`--admission-only` takes the existing actual profile/task inputs, build receipt and
contract helper. It excludes provider credential arguments and recovery mode,
requires an explicitly credentialed binding, and starts its own isolated installed
estate with no provider credential map. The same actual PR body supplies task
context; no model output or correctness label is generated.

The driver uses real public configuration preview/apply to disable network,
restore policy, revoke the profile and reactivate it. It derives fresh request
hashes through the Rust contract helper. It then closes its actual session and
attempts admission with that closed session. Paths/methods come from the engine
catalogue, including the session parameter. Denial assertions check the specific
engine reason, not merely any non-success status. Process restart reads the final
configuration exactly and proves the fresh request reaches the expected missing
credential check. No provider credentials were supplied in this run.

Observed installed outcomes:

| Controlled action | Actual response |
|---|---|
| Unauthenticated request | 401 |
| Missing credential | 412; configured endpoint credential unavailable |
| Network disabled in applied policy | 403; PermissionDenied |
| Old request after real configuration replacement | 412; InputsChanged |
| Fresh request after restoration | 412; configured endpoint credential unavailable |
| Applied profile revocation | 400; endpoint profile absent |
| Closed session | 412; SessionExpired |
| Restart with reactivated profile | Exact configuration readback; fresh request reaches missing credential |

Task-budget readback remains absent throughout and after restart. Owned servers
were stopped. This proves no effort reservation for these admission failures;
it is not concurrent revocation, in-flight Stop, provider termination or routing
qualification. Restored admission proves configuration freshness recovery only,
not repository-source edit discovery or successor planning.

## Validation and evidence

The installed command was:

```text
python scripts/inference/installed_controller_proof.py --admission-only --build-receipt <d365-build-receipt> --contract-example <built-endpoint-selection-example> --profile <actual-profile> --task-source <actual-PR206-source> --output <new-denial-evidence>
```

Receipts are under the operator's
`.swag/evidence/20261029/installed-controller-admission-denials/`.
Binary SHA256 `7fe07a7198b864a2642c5a9a521e8e13ac7a7d935fd62e8477f2c075fd7680b9`;
runner SHA256 `27e9fc99981f2daa1908d5bff9c032a10e295b5e69453633add59490d2fddbd1`.
Two mixed-mode subprocess checks exited 2 before creating output or a server.
Syntax compilation passed. Re-running recovery against the original uncertain
estate returned exactly the prior attempt and budget; the new receipt is in
`installed-controller-recovery-after-denial-tooling/`. No failed estate was cleared
and no model request was repeated. Prior journal/capture bytes remain unchanged.

Navigation/documentation/inventory/workflow and staged whitespace checks passed:
18 indexes, 746 document statuses and 1,981 inventory records. Commands were
`render_navigation.py --check`, `check_documentation.py`,
`build_execution_inventory.py --check`, `check_workflow.py` and
`git diff --cached --check` at their existing repository owners. The passing Rust/SDK suites are
not rerun without a runtime/schema change. Trace uses actual public request IDs,
configuration revisions, input hashes and owned process receipts. Exclusive output
directories preserve failures. No new known product deficiency was introduced;
POAM-009's successful-execution and uncertainty-reconciliation gaps remain open.

Cluster SSH authentication is still pending. Successful installed BF16 completion,
accepted-result restart/replay, in-flight Stop/authority/input-change cases and
candidate CI remain required before this draft is ready. No training, fabricated
labels, new provider observation, learning admission or full integration is claimed.

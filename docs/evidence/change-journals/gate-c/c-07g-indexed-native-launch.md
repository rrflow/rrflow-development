# C-07g indexed candidate: observed native launch

**Status:** active launch evidence; full corpus and soak acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-indexed-native-launch`
**Owner:** OBJ-02 / C-07g / POAM-035 / PR139

## Scope and identity

This follows the [integrated candidate record](c-07g-indexed-native-candidate.md).
The current Codex driver updated [PR139](https://github.com/rrflow/rrflow-development/pull/139)
with the evidence-only brief before this edit. Scope is this journal, POAM-035
link and generated navigation/inventory; no runtime, test, public API,
configuration or corpus changes. Stop for an unbound artifact, unverified
launch, changed limits or an unsupported completion claim. Native source stays
frozen at the revision below even when this evidence-only commit advances the
branch.

- Source: `7c329d491cda5326858de0eab36a6f9912c492da`.
- Tree: `cdb2f8d316e4ec5c96b68bd6587afc75df0069a7`.
- Primary SHA-256: `7dfc4a69b8829579b9f33eea1fc1fc4275e0463ed4585c1c34f6d9dceff05416`.
- Harness SHA-256: `f9ba9f32aef84878b451d151bad4d1d421321127139f5cf91534a898732bc245`.
- Configuration SHA-256: `d2013902bcc7c9efe4840170d35a35aae22e5fcffd0f1f23eca6e9c0e7fcfbf2`.
- Campaign-file SHA-256: `634e380e113a753e448f1bab359598d8193b19a06b38dc35dfe03faf396cf035`.
- Semantic campaign digest: `170295519223ad438da894799c5f60767a1090db542bde5bf63c4edb23d7df7f`.
- Service launch receipt SHA-256: `ee55f5aedcc359b9ebe64270c2a66e011627a1db378106e7ceb3d043f0dce518`.

## Actual commands and observations

On the integrated source, the Windows command
`cargo test -p rrflow-cli --test installed_reopen --locked -- sustained::installed_capacity_workload_smoke --exact --ignored --nocapture`
passed one real installed smoke test: 9.38 seconds test execution, 62.735 seconds
including build. It exercises mixed-family independent readback, backup,
read-lifetime interference and restart. Local retained log
`target/c07-indexed-installed-smoke.log` is 6136 bytes, SHA-256
`ae05bc670b1cd7b2e366e517094c71e7be96ea6801b562d6ae213293b0b5d32f`.
This is a smoke result, not full-scale acceptance.

Before staging, the owned guest reported no running `rrflow-c07-*` services
and 93305454592 free bytes. The public exact revision and recursive pinned
submodule were fetched into a new clean checkout. Preparation service
`rrflow-c07-desktop-prep-7c329d49` started at 2026-10-26 13:31:54 UTC. Locked
offline release primary/harness compilation completed in 7m11s. Installation,
authenticated storage observation and campaign validation succeeded; the
preparation service exited zero. No failed estate was reused or deleted.

The reviewed launcher started `rrflow-c07-desktop-integrated-7c329d49` at
2026-10-26 13:40:06 UTC, with a 15150-second outer limit. Read-only inspection
confirmed active/running and 896 acknowledged setup operations at that sample.
This is partial setup, not completed readback or throughput evidence. A
subsequent read-only inspection of the actual serving PID's cgroup reported
`memory.max=2147483648` and `cpu.max=400000 100000` (four CPUs). The frozen
allocation remains 8 GiB guest/2 GiB engine, 64 MiB cache, 100000 records,
500000 edges, 100000 vectors of dimension 384, four clients and 5 requested
operations/sec. The 30-second operation and two-hour setup limits are unchanged.

Retained guest root:
`/home/devcontainers/rrflow-c07-native-79663cdd/desktop-integrated-preflight-7c329d49/`.
It contains campaign/configuration/build/installation identities,
`service-launch.json`, `evidence/progress.jsonl` and launcher output. The fresh
estate parent is `desktop-integrated-files-7c329d49`. Build log is the sibling
`desktop-preparation-7c329d49.log`. Local observations are retained as
`target/c07-indexed-launch-inspection.log` and
`target/c07-indexed-resource-inspection.log`. Terminal result and cleanup
receipts must be read after completion; they did not exist at this observation.

## Review and limitations

Full reads: previous integration journal, reviewed staging/preparation/launch
scripts, launch receipt, installed smoke output and direct resource observation.
Research: no new source adaptation; the [implementation journal](c-07g-point-range-index.md)
owns primary research and component oracles. Trace/debug/resource decision:
retain exact identities, live service state, cgroup limits and independent
setup/readback/load phases. No source replacement or deletion occurs here.

Checklist: pre-effect PR brief, exact source/artifacts, actual command results,
retained prior failures, declared scope/stop conditions, complete evidence diff
review and regenerated projections. Documentation, navigation, execution
inventory, workflow and whitespace checks passed before push. No further Cargo run is needed
for this evidence-only follow-up; the exact integrated smoke is recorded above.

PR140 candidate run 40326533968 still awaited Windows/macOS 100-cycle reopen
jobs at this observation; 20 other jobs had passed, including Linux's reopen
and all three native crash/capacity jobs. PR141/139 remain draft pending shared
qualification and normal integration. No merge or completed candidate gate is
claimed here.

C-07/POAM-035 remain open. Complete independent full-corpus verification,
measured profile acceptance and the full capacity matrix are not proven.
The campaign's short 600000-ms soak phase is characterization only;
**no actual 24-hour soak has started**. The driver must retain the terminal
result under this frozen source, then decide the next qualification action
from that result without changing limits or transferring evidence to a later
binary.

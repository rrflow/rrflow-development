# Installed incremental Inventory trial and unresolved startup rejection

**Status:** active implementation evidence; native qualification incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-incremental-installed-trial`
**Owner:** OBJ-14 / D-03 / POAM-006; PR226 driver: active SWAG session

## Source, artifact and scope

This follows the [implementation record](d-03-incremental-publication.md) without
changing its historical observations. Public PR226 source
`923326ab8feb7b5eb8db38ff1050b7dbece17c3d`, tree
`95546fd0c25e04261f7f4c25f125d2455330bfc6`, was clean when built with
`cargo build -p rrflow-cli --bin rrflow --locked`. The dedicated
`target/delta-168e93c6` used two build jobs and disabled dev/test debug information.
Build exit was zero after 5m21s. The separately copied Windows primary has one
filesystem link, 150,600,192 bytes and SHA-256
`df1cb50ddd70873ad22a29668a91618ac05c9a51d0af88c04eec9ab818dead2f`.
It is an unoptimized engineering executable, not a release or model artifact.

The disposable actual repository contains 1,870 tracked paths at public source
`d1b8a26a73cfb8f01791737913b4098988c1acdd`, tree
`c4cc385adc54705cd5b22cdc26ade218b2ebd4a0`. Earlier estates were preserved.
The Python public SDK was installed from its frozen lockfile and used actual
loopback processes, private installed credentials in memory, unique request keys,
120-second request deadlines and unchanged 60-second job leases. No provider,
model, project worker or learning admission was invoked.

## First failure and preserved recovery

The initial trial successfully planned and applied installation. Its first
`serve` exited with `ProjectBindingMismatch`, producing no readiness line; the
harness consequently raised `JSONDecodeError`. Inventory had not executed.
The original result, stderr and estate remain intact. A subsequent offline
quick verification passed, including current source and executable identity.
One separate bounded startup probe then reached `listening` using the identical
executable and estate. No binding check, installation record, budget or binary
was changed. These observations do not establish the cause or a fix.

A separate continuation verified the preserved installation, read the untouched
revision-one job with no checkpoints, and proceeded without reinstalling. Its
three process passes recorded:

| Observation | Runtime mutations | Seconds |
|---|---:|---:|
| Initial complete Inventory | 4,279 | 24.427 |
| Unchanged repeated Inventory | 3 | 16.636 |
| Restart, README edit, accepted successor, Inventory | 6 | 14.050 |
| Unchanged repeated successor Inventory | 3 | 13.704 |

Each restart recovered the exact prior job. A third process recovered the final
job; offline quick verification passed in 5.409 seconds. These are single-run
observations, not a latency benchmark or RSS qualification. The job's scheduled
Verify phase was not executed. Restoring the original README after verification
made the successor source binding stale again; it was not relabeled current.

## Reproduction and retained evidence

Local raw evidence is under
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/`:
`delta-primary-build.log`, `delta-publication-source-envelope.json`,
`delta-repository-trial.py`, `delta-repository-result.json`,
`delta-repository-server-0.log`, `delta-failed-estate-quick-verify.log`,
`probe-delta-startup.py`, `delta-startup-reprobe.json`, and the separate
`delta-resumed-repository-trial.py` / result / process logs. The original and
continuation result SHA-256 values respectively are
`2b59ed7cc40c94f4aa305c4cf56874901a27713051da84005c91209e9ba6d077` and
`a9ef9789722465ceb60a919b1d518c293fa80e15f1349be2e6c72601f49ef28c`.
The scripts ran with `uv --directory sdks/python run --frozen python` from the
candidate worktree; each preserves its result and targets the recorded estate.
Do not replay installation over these evidence estates.

The package review inspected the implementation diff, reuse/current-record
tests, publication recovery, installation open/inspect checks, serving publisher,
bootstrap verification and actual CLI/SDK trial. Only this journal, POAM-006 and
generated navigation change in this evidence increment. No production fix is
inferred from the successful continuation; the 341-test implementation result
retains its exact source and prior failed run. Navigation, documentation,
execution-inventory and whitespace checks accompany this documentation commit.

## Remaining acceptance

PR226 stays draft pending diagnosis of the initial native startup rejection,
required candidate/protected checks and PR224 landing. PR224's exact-head
candidate run `41161519177` passed all 23 jobs, but queue admission reports its
required check as expected despite the successful check. A later same-head
skipped suite was observed; causation is unproven. Full normal dispatch
`41165175925` refreshes required CI without bypass. Neither PR is claimed merged.
No roadmap status closes here. Full scans, safe upgrades, release delivery and
the task-to-worker-to-observation controller journey remain incomplete.

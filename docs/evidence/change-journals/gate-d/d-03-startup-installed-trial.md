# Installed startup recovery and retained Inventory behavior

**Status:** active installed Windows evidence; candidate/protected qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-startup-installed-trial`
**Owner:** OBJ-14 / D-01 / D-02 / D-03 / POAM-006; PR227 driver: active SWAG session

## Source and change boundary

This records the installed acceptance in [PR227](https://github.com/rrflow/rrflow-development/pull/227)'s
existing brief, following its [implementation oracle](d-03-startup-binding-inspection.md).
Clean source `39a7b6444c933bc2b931f5a00825942f5f0dd46f`, tree
`92b6b96c759f77a7a79eb5f2185607ab9457c402`, built successfully with
`cargo build -p rrflow-cli --bin rrflow --locked` in 6m24s. The dedicated
`target/startup-f1a033b7` used two jobs and disabled dev/test debug information.
The independently copied primary has one filesystem link, 150,641,664 bytes and
SHA-256 `dfcbda6fa7743ca8851b42187e46350291dc6a3970339288a1a4f19cb105a9c3`.
It is an unoptimized engineering executable, not a distribution or model artifact.

The disposable repository has 1,870 tracked paths at public source
`d1b8a26a73cfb8f01791737913b4098988c1acdd`, tree
`c4cc385adc54705cd5b22cdc26ade218b2ebd4a0`. Earlier failed and successful estates
remain intact. The frozen Python SDK drove actual loopback processes with
installed credentials retained privately, one attempt per request, 120-second
request deadlines and unchanged 60-second job leases. No model, worker command,
training, independent review or consent was manufactured or invoked.

## Installed observations

Installation preview and exact apply passed. The harness then replaced only the
disposable project's Git configuration with invalid bytes. The server reached
listening in 0.809 seconds and returned its original job. Source-dependent Start
rejected with `Storage("Git inventory input: InvalidConfiguration")`; the HTTP
surface currently returns 500/internal for this error. Authenticated readback
proved the job unchanged. Restoring the saved valid configuration allowed Start
and Connect. No reinstall, binding bypass or retry concealed that denial.

| Observation | Runtime mutations | Seconds |
|---|---:|---:|
| Initial complete Inventory | 4,279 | 26.120 |
| Unchanged repeated Inventory | 3 | 15.672 |
| Restart, README edit, accepted successor, Inventory | 6 | 24.101 |
| Unchanged repeated successor Inventory | 3 | 25.152 |

All three server starts and exact prior-job readbacks passed. Replan preview and
acceptance took 6.104 and 7.967 seconds. The final job still names Verify as its
next phase; the scheduled phase did not execute. Offline quick verification
passed all 13 checks in 7.027 seconds at runtime cursor 4,293.

That report explicitly returned `attunement_source_current: false` before cleanup.
Inspection compares current content to the original installed source precondition,
not the accepted successor's source. A successful quick verification therefore
does not assert current project attunement. The original README and Git config
were restored after the trial; restoring README makes the successor binding stale.
No source-freshness value is relabeled. These are single-run correctness
observations, not latency, memory, scale or repeatable-startup qualification.

The seven existing installed CLI lifecycle tests also pass, including selected-plan
tamper denial, fresh/existing install, serve/authentication/reopen, forced-restart
replan/receipt behavior and duplicate-serving refusal. Command:
`cargo test -p rrflow-cli --test installed_lifecycle --locked`; seven passed,
zero failed or ignored, 20.29 seconds after a 42.31-second compile. The earlier
342-test engine suite and strict lint remain attributed to implementation source
39a7b644, rather than being counted as new tests in this evidence increment.

## Evidence and reproduction

Local raw records remain under
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/`:
`startup-primary-build.log`, `startup-binding-source-envelope.json`,
`startup-repository-corpus.json`, `startup-repository-trial.py`,
`startup-repository-result.json`, `startup-repository-trial.log`, the three
`startup-repository-server-*.log` files and `startup-installed-lifecycle.log`.
The trial command was
`uv --directory sdks/python run --frozen python <evidence>/startup-repository-trial.py`.
It exited zero, with completed/README-restored/Git-config-restored all true.
The script refuses an existing result or installation; do not replay it over this
preserved estate. Raw machine-local records are not a publicly downloadable bundle.

| Record | SHA-256 |
|---|---|
| Source/artifact envelope | `1dff9505b4f00e44d9af6b6b024b68e4a02523f37d0baceee8473ec4257fdda4` |
| Trial harness | `9b16dc759dd6d85f98a7cc94eb983f8d02ce5b6fbad368404e472c14c3d15a9e` |
| Completed result | `fe5ee0b2c93089dd0ba69331ec6473da98b1f394d951a28f4678b5bcd0edc77e` |

## Review and remaining acceptance

This evidence-only increment changes this journal, POAM-006 and generated Gate D
navigation. It preserves the prior immutable journals. Review covered the full
trial/freezer, resulting records and logs, the CLI lifecycle cases, and the
installation report's source-precondition comparison; production implementation
review remains in the linked implementation record. No new research or production
tracing is needed for this exact-binary correctness trial. Resource measurements
were not collected, so none are claimed. Navigation passes with 830 nodes and
3,035 edges. The first execution-inventory check detected the newly added journal;
regeneration and its check pass with 2,104 records. Documentation passes with
831 statuses and 829 coordinates; whitespace checks pass. Full changed-file and
diff review found only the three declared documentation paths. There is no new
source change requiring another engine suite.

PR224 remains the landing predecessor. PR227 retains PR226's implementation and
now has a combined installed startup/Inventory/edit/replan history; neither ancestry
nor this local trial proves landing. Keep PR226 open until the composed behavior
has a verified main destination. Candidate/protected CI, platform qualification,
safe executable upgrades and the actual task-to-worker-to-observation journey
remain open. The original intermittent PR226 binding rejection's exact cause is
still unproven. No roadmap or POA&M item closes here.

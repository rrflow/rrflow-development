# Installed LFG and selected-worker readback

**Status:** active local controlled evidence; integration and protected CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/lfg-selected-worker-installed-controlled-20261104`
**Owner:** OBJ-06, G-01b/G-02a, H-05 and POAM-009; PR249 / PR229

This evidence follows the [local composition](lfg-selected-worker-composition-20261104.md).
The developer executable was built from clean local merge commit
`0fc8c9759032222fcf9d61777eb493095ab33012` with
`python scripts/build_rrflow.py --profile dev --target-dir
D:/localdev/devlabs/rrflow/target/lfg246-contract-build --output
D:/localdev/devlabs/rrflow/target/worker-lfg-dev-build-receipt-20261104.json`.
Its unqualified executable SHA-256 is
`19d00c00ae1d31415ed6cc4c1c3913923d7fb9dd4ead35a03483ce8d86f70ca3`.
The typed endpoint contract example was built from the same source.

An isolated driver ran the existing installed public-operation proof with
`--worker`, an explicit test profile, synthetic task and generated test
credential, a running local LFG service, and an owned loopback HTTP fixture
for exactly one selector and one worker reply. It opted into
`RRFLOW_TRACE=rrd_engine::lfg=debug` and `RRFLOW_TRACE_FORMAT=json`.
The driver exited zero. Independent assertions then compared the executable
hash, LFG input/response hashes, emitted analyzer wire hashes, the selector
and worker packet digests, the worker's task/configuration/selection binding,
and the separately persisted analyzer and worker attempts. Both attempt
outcomes matched after process restart and each same-attempt replay was
rejected with HTTP 412. The diagnostic events had the expected field
allowlists and omitted the generated secret, synthetic task title and local
endpoint. The [sanitized receipt](lfg-selected-worker-installed-controlled-20261104.json)
records those identities and outcomes; raw proof files remain under ignored
`target/`.

The observed LFG process used a local executable path whose on-disk SHA-256
at readback was
`91f79f04ea56b360654f0c5071925e648a8f0918ec84baa09c2b6145c003e7b0`.
Its loaded image and source revision were not independently attested. The
selector returned a predetermined fixture choice and the worker returned a
fixture draft: this does not prove that standalone SWAG used the advisory,
that a model ran, or that any result is correct or efficient. No prompt-flight
provider arm, project-source mutation, native capacity/soak or release
qualification follows. Debug events are transient; stderr can block and
they do not replace durable attempt records.

The only changes in this increment are the evidence receipt, this journal and
the generated navigation entry. The exact source and binary identities above
remain attached to the test rather than to a later evidence-only commit. The
separate dependency PRs, their actual merge order, and protected candidate and
merge-group CI remain open; no roadmap or POAM status changes.

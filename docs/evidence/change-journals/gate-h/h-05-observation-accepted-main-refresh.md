# H-05: observation delivery and current accepted main

**Status:** active reconciliation evidence; local validation passed, candidate acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-05-observation-accepted-main-refresh`
**Owner:** H-05 observation delivery prerequisite and POAM-008

## Bound change and first oracle

The SWAG controller integration session drives
[PR235](https://github.com/rrflow/rrflow-development/pull/235). Starting head
`b856a8fb08200c9117d759c43f9eaad0ec55ef77` passed all 23 candidate jobs in
[run 41342695388](https://github.com/rrflow/rrflow-development/actions/runs/41342695388),
but did not compose with current accepted main
`e6f3632b32f1c9be33c1951886a9d9a2be648ddf`. Their merge base is
`daec1f1a2959f16ce41da9fb691ab1791ed06db5`.

The pre-edit PR brief binds this ordinary merge to the existing delivery
package. `git merge-tree --write-tree --name-only` reproduced two conflicts:
`crates/authority/rrd-engine/src/engine/tests/attunement.rs` and the generated
Gate H journal index. No new runtime capability, dependency, provider policy,
learning data or execution authority is introduced by this reconciliation.

## Resolution and traceability

The attunement test file equals accepted main byte-for-byte with the single
`mod observation_delivery;` registration added. Main's complete chronology
repair subsumes the earlier branch repair: recovery, resume and subsequent
operations use returned completion times. Every assertion remains intact.
The owning navigation generator rebuilds the journal index from both parents.

The automatically merged workflow retains main's checks, macOS toolchain
isolation and stale-snapshot oracle, together with this package's observation
adapter suite inclusion. The public protocol reference retains main's stale
transaction recovery guidance and this package's OpenAPI digest. Canonical
contract and SDK bytes are unchanged from the starting observation candidate.

Exact Git mode/blob comparison verifies all 2,112 non-overlapping parent entries.
The four overlapping paths are the workflow, attunement tests, journal index
and protocol reference. No implementation is moved or deleted. Existing sender,
authorization, storage and adapter owners remain canonical. This new journal
and its generated index entry are the only additional evidence paths.

## Validation

The focused attunement/observation suite passed 45 tests with two existing
ignored helpers. All 165 contract/adapter tests and generated surface parity
passed. The combined primary executables built. Engine/server/client/evaluator
all-target suites report 511 passes, zero failures and four existing ignored
cases. One Rust SDK entry can return early without its live-daemon manifest;
that test count does not replace candidate shared-daemon SDK conformance.
Strict affected-package lint, formatting and CI policy pass. Exact commands,
exit codes, raw logs and SHA-256 digests are retained in the private dated
`observation-main-refresh` evidence directory.

| Command | Result |
|---|---|
| `cargo test -p rrd-engine --lib --locked -- engine::tests::attunement` | 45 passed, two existing ignored helpers. |
| `cargo test -p rrd-contract -p rrd-observation --all-targets --locked` | 165 passed. |
| `python scripts/ci/check_generated_surfaces.py --check` | 74 operations and seven signal projections agree. |
| `cargo build -p rrflow-cli --bins --locked` | Pass. |
| `cargo test -p rrd-engine -p rrd-server -p rrd-client -p rrflow-eval --all-targets --locked` | 511 reported passed, four ignored; SDK limit above. |
| `python scripts/ci/test_inspect_candidate_runs.py` | Nine passed. |
| `python scripts/ci/test_analyze_sustained_conflicts.py` | Eight passed. |
| `cargo clippy -p rrd-contract -p rrd-observation -p rrd-engine -p rrd-server -p rrd-client -p rrflow-eval --all-targets --locked -- -D warnings` | Pass. |
| `cargo fmt --all --check` | Pass. |
| `python scripts/ci/check_workflow.py` | Pass. |
| `python scripts/knowledge/render_navigation.py --check` | 18 indexes, 935 nodes and 3,339 edges; no drift. |
| `python scripts/ci/check_documentation.py` | Status, coordinate, index and local-link checks pass. |
| `python scripts/ci/build_execution_inventory.py` then `--check` | 2,331 current/generated/planned records; no drift. |

Focused test log SHA-256:
`eb3c03bc9e4b81293d7f9e5362391f84dfa319d8e9acff347dc1000d4b713f54`.
Owning-suite log SHA-256:
`4f24075ffb53ddfb76fbdab7af5b32dbff9976131b3abae805ae4851d9fcc3f2`.
Strict-lint log SHA-256:
`25f3a58b8d2a0383169c0c011c4d764fba983a7f5a8dcf87261c79b872ea4e49`.

## Installed delivery and restart

The isolated loopback trial used the newly built engine and the hash-verified
retained RROFLOW receiver. It installed an engram over a copy of an actual SWAG
document, explicitly configured export, prepared a real engine attunement-job
observation, delivered it and read both sides. After abruptly terminating and
reopening both owned processes, engine readback was identical. Acknowledged
replay preserved exactly one receiver event and one engine delivery attempt.
No active developer service was stopped.

| Identity | SHA-256 |
|---|---|
| Engine executable | `e8a26f99e0f8faed332014a15cae47d011afb5743fb3cea2e80a5d1febd0212e` |
| Receiver executable | `786b6c48e64fc7c5962b1b4dc427af1b06867f8c05a54669e096ebf27a9b82bb` |
| Exact authorized source response | `d559fb42d2ab6c1f2c51f39bf3d1cd43602962ef526e816adbb97eb8027f5830` |
| Exported payload | `b3028d8e56d3361727de47ab4270cbe401ab4c72ee1127790c6ef148e723c443` |
| Acknowledgement | `2f0933de11e27c8e887d3fc4d9dd27078f4ccf5b8cd7a26f7d75d45846247492` |

The tested staged integration tree was
`fae574cedcc3668e40801e865244ea45815cafcc`, before this final evidence journal.
Production sources match that tree. The retained receiver protocol specification
also matches the prior trial's recorded digest; receiver PR43 is merged. Raw
requests, readbacks, copied harness/binaries and result are retained under
`installed-roundtrip-20261102T194547Z`. This proves installed metadata delivery
and recovery, not hosted RROFLOW, a completed project task, model execution or
approved learning data.

## Review, surfaced limits and handoff

Reviewed owners include root instructions and README, the change-authoring
procedure, CI ownership, POAM-008, the complete previous integration journal,
the complete branch attunement delta, current-main recovery implementation,
and overlapping workflow/protocol changes. Whole-file comparison establishes
unchanged-region preservation; it is not a claim of independent review or a
manual reread of unchanged generated text. The final overlapping diff and this
journal are reviewed before publication.

`git diff --cached --check` against the old branch reports CRLF whitespace in
the inherited immutable audit landing journal. That file exactly matches
accepted main and is preserved. The candidate diff against accepted main
passes the whitespace check. This pre-existing formatting is not hidden by
rewriting historical evidence.

No external research or upstream adaptation is needed for this reconciliation.
Existing traces and retained receipts provide diagnostics; no resource campaign
or model run is added. The isolated installed-binary delivery/restart trial
checks the combined source; previous trials keep their original source
identities. No hosted delivery or real development-task completion is
inferred from protocol fixtures or transport receipts.

The checklist remains bound to the pre-edit PR brief, reproduced conflicts,
parent-preservation oracle, focused and owning checks, this immutable journal
and regenerated index. Stop on semantic conflicts, lost parent behavior,
changed assertions, failing checks or a changed remote head. The next step is
exact-candidate and protected CI through the existing merge controls. No
roadmap, POAM, model qualification, complete workflow or release gate closes.

## Links

- [Prior integration evidence](h-05-observation-main-integration.md)
- [Original delivery implementation](h-05-engine-observation-delivery.md)
- [H-05 roadmap owner](../../../roadmap/rrflow-1.0/gate-h.md)

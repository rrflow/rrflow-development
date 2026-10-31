# Controller composition with immutable publication repair

**Status:** active composition validated locally; clean-primary and protected acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-publication-integration`
**Owner:** Gate G/I controller composition; C-03/C-07 / POAM-035 prerequisite; driver: active SWAG session

## Change checklist and source identity

The pre-edit PR206 brief binds ordinary merge of PR228
`a40a4f8cb8f9657f320d6194c09188add6017466` into controller
`c5bd4d99fa84a107ce7c46171f335107977d0d2d`. Resolved pre-journal source tree is
`82065cd6caf18855b6c365a3665ce53eb45e11a1`. Product version remains `1.0.0`.
The [previous integrated failure](controller-inventory-integration.md) is
unchanged; the [owner repair](../gate-c/c-07-object-publication-repair.md)
retains the original failing component oracles and Windows acceptance.

One conflict in POAM-035 was resolved by preserving the original integration
failure and both persistence observation/repair links. The historical paragraph
now says publication behaved that way at the failed source, not currently.
`filesystem.rs` auto-merged only the crash-hook parity change and retains the
controller's native source-observation module and exports. `object.rs` matches
the exact repaired parent. Engine, inference, contract, CLI and Python installed
test sources are unchanged from the controller parent. No history is rewritten.

Reviewed the owning README/AGENTS, objective/Gate G/I and persistence gap,
change-authoring/CI rules, both parent diffs, complete conflicting record,
immutable publication and native source-observation boundaries, and the existing
quantization concurrency oracle. The effect scope is those two existing source
files, imported immutable Gate C journals/index, this Gate G journal/index and
POAM-035. No public contract, installation identity, endpoint/model policy,
authorization or budget change is introduced. New receipts are not fabricated
from prior binaries or provider histories.

## Acceptance evidence

The previously failing `concurrent_builds_with_one_key_publish_one_generation`
passes on the combined source. More importantly, the complete original engine
command now passes: `cargo test -p rrd-engine --all-targets --all-features
--locked --no-fail-fast` reports 353 passed, four existing ignored, across
13 targets. Original failure log remains
`b4ad363794155d2f63d521eb55bac186a570834d1afc38fd886e6e63b17ac609`;
new full-suite log is
`9b41a92c1f4286458dea594902c4b51b298f75c4225ea89380cbc60d057b4942`.

`cargo test -p rrd-contract -p rrd-inference -p rrd-client --all-features --locked`
passes 208 tests with no ignored cases across 37 targets, including the optional
inference feature build. Its log SHA-256 is
`3260d4bc096f4af4e87a29c2f3e447ed7fa0dc1cc3c9322b19c48e622e25cd87`.

Strict all-target/all-feature Clippy for engine, contract, inference and server
passes with `-D warnings`. All seven `rrflow-cli --test installed_lifecycle`
cases pass (18.91 seconds test time). The real planner's
`refresh_sdk_installation_identity --all-features --locked -- --check` passes
without changing the fixture. Formatting and workflow-policy checks pass.
Generated parity retains 71 operations, seven signals and OpenAPI digest
`f1856602eb3b5ba419f600b3be0e9626d62ad533032fe220a2728795bd28048a`.
Documentation/navigation, the 2,159-record execution inventory and whitespace
checks pass; no public projection changed.
Raw exact commands, exit codes, timings,
parents, resolved tree and log hashes are retained in
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/` under
`controller-publication-*` and `validate-controller-publication.py`.

## Limits and next acceptance

This validates composition after a test-first component repair; it does not
establish the exact native operation that caused the historical engine failure.
No new external research, copied algorithm, production tracing, resource claim
or model call is needed for this ordinary merge. Existing bounded logs and
parent/source comparisons provide diagnostics. Component fixtures remain
correctness data and cannot enter reviewed learning datasets.

Keep PR206 draft through remaining qualification. PR228 candidate/native CI and
the inventory/startup dependencies have not landed merely because this local
composition passes. New clean-source primary/SDK evidence, required candidate
and protected CI, actual UI-to-worker completion, correlated RROFLOW signals,
reviewed routing data, model qualification and public distribution remain
separate acceptance. Prior installed BF16 calls and PR227's Inventory/restart
trial stay bound to their original executables. No roadmap gate closes.

# Native SDK manifest and inventory candidate integration

**Status:** active draft; dependency landing and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-sdk-inventory-candidate-integration`
**Owner:** H-04 / POAM-011 / [PR177](https://github.com/rrflow/rrflow-development/pull/177)

Starting SDK head `9d84c346a03a9b39a18673d790f3528897abe90d` passed its
[full candidate](https://github.com/rrflow/rrflow-development/actions/runs/40809207593)
before main changed. The PR brief was extended before merging exact [PR193](https://github.com/rrflow/rrflow-development/pull/193)
candidate `bdb954e48d2eb751b4839ed6efa761cc1e294071`, which contains landed
main `e73ba9287dafba5d8fb8fb6a990fd145ffd01ab3`. PR193 must land before this
PR is ready; the old CI pass belongs only to its original head.

Only the Gate H generated journal index and old generated inventory conflicted.
The latter retains the retirement descriptor and the former is regenerated
from both sides. The current exhaustive inventory remains a checked, ignored
Git-derived projection. The SDK manifest's private-at-creation Windows ACL,
security verification, Unix mode denial, executable resolution, Rust staging
denial and all existing candidate jobs remain. No product authority, public
SDK contract, test threshold or earlier journal is intentionally removed.
Other staged source and generated SDK changes come from the dependency merge.
Read the prior SDK integration journal, PR briefs, native manifest tests and
security code, combined workflow diff and both generated conflicts. Their
native ownership and bounded conformance checks are the trace/oracle; no
external source adaptation or performance claim applies.

On Windows x64, `python -m unittest discover -s scripts/ci -p
test_sdk_private_manifest.py -v` passed four tests with the POSIX-specific
case skipped. `python scripts/ci/test_execution_inventory.py` passed ten,
`python scripts/ci/test_candidate_admission.py` passed 11, and
`python scripts/ci/check_workflow.py` passed. In the combined working tree
with staged tree `92c04853b726c0ce6ed46c577429639556a7c115`, an isolated
`CARGO_TARGET_DIR=target/sdk-integration-193-build`, `CARGO_BUILD_JOBS=2`,
dev/test debug info disabled, incremental disabled and Cargo offline ran
`cargo test --offline --locked -p rrd-client --example sdk_conformance_server
-- --nocapture`; the staging-publication denial test passed. The retained
local build/test log has SHA-256
`581f160266e9f3740aa7c6faa58b74970070fe2d7c9896aee9c4010e198578fd`.

`python scripts/ci/run_sdk_conformance.py` then passed the shared real-daemon
corpus for Rust, TypeScript, Python, Go, Java and .NET on the same Windows
host. All six reported corpus SHA-256
`f7333a11df06930cca1f7b327f28080890e5268348ac82113e6b56915683c131`;
the locally built server executable SHA-256 was
`bdedecd223851beb0eefaea9bf84ba65b32949c480180ef1d2330565ef0fff0c`,
and local corpus log SHA-256
`9eb0598eda4ec48503a5c2b0d0bb8869838fbee5bd590f1b0a644cea4c1b7180`.
The run used the isolated JDK 25.0.4.1+1 and Maven 3.10.0, Node 24.21.0,
the local pnpm 12.14.0 shim and the existing uv/Go/.NET installations. This
is a real local process corpus, not an exact clean final-commit build, hosted
platform or complete provider-bridge evidence. It does not prove every new
operation has a semantic case.

After generating navigation and inventory, `python scripts/ci/check_documentation.py`,
`python scripts/knowledge/render_navigation.py --check`,
`python scripts/ci/build_execution_inventory.py --check` and
`git diff --cached --check` passed. Regenerate inventory after this journal
edit. No new candidate or protected merge-group result exists yet. PR144 remains a
separate dependent audit repair and must integrate the actual landed SDK
revision before its own readiness. No H-04, POA&M or alpha status changes.

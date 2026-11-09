# Verified estate startup without an unused project-source scan

**Status:** active implementation evidence; installed qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-startup-binding-inspection`
**Owner:** OBJ-14 / D-01 / D-02 / D-03 / POAM-006; PR227 driver: active SWAG session

## Baseline, source review and first oracle

[PR227](https://github.com/rrflow/rrflow-development/pull/227) received its bounded
brief before implementation. Its separate worktree starts at PR226 evidence head
`f1a033b741337a9e9d963fbaa7c0950f0afd09c1`; PR224/226 remain dependencies.
No other worktree is modified by this package.

The [prior installed trial](d-03-incremental-installed-trial.md) retained an
unexplained first-start `ProjectBindingMismatch`. A further fresh clone rejected
preview before installation. A native metadata probe on another fresh clone
observed replacement of `.git/config` and other Git metadata changes during
preview; that probe ended with `DeadlineExceeded`. Retrying preview on the
earlier preserved, uninstalled clone succeeded with no before/after metadata
changes. These observations do not identify the exact guard or cause of the
original startup rejection. No process is blamed and no freshness guard relaxed.

Code inspection independently established a deterministic startup defect:
`open_installed_anchored` requested the complete verification report, which scans
project inputs. It never required `attunement_source_current` to be true, yet an
error producing that unused source report prevented opening valid engine state.

The new real-filesystem regression installs valid state, makes only project Git
configuration invalid, then requires full inspection to reject while startup and
job readback succeed. Before production edits it failed at reopen with
`Storage("Git inventory input: InvalidConfiguration")`: zero passed, one failed.
The dedicated cold build compiled successfully in 7m23s. Failure and test hashes
are retained in `startup-first-failure.json`; this is an observed failing oracle,
not an inferred test result.

Reviewed `AGENTS.md`, README ownership, ADR-0002, Gate D, POAM-006, installation
open/inspect and bootstrap predicates, rooted estate layout, serving publication,
source/Git observation and hashing, the native observation representation,
attunement admission/readback helpers, and `RrflowKvInspector` snapshot lifetime.
The explicit source/native probe used the documented
[Windows metadata fields](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_basic_info);
metadata observations are not proof of content equality. No upstream code or
algorithm is adapted into the production change.

Implementation review paths were `crates/authority/rrd-engine/src/engine/`
`installation.rs`, `core.rs`, `clock.rs`, `estate_layout.rs`, `serving.rs`,
`attunement.rs`, `installation/bootstrap.rs`, `installation/inventory.rs`,
`installation/inventory/git_inputs.rs`, `tests/installation.rs`,
`tests/attunement.rs` and the new test below; additionally
`crates/persistence/rrd-lsm/src/filesystem/source.rs`,
`crates/persistence/rrd-store/src/rrflow_kv.rs`, and
`crates/adapters/rrflow-cli/src/main.rs` / `installed.rs`.

## Canonical change and preserved checks

`engine/installation.rs` now has a private `InstalledBindingInspection` and one
`with_installed_binding` helper. It retains the executable/profile, rooted private
layout, locator/installed-record, bootstrap, configuration, credentials/security,
clock, runtime/control, checkpoint and saved attunement checks. Startup consumes
these verified binding facts without scanning unrelated project inputs.

Explicit full inspection performs the source scan and builds the unchanged public
report with its actual digest, freshness flag and evidence order. No source digest
or freshness flag is synthesized for startup. The helper retains the authenticated
read-only physical snapshot through its caller's inspection work and releases it
before startup acquires the writer. No public operation, SDK/schema, configuration,
product version, worker authority or retry behavior changes.

The new `engine/tests/attunement/startup_binding.rs` regression also proves the
original job remains readable and unchanged after a denied Start, restoration of
valid input permits Start, full inspection retains all 13 checks with the actual
source digest, and changed engine-owned bootstrap bytes still deny startup.
Existing source-dependent Start/Resume/Step admission remains unchanged; opening
the engine does not authorize work against stale or unavailable project input.

## Validation and remaining acceptance

The focused regression passes. The first refactor compile rejected reuse of a
JSON-specific error mapper for a contract-ID error; the existing explicit contract
error mapping corrected it, without changing the rejection type. Both logs remain.
The full all-target/all-feature suite passes **342 tests across 13 targets**:
279 library and 63 integration passes, with four existing ignored helpers. This
includes source closure and all 32 architecture checks. Strict all-target/all-feature
Clippy passes with warnings denied. Correctness fixtures are not training or routing data.

Commands use `target/startup-f1a033b7`, dev/test debug information disabled and two
build jobs:

```text
cargo test -p rrd-engine installed_startup_keeps_source_admission_at_the_dependent_operation --locked
cargo test -p rrd-engine --all-targets --all-features --locked --no-fail-fast
cargo clippy -p rrd-engine --all-targets --all-features --locked -- -D warnings
cargo fmt --all -- --check
python scripts/knowledge/render_navigation.py
python scripts/ci/build_execution_inventory.py
python scripts/ci/build_execution_inventory.py --check
python scripts/ci/check_documentation.py
git diff --check
```

Raw logs and native-probe records remain under
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/` with `startup-`,
`native-plan-observation-` and `fresh-native-observation-` prefixes. The original
failed/successful estates and source-identified PR226 binary remain intact.

Changed paths are this installation owner, the attunement test module/new test,
this linked journal, POAM-006 and generated navigation. Navigation, documentation,
the 2,103-record execution inventory, formatting and whitespace checks pass.
Build a source-identified primary and prove unavailable-input
startup plus later admission/restart on a real repository before installed
qualification. Candidate/protected CI and dependency landing remain required.
The original intermittent failure is not claimed fixed; no release, zero-read
Inventory, latency/RSS target or complete controller journey is established.

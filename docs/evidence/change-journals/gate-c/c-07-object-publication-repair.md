# Preserve a verified immutable-object publication winner

**Status:** active component repair validated on Windows; integration and native CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-object-publication-repair`
**Owner:** C-03 / C-07 / POAM-035; driver: active SWAG session; PR228

## Outcome and change checklist

The [prior observation](c-07-object-publication-observation.md) preserves the
original concurrent engine failure and native probe. PR228's pre-implementation
brief bounds this repair to immutable publication and existing crash hooks.
Baseline is public main `f256c8e1337d85fa01f2a0c61e895a4d213163fa`;
opening evidence commit is `33e5477032ce7827377d3b62e91947f8f04475d5`.
Product version stays `1.0.0`. This is a storage prerequisite, not a completed
worker workflow or a cleared original engine failure.

Reviewed owners: README/AGENTS, alpha objective, Gate C, POAM-035,
change-authoring and CI procedures. Implementation review covers
`rrd-store/src/object.rs` in full, anchored filesystem publication/open/sync and
native rename semantics, crash instrumentation, native filesystem tests,
object durability cases, and the failed engine quantization concurrency case.
The final diff is limited to these two canonical source files, this journal,
the Gate C generated index and the POAM-035 link. No moves, deletions, new
dependencies, public contracts or parallel authorities are introduced.

`publish_object_path` now uses anchored non-replacing rename. Only
`AlreadyExists` admits reuse: the actual winner's bytes must match the expected
SHA-256, its parent namespace is synchronized, and only the caller's stage is
removed. A corrupt collision preserves both files and returns `ObjectCorrupt`.
Other I/O errors propagate. Existing quarantine still uses its original rename
semantics because quarantined bytes are intentionally not hash-valid content.
`rename_new_to` retains the same before/error/after crash hooks already provided
by replacing rename, as well as directory synchronization and test tickets.

The replacement trace is confined to the old replacing publication helper;
its object callers pass their already validated digest. Existing put, stream,
verification, receipt and corruption/quarantine tests retain their assertions.
Four new tests cover a reader-held winner, corrupt collision, absent source,
and four simultaneous callers producing one verified address with no stages.
Fixture bytes are correctness data, never training or qualification examples.

## Before and after evidence

Before the production edit, both deterministic tests compiled and failed:
the reader-held winner returned `Access is denied. (os error 5)` and the
corrupt-collision assertion failed because replacing rename succeeded.
The original oracle log SHA-256 is
`20800823dd606d504bfb52e6485a24d032c62213ac6937d9af21fee7890406b6`.
After the edit, both tests pass. Final owner validation on Windows:

- `cargo test -p rrd-store --lib object::tests:: --locked`: 13 passed.
- `cargo test -p rrd-lsm --test filesystem --features crash-test --locked`:
  six passed; production crash-test hooks are compiled.
- `cargo test -p rrd-store --all-targets --all-features --locked --no-fail-fast`:
  257 passed, two existing ignored, across 41 targets; 58.34 seconds including build.
- `cargo clippy -p rrd-store -p rrd-lsm --all-targets --features crash-test
  --locked -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` pass.

The first format check exposed only new-test wrapping; formatting was corrected
before final validation. No runtime assertion or error was weakened. Raw logs,
command exits, timings and exact changed-source SHA-256 values are retained in
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/` under
`object-publication-*` and `validate-object-publication.py`.
The full store log SHA-256 is
`e3c49112ee707fb16fb67cf8a45e7d3adea04746b53106190703cf102dcd027e`.

## Limits and next action

Research uses the primary Windows/Rust rename references linked in the prior
observation. No upstream implementation was copied. Existing fault hooks and
retained failing/passing commands provide diagnostics; no new content logging,
retry loop or global serialization is added. Resource behavior is one bounded
64 KiB digest buffer on collision; no measured memory, latency or throughput
improvement is claimed. Native process-death qualification of this collision
path is not established by compiling hooks or by component tests.

Next: ordinary integration into PR206, rerun its failed full engine suite and
remaining installed checks, then required candidate/protected native CI. The
original engine error's exact operation remains unproven; the deterministic
hazard is repaired, not retroactively declared its established cause. Linux,
macOS, full workload, 24-hour soak and public-distribution qualification are
not run by this package's local checks. No roadmap gate or POAM closes.

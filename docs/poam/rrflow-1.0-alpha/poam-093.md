# POAM-093 — development launcher selects a different engine

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-093`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

The parent ledger owns priority and lifecycle status.

## Observed deficiency

The [bitemporal desktop proof](../../evidence/change-journals/gate-h/bitemporal-public-transactions-20261122.md)
also reproduced Windows refusing replacement of the staged executable while
it was running. The launcher now reuses an identical single-link private copy;
changed builds still require replacement and do not silently select old bytes.
This narrow repair does not close the complete desktop/task-control oracle.

That journey also found the desktop's old SDK contract pin rejecting the newly
exposed query-transaction operation, and Tables incorrectly refusing native
schemaless records without a strict record schema. The paired child candidate
updates both SDK pins and repairs that table eligibility condition. Actual
desktop readback now shows the same record before/after correction and the
persisted relationship; this remains narrower than the full task-control gate.

On 2026-11-20, the documented development launcher at engine revision
`36dccea2c8b47f6e431078fbdbb54165546236ef` inherited the engine Cargo target into
Connectome. Cargo placed both executables under its `debug` directory. With a
canonical engine manifest staged for Connectome, the native resolver correctly
refused the adjacent development `rrflow.exe`: its bytes differed from the
sidecar hash embedded in the app. Putting the private development copy first
on PATH did not override the higher-priority bundled-engine path.

After separating the targets, a desktop installation succeeded but selected
`connectome-target/debug/deps/rrflow.exe`, the process-test fixture. Cargo had
prepended that directory to PATH after the launcher's initial check. The
successful installation therefore did not prove selection of the reported
checkout binary. No fixture was deleted to conceal this mismatch.

The [paired controller journal](../../evidence/change-journals/gate-g/system-one-controller-20261120.md)
records the actual desktop refusal, expected and observed build identities,
launcher/environment source trace and acceptance results. Existing user project
records were displayed but no installation was performed by the failed attempt.

## Impact

The launcher can report a different executable from the one the desktop runs.
This can refuse a valid local start or misattribute successful installation and
readback to the checkout binary. A passing initial PATH check is insufficient.

## Owning gates

H-06 owns the usable local desktop journey.

Roadmap owner: [Gate H](../../roadmap/rrflow-1.0/gate-h.md).

The existing `scripts/dev/connectome_dev.py` is the canonical repair
boundary: give the application a separate target, report that target, refuse
reusing the engine target and bind the exact executable through
`RRFLOW_DEV_ENGINE`. The debug desktop resolves that absolute path and refuses
a conflicting saved setting. Release sidecar verification remains intact.

## Closure evidence

The environment tests must prove distinct application output and exact staged
engine selection despite Cargo's PATH changes; a real desktop launch must preview, install and serve
an isolated project through that engine, then expose its task controls. The
candidate and merge-group gates must pass. A loaded window, browser mock or
compilation alone does not close this gap. The separately observed slow first
Vite load recovered without code changes; this repair does not qualify startup
latency.

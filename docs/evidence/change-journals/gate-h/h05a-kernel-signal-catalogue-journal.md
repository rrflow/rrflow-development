# H-05a kernel signal catalogue journal

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h05a-kernel-signal-catalogue`
**Owner:** H-05a slice-1 implementation receipt; not runtime, roadmap, or release authority

This record proves the bounded kernel signal-catalogue package. The
[engine data-flow architecture](../../../architecture/engine-data-flow.md#metric-instruments-and-cardinality)
owns the accepted signal semantics, the [Gate H record](../../../roadmap/rrflow-1.0/gate-h.md)
owns acceptance, and the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
owns dependency and completion status.

## Package and outcome boundary

The package is `H05a-01-kernel-signal-catalogue-v1`. It advances the first
executable prerequisite for H-05a, OBJ-09, and OBJ-10: one dependency-free,
kernel-owned identity for diagnostic levels, durable trace projections, metric
descriptors, bounded metric attributes, and their deterministic fingerprint.

This is an inert semantic foundation. It does not collect a sample, read a
clock or environment variable, install a subscriber, choose an exporter,
persist diagnostic data, change an engine operation, define a build profile,
project a public contract or SDK, or prove a running engine. H-05a and H-05
remain open.

## Starting state and planning boundary

- Starting revision: `717b79538d6d5242b2756549fd309c8586c8d20f`.
- Starting tree: `c8f4adfbb8c8d302048b15e235b8c3816b007613`.
- Branch: `agent/connectome-temporal-runtime-visualizer`, with a clean starting
  worktree.
- Planning-only commit: `ceb7d88ceac651a312bc1061d74c9afac08a52ca`;
  tree `72ac75cccfa1e37b9ea2d5ea2f5166d087c9cbb1`.
- The incremental-development remote `development` resolves to
  `https://github.com/rrflow/rrflow-development.git`.
- The promotion-only remote `origin` resolves to
  `https://github.com/rrflow/rrflow.git`.
- `Cargo.lock` began at SHA-256
  `316533e7512dbb9029e494386503ca8430b0c69853b3b78ef02319dcbab93a27`
  and was not changed.
- The planning validator accepted the separate planning commit with zero
  post-plan paths before implementation began.

The committed plan permits exactly the kernel export, one new kernel module,
one generated fixture, one contract test, this journal, its generated nearest
index, and the generated Git-backed file inventory. No source, evidence, or
status path outside that envelope is changed.

## Change brief and authority

At baseline, `rrd-core` already owned `TraceBoundary`, `TraceOperation`, and
`TraceAttribute`, including their closed `ALL` catalogues, but no executable
owner joined those identities to the architecture's four diagnostic levels,
twenty-two metrics, nine low-cardinality metric attributes, or default 2,000
point limit. Later build manifests, public projections, runtimes, and SDKs
could therefore duplicate or drift signal semantics.

The target is one strictly validated v1 catalogue whose trace descriptors are
derived from the existing trace owners, whose metric descriptors exactly
match the architecture, and whose SHA-256 is computed from a domain-separated,
field-tagged, length-delimited byte preimage. Invalid or noncanonical
catalogues must not acquire a trusted fingerprint. Existing runtime traces,
engine execution, persistence, public operations, and outward contracts remain
unchanged.

Work stops if this boundary requires a normal dependency beyond `serde`,
copies the trace vocabulary, admits dynamic/content labels, reads runtime
state, causes an external effect, silently regenerates a fixture, changes a
version or roadmap state, or expands into a later H-05 package.

## Complete baseline and owner reads

The implementation plan bound complete line coverage and SHA-256 digests for
the following files, and each was read completely before implementation:

- `README.md` (238 lines), root `Cargo.toml` (44), and
  `rust-toolchain.toml` (4);
- `docs/objectives/rrflow-1.0-alpha.md` (65),
  `docs/roadmap/rrflow-1.0.md` (165),
  `docs/roadmap/rrflow-1.0/gate-h.md` (74),
  `docs/poam/rrflow-1.0-alpha.md` (43), and
  `docs/poam/rrflow-1.0-alpha/poam-026.md` (45);
- `docs/architecture/engine-data-flow.md` (1,416),
  `docs/research/rrflow-build-debug-trace-optimize-architecture-research.md`
  (833), and
  `docs/roadmap/rrflow-1.0-execution/change-authoring.md` (145);
- the superseded active-change record (384),
  `scripts/knowledge/render_navigation.py` (351), and
  `scripts/ci/build_execution_inventory.py` (1,841);
- `crates/kernel/rrd-core/Cargo.toml` (15), `src/lib.rs` (87),
  `src/digest.rs` (202), `src/error.rs` (61), and `src/trace.rs` (2,537);
  and
- `crates/kernel/rrd-core/fixtures/runtime-trace-v1.json` (108),
  `tests/golden.rs` (157), and `tests/trace_golden.rs` (15).

The final `telemetry.rs`, contract test, kernel export, and generated fixture
were then reread completely after formatting and fixture regeneration. The
complete diff is reviewed again after evidence generation.

## Methodical benchmark diagnostic supplied during the package

The failing benchmark payload supplied during implementation was treated as a
separate diagnostic, not as authority to weaken a gate or edit unrelated
storage code. The current and historical harnesses and the complete active
write path were read before classifying it:

- current `crates/persistence/rrd-store/examples/engine_benchmark.rs` (818
  lines), benchmark contract (239), historical promotion record (247), and
  benchmark-evidence test (58);
- the historical benchmark source at frozen official revision
  `8406e7114b7f7887e9f7ac4387df94184638eeca` (1,027 lines); and
- current `rrflow_kv.rs` (1,758), claim repository (342), store transaction
  port (128), store engine (540), LSM database (2,157), WAL (557), LSM
  transaction (251), memtable (529), and batch (253).

The exact official scheduled run `34846119226` at that frozen revision failed
only sustained native write throughput: the native/Fjall ratio was
`0.7269154837184973`. Native write batches had 44--126 ms extreme tails in
several trials while reads, recovery, RSS, disk, semantic sequence, and
correctness passed. The same exact revision passed in official run
`34123173881`, and the current development revision
`65e679ef2852447d5ebbbd7ed0220eedf2372f63` passed format-5 run
`34845805483` with nine substantially stable sustained trials, aggregate
148,354 native writes/s, and zero automatic flush, write-stall, failed-flush,
or oversized-batch counters.

The current path is `RrflowKvStore` through the claim repository transaction,
database commit, WAL append plus `sync_data`, and memtable application; the
reported workload does not reach segment or compaction work. Host-dependent
durable-sync tail variance is therefore the leading explanation for the old
run, but the available hosted evidence does not prove a hardware-level root
cause. It is a real historical red diagnostic, not evidence of a current-code
regression and not fixed-hardware release evidence. No benchmark threshold,
storage implementation, roadmap state, or POA&M row changes in this package.

## Paths and implemented behavior

- Added `crates/kernel/rrd-core/src/telemetry.rs` as the sole semantic owner
  for contract version 1, diagnostic levels `off`, `normal`, `detailed`, and
  `profile`, the default 2,000 point cardinality policy, nine metric
  attributes, twenty-two metric instruments, descriptors, validation, and
  fingerprinting.
- Derived all durable trace-operation and trace-attribute descriptors by
  iterating `TraceOperation::ALL` and `TraceAttribute::ALL`; no trace name is
  copied into a telemetry-owned list.
- Added exact metric kind, UCUM/OpenTelemetry unit, description, and canonically
  ordered allowed-dimension metadata. Runtime points may use only an applicable
  subset, and later instrumenting sites must bind values to closed catalogues;
  the static catalogue does not attempt to schema private reasoning or content.
- Added validation that rejects version, default, order, count, owner,
  boundary, descriptor, kind, unit, description, or dimension drift. Public
  fingerprinting first validates and refuses invalid projections.
- Added a deterministic SHA-256 over a versioned, domain-separated,
  field-tagged, length-delimited preimage using the existing dependency-free
  `rrd_core::digest` implementation.
- Added and re-exported only the canonical telemetry types, constants, and
  constructors from the existing `rrd-core` boundary.
- Added the generated canonical JSON projection. Its final catalogue
  fingerprint is
  `239df2ced5974d4351ca01565369646964cb65c63d0fbae01dddccb4c3ac5a07`;
  its file SHA-256 is
  `109bd9f34d2114d16118ed79f3e938e545bc59ba123515baa3897210edb16c20`.
- Added independent integration assertions for all names, kinds, units,
  canonical orders, trace-owner derivation, golden bytes, public digest parity,
  unknown fields, missing/reordered descriptors, wrong trace boundary/name,
  wrong metric instrument/kind/unit, duplicate/dynamic dimensions, and absence
  of effect/content authority.
- Added an internal preimage-sensitivity test that changes every semantic
  section independently and requires the raw digest to change.
- No implementation was deleted, moved, merged, renamed, or rewritten, so the
  implementation traceability procedure is not applicable.

## Research and adaptation decision

Research was required and completed in the preceding linked research package.
This implementation retained the current OpenTelemetry metric data-model/API
semantics for stable kind/unit/descriptor identity, explicit cardinality and
failure isolation from the Metrics SDK and error-handling specifications, and
Prometheus's bounded-instrumentation practice. It adapts semantics only: no
upstream source tree, SDK, exporter, public compatibility surface, runtime
topology, or external dependency was imported.

## Trace, resource, and debugging decision

- Runtime trace: **preserved**. Existing durable operations, attributes,
  events, propagation, call sites, clocks, and persistence are untouched; the
  catalogue reads their owners only.
- Resource evidence: **added as identity only**. The catalogue names the
  bounded metrics later runtimes must implement, but records no sample and
  makes no throughput, latency, allocation, RSS, or overhead claim.
- Debugging evidence: **added as identity only**. The four closed diagnostic
  levels and fingerprint can later enter build/capture identities; no level is
  activated and no debug-only semantics exist.

## Test-first oracle and surfaced failures

The contract test was authored before the implementation. The first focused
run failed as intended because the fixture did not exist and the public
telemetry imports were unresolved. This proved that the test could not pass on
the baseline inventory.

The following additional failures were surfaced and corrected rather than
hidden:

1. `python3 scripts/ci/check_change_plan.py` rejected an intentionally partial
   state because six declared result paths did not yet exist. That was the
   correct repository-effect response during the test-first stage.
2. After initial implementation, the strengthened canonical-order assertion
   rejected `rrflow.telemetry.dropped` because `rrflow.queue.kind` preceded
   `rrflow.work.kind`. The source descriptor was corrected to the one canonical
   `MetricAttribute::ALL` order.
3. The normal golden comparison then rejected the stale intermediate
   fingerprint. The guarded generation mode was rerun, the complete fixture
   was inspected, and a subsequent normal test proved it did not rewrite the
   file.
4. The first workspace-architecture run passed 27 of 28 assertions but
   correctly rejected the new Cargo test target as untracked. After the four
   reviewed source/fixture/test paths were staged into the Git index, the same
   command passed all 28 assertions. The guard was satisfied, not bypassed.
5. The historical benchmark diagnostic and its bounded classification are
   recorded above. It did not justify a code edit or a false current
   deficiency.

## Acceptance evidence

- Guarded fixture generation,
  `TELEMETRY_GOLDEN_WRITE=1 cargo test -p rrd-core --test telemetry_contract --locked`,
  passed all 4 tests.
- Normal `cargo test -p rrd-core --test telemetry_contract --locked` passed all
  4 tests and left the fixture file digest unchanged.
- `cargo test -p rrd-core --locked` passed the complete kernel suite, including
  52 unit tests and all integration and golden tests.
- `cargo clippy -p rrd-core --all-targets --locked -- -D warnings` passed.
- `cargo test -p rrd-core -p rrd-lsm -p rrd-store -p rrd-maintenance --all-targets --locked`
  passed all kernel, LSM recovery/fault/transaction/compaction, store
  durability/parity/soak, and maintenance targets.
- `cargo clippy -p rrd-core -p rrd-lsm -p rrd-store -p rrd-maintenance --all-targets --locked -- -D warnings`
  passed with warnings denied.
- `cargo test -p rrd-engine --test workspace_architecture --locked` passed all
  28 tests after the tracked-target condition described above was satisfied.
- The exact normal-dependency guard based on
  `cargo tree --locked -p rrd-core --edges normal --depth 1` passed: `serde`
  remains the sole normal dependency.
- `cargo fmt --all -- --check` and `git diff --check` passed for the source
  implementation.

- `python3 scripts/ci/check_change_plan.py` passed package
  `H05a-01-kernel-signal-catalogue-v1` at planning commit
  `ceb7d88ceac651a312bc1061d74c9afac08a52ca` with exactly 7 post-plan
  paths.
- `python3 scripts/ci/check_documentation.py` passed knowledge ownership, 260
  document statuses, 258 classified coordinates, parent indexes, and local
  links.
- `python3 scripts/knowledge/render_navigation.py --check` passed with 13
  generated indexes, 259 nodes, 1,078 edges, and zero files requiring update;
  all 6 navigation tests passed.
- `python3 scripts/ci/build_execution_inventory.py --check` passed with 1,131
  current, generated, and planned path records.
- All 11 knowledge-export tests passed.
- `python3 scripts/check_version.py` passed and retained release train
  `1.0.0`.
- Final Cargo formatting and staged/unstaged diff-integrity checks passed.

## Checks not run

No public contract/OpenAPI/SDK projection, runtime signal collection, W3C
propagation, exporter, diagnostic capture, diagnostic/release binary parity,
runtime overhead, fuzz, Miri, Loom, sanitizer, native profiler, fixed-hardware
benchmark, installation, daemon, deployment, offline bundle, signing,
provenance, or platform qualification test was run. Those behaviors are not
implemented by this slice and cannot be inferred from a static catalogue.

## Remaining work and status

The next separately planned H-05a slices must mechanically project this owner
through `rrd-contract`, OpenAPI/internal bindings/language SDK models and
conformance; bind its digest into release-derived build profiles and a build
manifest; expose machine-readable version/build identity with release and
diagnostic parity; and implement runtime level activation with measured
overhead. Later H-05 packages own actual instrumentation, propagation,
bounded export/capture, fault evidence, profiling, and end-to-end engine proof.

There is no remaining known error inside the bounded static catalogue after
its declared acceptance commands pass. That statement is not an engine,
observability, release, or performance completion claim.

No roadmap checkbox, objective outcome, POA&M status, product maturity,
version, release state, or promotion state changes. The final result commit is
pending at the time this immutable receipt is authored. No push or promotion
is authorized or performed by this package.

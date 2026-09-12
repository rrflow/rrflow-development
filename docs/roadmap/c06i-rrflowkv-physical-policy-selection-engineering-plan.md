# C-06i rrflowKV physical-policy selection engineering plan

**Status:** active retained-policy integration; candidate screen complete
**Coordinate:** `rrflow://rrflow-instance/data/work-package/c-06i-rrflowkv-physical-policy-selection`
**Owner:** C-06i implementation order, measurement rules, and retained-policy handoff

This record explains the machine-enforced
[`rrflow-1.0-active-change.json`](rrflow-1.0-active-change.json) package. The
canonical [RRFlow 1.0 roadmap](rrflow-1.0.md) owns gate status. This record
cannot mark C-06 complete.

## Outcome

C-06i must answer four questions with RRFlow measurements before changing the
durable format:

1. Which v4 page kinds and semantic families benefit from adaptive LZ4 or
   Zstandard, including encode/decode CPU and the cost of leaving a page raw?
2. Does a serialized row-group Bloom filter remove normal-reopen point-miss
   reads without false negatives and with an acceptable false-positive and
   memory cost?
3. Does a scan-resistant cache admission policy preserve more hot pages than
   the current exact-byte LRU under the declared mixed trace while retaining
   exact byte accounting?
4. Does value separation deserve a real implementation package, after
   accounting for pointer bytes, indirection, recovery, snapshots, range reads,
   and value-log garbage collection rather than compaction bytes alone?

The candidate screen answers what should advance. A second, separately bound
package must integrate only accepted candidates into a new explicit segment
format and rerun correctness, recovery, snapshot, garbage-collection, and
mixed-family regression evidence. Rejected candidates leave no production
option, dependency, compatibility path, or dormant runtime.

## Fixed-machine candidate result

The retained run is bound to clean implementation revision
`f7257fa45d85f9a06db739e60e9eb623608ca1b4`, tree
`4ceb46394dfa3547e9d52b0a10a4afb5a2209208`, Cargo.lock SHA-256
`316533e7512dbb9029e494386503ca8430b0c69853b3b78ef02319dcbab93a27`,
and the executable digest recorded in the
[raw evidence](../evidence/c06i-rrflowkv-physical-policy-linux-x86_64.json).
One warm-up and all three retained children exited zero and produced the same
corpus digest, counts, deterministic bytes, filter/cache/value observations,
reopen counters, and candidate decisions.

| Observation | Fixed-corpus result | Consequence |
|---|---:|---|
| corpus | 16,384 mutations; 8,192 keys; 139 row groups; 834 pages | sufficient for this candidate screen only |
| adaptive LZ4 | 1,426,554 bytes; 84.12% saving; 677 selected pages | advance to separately planned integrated hot-page placement |
| adaptive Zstandard level 1 | 1,226,437 bytes; 86.35% saving; 677 selected pages | advance to separately planned cold/level placement; not a hot-path default |
| serialized Bloom | 13,304 bytes; zero false negatives; 0.634% observed false positives | first retained-policy implementation candidate |
| normal reopened rrflowKV | 16,234 filter checks; zero filter negatives; 426 page loads | confirms the current persisted-filter gap |
| segmented LRU model | 96/96 post-scan hot hits versus 84/96 for current LRU | advance only to an integrated concurrency/lifetime trial |
| separated values | 8,082,816 modeled bytes avoided | reject for production from this slice |

Across retained children, the complete screen processed 33,394–34,180
mutations/s and the integrated reopen workload processed 891,381–896,144
point misses/s. These are source-bound diagnostics on one loaded host with
uncontrolled device cache, not release, cross-platform, end-to-end reasoning,
or competitor evidence.

The next package must implement the authenticated persisted row-group filter
as the smallest production slice and measure actual miss-I/O reduction without
changing semantic reads, MVCC, recovery, snapshots, or garbage collection.
LZ4/Zstandard placement and segmented-LRU replacement remain later, separately
bound integrated trials. No value-log work advances from this result.

## Verified baseline

At baseline `0dac17d21b39c9f50f23911d9be71cf23a8a6234`:

- segment v4 contains six 64-byte-aligned Arrow-layout pages per row group;
- every page descriptor requires encoding `plain` and compression `none`;
- a standalone `Segment::open` reads every semantic page and builds a
  process-local ten-bit/seven-hash Bloom filter;
- normal `Database::open` authenticates manifest-owned segments without
  semantic page reads and installs `allow_all` filters, so absent point reads
  inside a row-group range cannot be rejected by a persisted filter;
- the shared page cache is one exact-byte global LRU;
- values are inline in the immutable value-data page; and
- the existing semantic and AI hot-set programs do not provide the complete
  clean-revision and fixed-machine provenance needed to select these policies.

The first widening check also proved that `rrd-lsm` was absent from the
repository's exhaustive optional-feature CI matrix. Planning amendment
`c169f6b204fbcbcb81ef635d5e8e94a377f44b8b` binds that discovered path before
the workflow changes: the laboratory must pass locked all-target tests and
strict Clippy with all features under the existing bounded runner profile.

The current bytes are a coherent correctness baseline, not a tuned physical
policy and not proof of Arrow/DataFusion streaming.

## Source boundaries

| Path | Responsibility in this slice | Explicit exclusion |
|---|---|---|
| `rrd-lsm/src/segment/physical_policy_lab.rs` | deterministic corpus, real v4 page extraction, candidate mechanics, real reopen characterization, exact oracles | no production write/read policy |
| `rrd-lsm/examples/rrflowkv_physical_policy.rs` | release-profile child isolation, provenance, raw trials, aggregates, JSON publication | no engine or lifecycle authority |
| `rrd-lsm/Cargo.toml` | default-disabled lab feature and exact measurement-only codec dependencies | codecs absent from default/runtime closure |
| `.github/workflows/ci-reusable.yml` | route `rrd-lsm` through the existing optional-feature qualification matrix | no second workflow, broader runner, or default feature |
| `rrflowkv-current-format.md` | truthful current bytes and measured gap | no future format declared current |
| `rrflowkv-benchmark-harness.md` | reproduction and evidence eligibility | no release or competitor claim |
| C-06i evidence JSON | exact host/revision/trial output | no roadmap authority |

The laboratory is a child of the canonical `segment` module. It calls the real
`Memtable` and v4 encoder and consumes the encoder's private parsed page
descriptors. It does not reimplement the on-disk parser or infer page offsets.

## Deterministic corpus

One seed generates ordered keys and version histories for the eight semantic
families already used by rrflowKV qualification:

| Family | Payload shape exercised |
|---|---|
| `audit` | structured repeated evidence text |
| `edge/in` | inbound temporal pointer fields |
| `edge/out` | outbound temporal pointer fields |
| `record` | structured project/domain records |
| `runtime` | compact mutable state fields |
| `scalar` | fixed-width ordered values |
| `term` | integer/posting-like values |
| `vector` | deterministic entropy-like embedding bytes |

The corpus digest includes every operation tag, key length/key, value
length/value, and tombstone. It is computed before either the encoder or
database consumes the mutations. Every retained child must report the same
digest, row/page counts, physical bytes, filter/cache/value observations, and
candidate verdicts as the warm-up child. Timing and peak RSS may vary and are
retained rather than normalized away.

## Measurement matrix

| Candidate | Input | Required observations | Advancement threshold |
|---|---|---|---|
| no compression | every exact v4 page body | copy encode/decode time and stored bytes | comparison baseline only |
| adaptive LZ4 | every exact v4 page body | raw compressed bytes, frame cost, selected pages, exact decode, CPU | zero mismatches and at least 12.5% aggregate adaptive saving |
| adaptive Zstandard level 1 | every exact v4 page body | same fields as LZ4 | zero mismatches and at least 12.5% aggregate adaptive saving; hot/cold placement remains a later decision |
| persisted Bloom | exact unique keys per parsed row group | serialized bytes, bits/key, hash count, member/absent probes | zero false negatives and at most 2% observed false positives |
| current LRU | repeated hot pages, complete one-hit scan, repeated hot pages | exact capacity, hits/misses, admissions/evictions, post-scan hot hits | comparison baseline |
| segmented LRU | identical page identities, weights, capacity, and trace | same fields as LRU | exact capacity/identity and more post-scan hot hits than LRU |
| separated values | actual non-null/latest/obsolete corpus bytes | inline values, pointers, append log, modeled avoided compaction bytes | cannot advance from this slice |

Codec selection is per page. The candidate frame cost is counted and an
individual page remains raw unless its stored size clears the same 12.5%
threshold. A high compression ratio cannot hide slower decode cost; raw
encode/decode timing remains in the evidence and the production package must
choose hot/cold placement explicitly.

The persisted filter candidate uses the same ten-bit/seven-hash behavior as
the current process-local filter, serialized and reopened independently for
each real row group. This establishes candidate size and accuracy only. The
production package must place authenticated filter bytes in the explicit new
format and prove corrupt/truncated filters fail closed.

The cache comparison is a deterministic simulator over exact real page sizes.
It is not integrated cache latency. Both policies must return the same page
identity and remain within capacity after every access. Moka is not included:
its best-effort bounds do not replace rrflowKV's exact byte/generation/lifetime
accounting. A higher-level immutable plan/context cache may evaluate it under a
different owner later.

Value separation is deliberately labelled `analytical-model-only` and always
reports `production_retainable=false`. Modeled savings cannot substitute for:

- authenticated pointer and value-log framing;
- atomic WAL, value-log, segment, manifest, and `CURRENT` publication;
- read-view, snapshot, backup, and restore closure;
- live/dead accounting and crash-safe garbage collection;
- point/range and compaction read amplification;
- corruption, torn-tail, lost-acknowledgement, and ENOSPC behavior; or
- mixed small/large family regression evidence.

## Real rrflowKV characterization

Each child also creates a real `Database`, commits the same independent
corpus, flushes segment v4, drops the process-local state, reopens from
`CURRENT`, and executes absent keys that sort inside each semantic-family
range. It verifies every miss and a bounded present/tombstone sample against
the independent corpus and records:

- filter checks and filter negatives;
- page-cache hits, misses, loads, residency, and evictions;
- physical bytes read, decoded, and decompressed;
- immutable segment bytes; and
- point-miss elapsed nanoseconds.

This is the integrated baseline for the production persisted-filter comparison.
The isolated filter result must never be presented as if segment v4 already
uses it.

## Provenance and aggregation

The parent executable must be an optimized release build. One complete child
is a warm-up; each retained trial runs in a fresh child and temporary database.
No child may be retried or discarded. The parent records:

- exact commit/tree/branch and clean/dirty status;
- Cargo.lock and executable SHA-256;
- complete invocation, compiler, target, profile, and Rust flags;
- OS, architecture, kernel, CPU, logical CPU count, total memory;
- filesystem/mount, visible block devices, CPU governor, and load average;
- fixed configuration and corpus digest;
- every warm-up/retained child exit code, checked operation/s and point-miss/s,
  and raw trials plus nearest-rank p50/p95/p99/p99.9 and extrema; and
- uncontrolled device-cache/host-load and single-machine limitations.

Dirty runs require `--allow-dirty`, are diagnostic, and cannot set
`fixed_machine_candidate_screen=true`. Even a clean run is explicitly not
RRFlow 1.0 release evidence because it does not execute the installed,
governed reasoning/recall path.

## Execution order

1. Commit the machine plan alone and validate the direct clean baseline.
2. Implement the feature-gated laboratory and unit oracles.
3. Implement strict release-profile parent/child evidence execution.
4. Run a bounded dirty smoke only to surface implementation errors.
5. If widening exposes an undeclared path, preserve the implementation,
   commit a planning-only amendment, validate it, and only then resume edits.
6. Route `rrd-lsm` through the existing optional-feature matrix and require
   locked all-target tests plus strict all-feature Clippy.
7. Update current-format, benchmark, research, evidence-index, and journal
   records; regenerate the whole-repository inventory.
8. Pass package, strict-Clippy, default-dependency, architecture, workspace,
   documentation, version, generated-surface, and diff checks.
9. Commit the implementation, then run retained fixed-machine trials from that
   exact clean revision.
10. Record the clean evidence and select a new machine-bound production package.

## Commands

```bash
python3 scripts/ci/check_change_plan.py
cargo test -p rrd-lsm --features physical-policy-lab physical_policy_lab --locked -- --nocapture
cargo run --release --locked -p rrd-lsm \
  --features physical-policy-lab \
  --example rrflowkv-physical-policy -- \
  --seed 14592251008053203194 \
  --trials 3 \
  --records-per-family 1024 \
  --versions-per-key 2 \
  --value-bytes 512 \
  --misses 16384 \
  --cache-bytes 1048576 \
  --output docs/evidence/c06i-rrflowkv-physical-policy-linux-x86_64.json
cargo tree -p rrd-lsm --locked -e normal --no-default-features
cargo test -p rrd-lsm --locked
cargo clippy -p rrd-lsm --all-targets --all-features --locked -- -D warnings
python3 scripts/ci/check_workflow.py
```

The default dependency tree must contain neither `lz4_flex` nor `zstd`.

## Stop conditions

Stop and re-plan before production edits if any codec mismatch, filter false
negative, cache identity/capacity violation, corpus drift, real database
mismatch, arithmetic overflow, child failure, or incomplete provenance appears.
Also stop if a candidate requires changing an undeclared durable/public path,
if an optional dependency enters the default runtime, or if one host/result is
being generalized into a release, zero-copy, scale, or competitor claim.

## Primary references and adaptation

- RocksDB compression, Bloom-filter, and block-cache documentation contributes
  measured block-policy and accounting behavior, not its option tree or bytes.
- `lz4_flex` and `zstd-rs` contribute codec implementations only to the
  default-disabled laboratory; their upstream benchmark claims are rejected.
- Moka contributes the scan-pollution/admission question; it is not added.
- WiscKey contributes the value-separation tradeoff and the requirement to
  count indirection/GC; no value-log code is copied.
- DataFusion's benchmark harness contributes raw-run and revision provenance;
  it does not prove rrflowKV or F-01 behavior.

No upstream tree, format, public model, compatibility surface, or runtime is
copied or renamed into RRFlow.

# rrflowKV benchmark harness

**Status:** active implementation reference; current output is diagnostic and is not RRFlow 1.0 release evidence
**Coordinate:** `rrflow://rrflow-instance/data/reference/storage/rrflowkv-benchmark-harness`
**Owner:** current rrflowKV benchmark mechanics, output schemas, and evidence-eligibility boundary

This record describes the checked-in rrflowKV evidence programs. The
[RRFlow 1.0 roadmap](../../roadmap/rrflow-1.0.md) owns release status, and the
[POA&M](../../poam/rrflow-1.0-alpha.md) owns unresolved qualification gaps.
Historical comparison artifacts remain evidence about the code that produced
them; they are not a compatibility requirement or a current acceptance oracle.

## Executable boundaries

| Boundary | Checkout source | Current responsibility |
|---|---|---|
| Semantic storage | [`engine_benchmark.rs`](../../../crates/persistence/rrd-store/examples/engine_benchmark.rs) | Measures authoritative claim append, bounded replay, full-corpus verification, close/reopen recovery, maintenance, RSS, and physical footprint through `RrflowKvStore`. |
| AI storage access | [`ai_hotset_benchmark.rs`](../../../crates/persistence/rrd-store/examples/ai_hotset_benchmark.rs) | Measures hot, cold, missing, historical, and metadata-fan-out access with repeated, structured, entropy-like, and embedding-shaped payloads over the underlying rrflowKV LSM. |
| Physical-policy integration evidence | [`rrflowkv_physical_policy.rs`](../../../crates/persistence/rrd-lsm/examples/rrflowkv_physical_policy.rs) | Compares real segment-v6 none/adaptive-LZ4 output, independently measures laboratory Zstandard over the same logical pages, verifies production persisted row-group Bloom filters and normal reopen I/O, compares exact-byte LRU/segmented-LRU traces, and retains value placement as a model only. Filters and adaptive LZ4 are integrated; Zstandard, cache admission, and value placement remain separately gated. |
| Persistent model oracle | [`rrflow_kv_model_soak.rs`](../../../crates/persistence/rrd-store/tests/rrflow_kv_model_soak.rs) | Compares randomized rrflowKV mutations, snapshots, compaction, and reopen behavior with an independent in-memory model. |
| Retained historical storage provenance | [`benchmark_evidence.rs`](../../../crates/persistence/rrd-store/tests/benchmark_evidence.rs) | Parses the 35 retained rrflowKV/Fjall-era storage artifacts, requires both passing and failing recorded verdicts, and executes no current performance workload. |
| Scheduled diagnostics | [`rrd-lsm-benchmark.yml`](../../../.github/workflows/rrd-lsm-benchmark.yml) | Runs the semantic and AI-access matrices on `ubuntu-latest` and uploads raw per-run artifacts. |

These programs cover physical and semantic storage only. They do not exercise
the complete `RrdEngine` authorization, rrflowQL, graph, BM25, vector, RRF,
context-packet, transport, attunement, or Connectome flow. Passing them is not
end-to-end reasoning or recall evidence.

## Semantic storage protocol

The format-5 semantic program accepts positive `trials`, `operations`,
`batch-size`, `reads`, and `read-width` values. Batch size and read width cannot
exceed the operation count. Every trial uses a fresh directory and an isolated
child process so allocator and process high-water measurements do not leak
across trials.

The write phase appends one claim for each ordinal in `0..operations` using
authoritative batches. After a clean reopen, full verification pages over the
entire semantic sequence and checks exact cardinality and the exact
`payload-{ordinal}` object for every claim. Timed reads then select deterministic
bounded sequence ranges. Maintenance compacts unpinned history, collects
unreachable files, and closes and reopens the store before a second verification
and read pass.

The output retains each raw trial plus the median aggregate in `rrflow_kv`.
Latency aggregates are medians of each trial's percentile, not percentiles over
one combined sample population. A correctness failure terminates the program;
there is no external-engine ratio or promotion verdict.

## AI storage-access protocol

The format-4 AI program accepts one workload and payload profile per run:

| Dimension | Values |
|---|---|
| Workload | `current-hot-hit`, `cold-hit`, `point-miss`, `historical-hot-hit`, `metadata-fanout` |
| Payload | `repeated-byte`, `structured-json`, `deterministic-entropy`, `embedding-f32` |

Setup publishes a cold immutable corpus, flushes it, then overwrites a bounded
hot set in the active memtable. Timed reads use either the current or retained
historical snapshot. Metadata fan-out mixes hot hits, immutable hits, and misses
in each `get_many` request. The program verifies every timed result against its
deterministic expected value.

Embedding-shaped bytes exercise payload size and access locality only. This is
not HNSW, exact-vector, semantic-quality, or RRF evidence.

## Physical-policy integration protocol

The C-06i executable is an optimized, feature-gated developer tool. Its child
builds one deterministic MVCC corpus spanning audit, inbound/outbound edge,
record, runtime, scalar, term, and vector families. It sends that corpus
through the real `Memtable` and segment-v6 encoder, then consumes the encoder's
parsed page descriptors rather than maintaining another format parser.

Every exact page body is round-tripped through no compression, LZ4, and
Zstandard level 1. The result reports raw codec bytes and CPU separately from
an adaptive stored-byte result that counts framing and leaves an individual
page raw unless it saves at least 12.5%. Segment v6 applies that rule through
the production none/adaptive-LZ4 writer and authenticates the selected policy
and each page codec. Zstandard remains laboratory-only; this harness does not
choose hot/cold level placement.

For each parsed row group, the program extracts exact unique keys and probes the
production ten-bit/seven-hash Bloom words parsed from the authenticated v6
index. Zero member false negatives and no more than 2% observed false positives
are required. The integrated characterization creates, flushes, drops, and
normally reopens the same corpus through `Database`; requires nonzero persisted
filter count/bytes and zero semantic-page open operations; then measures absent
point reads before separately verifying bounded present/tombstoned samples.
Filter checks/negatives and miss-path page I/O therefore describe the actual
reopened implementation rather than a parallel candidate codec.

The cache comparison replays identical real page identities and byte weights
through exact-byte LRU and segmented-LRU simulators: repeated hot access, a
complete scan, then repeated hot access. Both must preserve identity and stay
inside exact capacity after every request. The candidate advances only if it
improves post-scan hot hits. This is an admission-policy screen, not integrated
concurrent-cache latency, pinned-file lifetime, or a reason to add Moka.

Value separation always remains `analytical-model-only` in this program. It
reports inline, latest-live, obsolete, pointer, append-log, and modeled
compaction bytes, but `production_retainable` is false until another package
implements and proves pointer framing, publication, recovery, snapshot closure,
range reads, corruption handling, and value-log garbage collection.

The parent requires a release build, executes one warm-up child, and retains
every configured child trial. It rejects corpus or deterministic-observation
drift. Nearest-rank p50/p95/p99/p99.9 plus extrema are aggregates over raw
children; no outlier is dropped. Clean mode records the exact commit/tree,
branch, Cargo.lock and executable SHA-256, compiler, target, command, CPU,
memory, kernel, filesystem/mount, visible devices, CPU governor, and current
load. It also records the actual warm-up and retained-child exit codes and
checked operation/s and point-miss/s distributions. `--allow-dirty` permits
only explicitly ineligible diagnostic output.

The historical candidate artifact is bound to clean revision `f7257fa`. It
selected authenticated persisted row-group filters as the first production
experiment. The separate persisted-filter artifact is bound to clean revision
`07a6bb8` and records the historical v5 integration: 139 filters/11,080 raw filter bytes,
zero member false negatives, 0.634% observed absent-key false positives, zero
semantic-page open work, 16,020 definite negatives from 16,106 in-range miss
checks, and 114 miss-path page loads. The current integration artifact is bound
to clean revision `eb7445e`; it records 50 raw and 784 compressed reopened v6
pages, 1,418,038 stored/8,988,877 logical page bytes, 336,041 bytes decompressed
by the sampled query path, 16,020 filter negatives, 132 page loads, and zero
child failures. Segmented-LRU still requires an integrated trial, Zstandard is
laboratory-only, and value separation remains rejected. The raw artifacts, not
this summary, own their exact host, timings, counters, and digests.

## Lifecycle measurements

Both programs report three explicit physical points:

| Point | Meaning |
|---|---|
| `active` | Writes completed while the store or read snapshot remains open. |
| `reopened` | Clean close/open and complete verification before explicit maintenance. |
| `maintained` | rrflowKV flush/compaction/garbage collection completed, followed by another clean reopen and verification. |

The harness records apparent bytes and, where the platform exposes them,
allocated bytes derived from filesystem block accounting. These values are
diagnostic until a fixed-hardware qualification run binds its environment and
source provenance.

## Provenance still required for release evidence

The semantic-storage and AI-storage JSON schemas record wall-clock time,
architecture, operating-system family, workload configuration, units, raw
trials, aggregates, lifecycle footprints, and rrflowKV physical counters where
applicable. Unlike the C-06i candidate artifact, those older schemas do not yet
bind:

1. the exact clean Git revision, source-tree and lockfile digests, executable
   digest, compiler, target triple, build flags, or complete command;
2. CPU model, memory topology, kernel, filesystem and mount options, storage
   device, power/frequency policy, or competing host load;
3. a fixed dataset/corpus digest, warm-up policy, confidence interval, outlier
   policy, CPU/NUMA affinity, or device-cache state; or
4. the end-to-end governed reasoning/recall workloads and quality metrics
   required by the release gates.

The C-06i artifacts close the listed source/host provenance gaps only for their
candidate, filter, and adaptive-LZ4 integration scopes. They still disclose uncontrolled
device cache and host load, lack cross-platform and complete end-to-end
workloads, and are never release evidence.

The scheduled workflow uses `ubuntu-latest`; its artifacts are useful regression
diagnostics, not fixed-hardware release evidence. The
[historical August record](../../history/rrd-lsm-promotion-benchmark.md) retains
prior comparison provenance without making it current RRFlow behavior.

## Reproduction

Run the semantic profile:

```bash
cargo run --release --locked -p rrd-store --example engine_benchmark -- \
  --trials 9 --operations 2048 --batch-size 64 \
  --reads 1024 --read-width 32 \
  --output target/rrflow-kv-standard-format-5.json
```

Run one AI access profile:

```bash
cargo run --release --locked -p rrd-store --example ai_hotset_benchmark -- \
  --workload metadata-fanout --payload-profile embedding-f32 \
  --trials 5 --cold-keys 8192 --hot-keys 128 --reads 8192 \
  --batch-size 128 --value-bytes 128 --fanout-width 32 \
  --output target/rrflow-kv-ai-metadata-fanout-embedding-f32.json
```

Run the current C-06i fixed-machine adaptive-page-compression integration proof from a
clean implementation revision:

```bash
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
  --output docs/evidence/c06i-rrflowkv-adaptive-page-compression-linux-x86_64.json
```

Use `--allow-dirty` only for implementation smoke runs. Such output records the
dirty paths and is not fixed-machine candidate evidence.

Validate current storage behavior separately from retained evidence:

```bash
cargo test -p rrd-store --test rrflow_kv_model_soak --locked
cargo test -p rrd-store --test durability --locked
cargo test -p rrd-store --test snapshot --locked
cargo test -p rrd-store --test benchmark_evidence --locked
```

These checks keep useful workload and provenance coverage alive. None alone
closes a roadmap performance or end-to-end context-flow gate.

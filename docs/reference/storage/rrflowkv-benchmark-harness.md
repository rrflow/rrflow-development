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
| Persistent model oracle | [`rrflow_kv_model_soak.rs`](../../../crates/persistence/rrd-store/tests/rrflow_kv_model_soak.rs) | Compares randomized rrflowKV mutations, snapshots, compaction, and reopen behavior with an independent in-memory model. |
| Retained historical storage provenance | [`benchmark_evidence.rs`](../../../crates/persistence/rrd-store/tests/benchmark_evidence.rs) | Parses the 35 retained rrflowKV/Fjall-era storage artifacts, requires both passing and failing recorded verdicts, and executes no current performance workload. |
| Scheduled diagnostics | [`rrd-lsm-benchmark.yml`](../../../.github/workflows/rrd-lsm-benchmark.yml) | Runs the semantic and AI-access matrices on `ubuntu-latest` and uploads raw per-run artifacts. |

These programs cover physical and semantic storage only. They do not exercise
the complete `RrdEngine` authorization, RRFlowQL, graph, BM25, vector, RRF,
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

Current JSON records wall-clock time, architecture, operating-system family,
workload configuration, units, raw trials, aggregates, lifecycle footprints,
and rrflowKV physical counters where applicable. It does not yet bind:

1. the exact clean Git revision, source-tree and lockfile digests, executable
   digest, compiler, target triple, build flags, or complete command;
2. CPU model, memory topology, kernel, filesystem and mount options, storage
   device, power/frequency policy, or competing host load;
3. a fixed dataset/corpus digest, warm-up policy, confidence interval, outlier
   policy, CPU/NUMA affinity, or device-cache state; or
4. the end-to-end governed reasoning/recall workloads and quality metrics
   required by the release gates.

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

Validate current storage behavior separately from retained evidence:

```bash
cargo test -p rrd-store --test rrflow_kv_model_soak --locked
cargo test -p rrd-store --test durability --locked
cargo test -p rrd-store --test snapshot --locked
cargo test -p rrd-store --test benchmark_evidence --locked
```

These checks keep useful workload and provenance coverage alive. None alone
closes a roadmap performance or end-to-end context-flow gate.

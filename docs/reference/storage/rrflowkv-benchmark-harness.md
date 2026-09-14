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
| Semantic storage | [`engine_benchmark.rs`](../../../crates/persistence/rrd-store/examples/engine_benchmark.rs) | Measures authoritative claim append, bounded replay, full-corpus verification, close/reopen recovery, maintenance, RSS, and physical footprint through `RrflowKvStore`; opt-in native write diagnostics retain raw batch phases and Linux thread-resource deltas. |
| AI storage access | [`ai_hotset_benchmark.rs`](../../../crates/persistence/rrd-store/examples/ai_hotset_benchmark.rs) | Measures hot, cold, missing, historical, and metadata-fan-out access with repeated, structured, entropy-like, and embedding-shaped payloads over the underlying rrflowKV LSM. |
| Physical-policy integration evidence | [`rrflowkv_physical_policy.rs`](../../../crates/persistence/rrd-lsm/examples/rrflowkv_physical_policy.rs) | Compares real segment-v6 none/adaptive-LZ4 output, independently measures laboratory Zstandard over the same logical pages, verifies production persisted row-group Bloom filters and normal reopen I/O, retains exact-byte cache trace simulation as screening history, and compares production exact versus scope-aware scan-resistant caches on the same persisted eight-family corpus. Filters, adaptive LZ4, and cache admission are integrated; Zstandard and value placement remain non-production. |
| Persistent model oracle | [`rrflow_kv_model_soak.rs`](../../../crates/persistence/rrd-store/tests/rrflow_kv_model_soak.rs) | Compares randomized rrflowKV mutations, snapshots, compaction, and reopen behavior with an independent in-memory model. |
| Retained historical storage provenance | [`benchmark_evidence.rs`](../../../crates/persistence/rrd-store/tests/benchmark_evidence.rs) | Parses the 35 retained rrflowKV/Fjall-era storage artifacts, requires both passing and failing recorded verdicts, and executes no current performance workload. |
| Scheduled diagnostics | [`rrd-lsm-benchmark.yml`](../../../.github/workflows/rrd-lsm-benchmark.yml) | Runs the semantic and AI-access matrices on `ubuntu-latest`, enables native write-phase capture for every semantic profile, and uploads raw per-run artifacts. |

These programs cover physical and semantic storage only. They do not exercise
the complete `RrdEngine` authorization, rrflowQL, graph, BM25, vector, RRF,
context-packet, transport, attunement, or Connectome flow. Passing them is not
end-to-end reasoning or recall evidence.

## Semantic storage protocol

The format-6 semantic program accepts positive `trials`, `operations`,
`batch-size`, `reads`, and `read-width` values plus the bare opt-in
`--write-path-diagnostics` flag. Batch size and read width cannot exceed the
operation count. Every trial uses a fresh directory and an isolated child
process so allocator and process high-water measurements do not leak across
trials. Collection is enabled and its bounded vector is allocated before the
timed write interval; without the flag, the storage path takes no diagnostic
timestamps, thread-resource probes, or sample allocations.

The write phase appends one claim for each ordinal in `0..operations` using
authoritative batches. After a clean reopen, full verification pages over the
entire semantic sequence and checks exact cardinality and the exact
`payload-{ordinal}` object for every claim. Timed reads then select deterministic
bounded sequence ranges. Maintenance compacts unpinned history, collects
unreachable files, and closes and reopens the store before a second verification
and read pass.

The output retains each raw trial plus the median aggregate in `rrflow_kv`.
Ordinary latency aggregates are medians of each trial's percentile, not
percentiles over one combined sample population. All latency records now carry
p99.9 as well as p50/p95/p99 and extrema. Native phase summaries in the
aggregate pool the exact raw records retained under `trials`; the aggregate's
`raw_samples_location: trials` prevents a second multi-megabyte copy. A
correctness or attribution-invariant failure terminates the program; there is
no external-engine ratio or promotion verdict.

### Native batch-write attribution

The opt-in collector lives at the one rrflowKV database boundary and records
accepted owned transaction batches only. The concrete store adds the wait to
acquire its short begin mutex and exclusive commit mutex. The physical record
then separates transaction validation, mutation-vector preparation, batch
encoding, maintenance/preflight, WAL record/checksum work, first-WAL extent
reservation, WAL write, `sync_data`, memtable apply, bookkeeping, and the
enclosing physical total. Every raw record carries mutation count, encoded
payload and WAL-frame bytes, physical sequence range, lower-bound memtable-byte
growth, durability, and maintenance-counter deltas. It never records a key,
value, claim, prompt, model input, project path, or authorization state.

On Linux, each executed phase pairs monotonic wall duration with
`CLOCK_THREAD_CPUTIME_ID`. Each complete physical sample also carries
`RUSAGE_THREAD` deltas for minor and major faults, block input/output operations,
and voluntary and involuntary context switches. Other platforms emit null CPU
and resource fields with `thread_resource_scope: unavailable`; they never
substitute process-wide counters. Wall minus thread CPU is useful off-CPU
evidence, not a standalone diagnosis: a durability call may voluntarily block,
and an involuntary switch can indicate scheduler preemption without explaining
why the batch was runnable.

The semantic append timer encloses both mutex waits, semantic transaction
setup, claim validation, key/JSON construction, the native physical total, and
small harness/timer transitions. `semantic_setup_and_harness_ns` is the exact
saturating remainder after the two mutex waits and native total; it is not
mislabelled as allocator time. Encoded bytes, memtable-byte growth, phase time,
and page faults together screen allocation/page-backing pressure, but this
harness does not count allocator calls or claim an allocator causal result.
Likewise, maintenance is attributed by both its timed phase and exact
per-sample counter deltas rather than inferred from a long batch.

Collection has a versioned contract, an explicit capacity of at
most 65,536 samples, an observed count, and a dropped count. The standard
semantic harness sizes capacity to the exact number of batches and fails if a
sample is absent, reordered, structurally inconsistent, or dropped; measured
sub-phases must fit inside the physical total, and mutex waits plus that total
must fit inside the semantic append. Enabling a second collector before draining
the first is rejected, preventing silent evidence loss.

To estimate measurement overhead, run identical disabled and enabled commands
on the same otherwise controlled host. To diagnose a tail, inspect raw samples
before summaries: `wal_sync` dominance supports a durability-wait hypothesis;
nonzero maintenance deltas support a flush/compaction stall; mutex duration
supports lock wait; faults plus encode/apply growth support memory pressure; and
involuntary context switches plus non-sync off-CPU time support scheduling
interference. Correlation does not authorize an optimization. Changing
durability, batching, maintenance, allocation, or a threshold requires a
separate workload-controlled package with correctness and crash evidence.

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

The C-06j executable is an optimized, feature-gated developer tool. Its child
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

The retained trace comparison replays identical real page identities and byte
weights through exact-byte LRU and segmented-LRU simulators. It remains an
admission-policy screen only. The accepted cache verdict comes from the real
reader: the child creates and flushes the corpus once, reopens the same durable
state independently under exact LRU and scope-aware scan-resistant LRU, reads
one present key per family in two distinct operations, streams the complete
key/value projection beyond cache capacity, then rereads the hot keys. Both
policies must preserve snapshot, manifest, semantic, hot-value, and projected-
row digests and remain inside exact byte capacity. Exact LRU must reload at
least one hot page; scan-resistant LRU must record cross-operation promotions,
same-stream suppression, and strictly fewer post-scan loads. This does not
claim concurrent-cache latency, load coalescing, pinned-file lifetime, or a
reason to add Moka.

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
child failures. The current C-06j artifact is bound to clean revision
`5b1c31d` and SHA-256
`8a008ee33bb50ca197783227cfcfbb4d58945dd2f26dca8cdf12e1804106b9ad`.
Across three zero-exit children, exact LRU reloads 48 hot pages after the scan;
scope-aware scan-resistant LRU reloads zero, records 18,200 same-scope
suppressions, 48 promotions, and 48 protected entries, and stays within the
1,048,576-byte experimental capacity. Durable and semantic digests are equal.
Zstandard is laboratory-only; value separation and family partitioning are
rejected for this format. The raw artifacts, not this summary, own their exact
host, timings, counters, and digests.

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
applicable. Format 6 adds write-phase and calling-thread diagnostic evidence but,
unlike the C-06i candidate artifact, these schemas do not yet
bind:

1. the exact clean Git revision, source-tree and lockfile digests, executable
   digest, compiler, target triple, build flags, or complete command;
2. CPU model, memory topology, kernel, filesystem and mount options, storage
   device, power/frequency policy, or competing host load;
3. a fixed dataset/corpus digest, warm-up policy, confidence interval, outlier
   policy, CPU/NUMA affinity, or device-cache state; or
4. the end-to-end governed reasoning/recall workloads and quality metrics
   required by the release gates.

The C-06i/C-06j artifacts close the listed source/host provenance gaps only for
their candidate, filter, compression, and page-cache integration scopes. They
still disclose uncontrolled
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
  --reads 1024 --read-width 32 --write-path-diagnostics \
  --output target/rrflow-kv-standard-format-6.json
```

Repeat without `--write-path-diagnostics` on the same host and workload to
measure the observer's throughput and tail effect. Neither result is eligible
for a threshold decision unless the fixed-hardware and provenance requirements
above are also satisfied.

Run one AI access profile:

```bash
cargo run --release --locked -p rrd-store --example ai_hotset_benchmark -- \
  --workload metadata-fanout --payload-profile embedding-f32 \
  --trials 5 --cold-keys 8192 --hot-keys 128 --reads 8192 \
  --batch-size 128 --value-bytes 128 --fanout-width 32 \
  --output target/rrflow-kv-ai-metadata-fanout-embedding-f32.json
```

Run the current C-06j fixed-machine physical-policy integration proof from a
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
  --output docs/evidence/c06j-rrflowkv-scan-resistant-cache-linux-x86_64.json
```

Use `--allow-dirty` only for implementation smoke runs. Such output records the
dirty paths and is not fixed-machine candidate evidence.

Validate current storage behavior separately from retained evidence:

```bash
cargo test -p rrd-store --test rrflow_kv_model_soak --locked
cargo test -p rrd-store --test durability --locked
cargo test -p rrd-store --test snapshot --locked
cargo test -p rrd-store --test benchmark_evidence --locked
cargo test -p rrd-lsm --test write_diagnostics --locked
cargo test -p rrd-store --example engine_benchmark --locked
```

These checks keep useful workload and provenance coverage alive. None alone
closes a roadmap performance or end-to-end context-flow gate.

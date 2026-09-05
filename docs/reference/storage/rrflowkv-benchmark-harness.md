# rrflowKV benchmark harness

**Status:** active implementation reference; current output is diagnostic and is not RRFlow 1.0 release evidence
**Coordinate:** `rrflow://rrflow-instance/data/reference/storage/rrflowkv-benchmark-harness`
**Owner:** current `rrd-store` comparison mechanics, output schema, and evidence-eligibility boundary

This record describes what the checked-in benchmark executables actually do.
The [RRFlow 1.0 roadmap](../../roadmap/rrflow-1.0.md) owns release evidence and
the [POA&M](../../poam/rrflow-1.0-alpha.md) keeps fixed-hardware end-to-end
qualification open. The [historical August results](../../history/rrd-lsm-promotion-benchmark.md)
preserve prior local measurements but cannot close those gates.

The word `promotion` still appears in the current executable and JSON schema.
It means only “native met this run's Fjall-relative thresholds.” It does not
mean rrflowKV is promoted for RRFlow 1.0, that Fjall removal is authorized, or
that RRFlow outperforms another database generally. The naming itself remains
subject to the Gate A-07 vocabulary audit.

## Executable boundaries

| Boundary | Checkout source | Current responsibility |
|---|---|---|
| General storage comparison | [`engine_benchmark.rs`](../../../crates/persistence/rrd-store/examples/engine_benchmark.rs) | Generates an append/replay corpus and compares Fjall with the current native store. |
| AI storage-access comparison | [`ai_hotset_benchmark.rs`](../../../crates/persistence/rrd-store/examples/ai_hotset_benchmark.rs) | Exercises selected hot, cold, missing, historical, and metadata-fan-out access shapes. |
| Retained-artifact assertions | [`benchmark_evidence.rs`](../../../crates/persistence/rrd-store/tests/benchmark_evidence.rs) | Checks frozen JSON structure and recorded verdicts; it does not rerun their measurements. |
| Mixed-mutation correctness | [`mixed_storage_soak.rs`](../../../crates/persistence/rrd-store/tests/mixed_storage_soak.rs) | Compares deterministic put/overwrite/delete/reopen/compaction state with independent and Fjall oracles. |
| Scheduled execution | [`rrd-lsm-benchmark.yml`](../../../.github/workflows/rrd-lsm-benchmark.yml) | Runs the general and AI matrices on `ubuntu-latest` and uploads per-run artifacts. |

These are `rrd-store` physical and semantic-storage workloads. They do not
exercise the complete RrdEngine authorization, RRFlowQL, graph, BM25, vector,
RRF, context-packet, transport, attunement, or Connectome flow. A green storage
comparison is therefore neither an end-to-end context result nor a recall
quality result.

## General comparison protocol

The format-4 executable accepts positive `trials`, `operations`, `batch-size`,
`reads`, and `read-width` values. Batch size and read width cannot exceed the
operation count. For each trial it creates fresh per-backend directories and
runs each backend in a separate child process. Backend order alternates by
trial to reduce a fixed first/second ordering bias.

The write corpus appends one claim for every ordinal in `0..operations` in
authoritative batches. Measured reads deterministically select bounded ordinal
ranges. Full verification separately pages over the entire semantic sequence
and checks cardinality and the exact `payload-{ordinal}` object for every
claim. That is a meaningful exactness oracle for this corpus; it does not test
updates, deletes, graph edges, secondary indexes, or concurrent transactions.

Each backend is observed at three lifecycle points:

| Point | Meaning | Cross-backend verdict use |
|---|---|---|
| `active` | Logical writes completed while the engine remains open. | Informational footprint only. |
| `reopened` | Clean close/open, complete verification, then bounded measured reads without explicit maintenance. | Used by the current comparator for recovery, reads, RSS, and allocated footprint. |
| `maintained` | Each backend runs its own declared maintenance actions and reopens. | Diagnostic only because actions and resulting physical shapes differ. |

The parent retains every raw child result and reports medians of per-trial
metrics. Aggregate latency percentiles are medians of each trial's percentile;
they are not percentiles over a single combined sample population. The harness
has no warm-up phase, confidence interval, outlier policy, CPU/NUMA pinning,
frequency control, or device-cache control. Alternation and medians reduce
some noise but do not make the result statistically portable.

## Current comparator verdict

The current `promotion` object fails if correctness fails or native is worse
than Fjall for any of these available measurements:

- write or clean-reopen read throughput;
- write or clean-reopen read p95 latency;
- clean-reopen recovery time;
- clean-reopen process peak RSS; or
- clean-reopen allocated storage footprint.

Maintained results and apparent-byte ratios do not participate. A platform
that does not expose RSS or allocated bytes yields no failure for that missing
cell. Consequently `promotion.passes=true` means only that the implemented
checks found no failing available cell in that run.

## Output schema and provenance gap

Format 4 currently records:

- millisecond wall-clock time, architecture, operating-system family, and
  logical CPU count;
- input counts, aggregation language, lifecycle/footprint language, and the
  verification description;
- every Fjall/native child result, aggregate metrics, ratios, physical native
  counters, and the comparator verdict.

It does not record:

- the executed Git revision or dirty-worktree digest;
- the `Cargo.lock`, source-tree, or benchmark-binary digest;
- the exact command, dataset seed, or corpus digest;
- Rust compiler, target triple, build flags, or dependency identities;
- CPU model, memory topology, kernel, filesystem, mount options, storage
  device, power/frequency policy, or competing host load; or
- CI run identity and immutable runner/hardware identity.

The scheduled workflow uses `ubuntu-latest`. That is remote execution, but it
is not fixed hardware and the checked-in JSON does not bind itself to its
workflow run or source revision. The retained August JSON can be structurally
validated and hashed, but its exact execution environment cannot be recreated
from the artifact alone. It is historical diagnostic data, not admissible
Gate J evidence.

## Concrete artifact example

[`2026-08-23-rrd-lsm-standard-streaming-scan-v4.json`](../../../eval/results/2026-08-23-rrd-lsm-standard-streaming-scan-v4.json)
is a real nine-trial, 2,048-operation format-4 artifact. Its SHA-256 in this
checkout is
`ac30f4ad07534fcf1747b1358123a17ce86be65a96aec9aa9693e2bf98563ac9`.
It records x86-64 Linux, eight logical CPUs, complete-corpus verification, raw
trials, ratios, and a passing comparator verdict. It has no `git_revision`,
`command`, `cpu_model`, or `compiler` field. The retained-artifact test proves
its checked-in structure and numeric assertions; it cannot prove that the same
numbers describe the current source tree.

The historical record links the other retained green and red artifacts. A red
cell remains red; results from different runs are never averaged together to
erase it.

## Evidence required for release use

Before a benchmark artifact can support a roadmap gate, its schema and runner
must bind at least:

1. exact clean source revision, lockfile and harness digests, compiler/target,
   complete command, comparator identity, and input seed/corpus digest;
2. stable hardware and operating environment, including CPU, memory, kernel,
   filesystem, storage device, relevant mount/power settings, and isolation;
3. warm-up and sampling rules, raw trials, measurement units, missing-cell
   handling, and explicit pass/fail policy without discarded failures;
4. exact correctness oracle, lifecycle point, physical-byte method, and
   artifact digest; and
5. for RRFlow release claims, the actual end-to-end governed reasoning/recall
   workloads and quality metrics required by J-02 through J-05—not only this
   storage microbenchmark.

This list is an evidence eligibility contract, not a claim that those changes
already exist.

## Reproduction and validation

Run the current scheduled general profile locally:

```bash
cargo run --release --locked -p rrd-store --example engine_benchmark -- \
  --trials 9 --operations 2048 --batch-size 64 \
  --reads 1024 --read-width 32 \
  --output target/rrflowkv-standard-format-4.json
```

Omitting `--require-promotion` is deliberate for exploratory runs: the JSON
still contains the comparator verdict, while a noisy local machine does not
turn that verdict into a release decision. The scheduled workflow command
exercises the current threshold behavior; it still does not supply immutable
hardware or complete artifact provenance.

Validate the retained diagnostic artifacts and the mixed-mutation oracle:

```bash
cargo test -p rrd-store --test benchmark_evidence
cargo test -p rrd-store --test mixed_storage_soak
```

The first command validates frozen artifact structure and stated historical
cells. The second executes current code against its deterministic oracle.
Neither command supplies the missing fixed-hardware end-to-end Gate J proof.

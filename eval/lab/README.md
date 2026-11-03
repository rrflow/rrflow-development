# RRFlow comparison lab

**Status:** active evaluation tool; results are diagnostic and qualify no release gate
**Coordinate:** `rrflow://rrflow-instance/data/reference/evaluation/comparison-lab`
**Owner:** maintained comparisons of RRFlow against other systems, and the report any lab test writes

The lab shows RRFlow beside other systems on one frozen workload, so that
optimization work starts from a measured gap instead of a guess. It runs one
typed test package against RRFlow, Qdrant, SurrealDB, Milvus, or any custom
system. It belongs to the evaluation boundary of
[J-04](../../docs/roadmap/rrflow-1.0/gate-j.md#qualification-packages-and-optimization-decisions).

A lab result is not J-04 acceptance, and it supports no parity or superiority
claim. Gate J owns those requirements. The report has no winner field: it holds
measurements, declared questions and their answers, and its limits.

## What keeps a comparison honest

| Rule | How it is enforced |
|---|---|
| No system grades itself. | The harness computes exact answers before contacting a system and scores every reply against them. |
| Every system faces the same data. | The corpus uses arithmetic that IEEE 754 fixes exactly, and its digests match across x86 and ARM. `combine` refuses runs with different inputs or a different manifest. |
| Speed counts only at matched quality. | Queries per second at the target recall exists only for error-free cases that met the target declared before running. A system that never met it has no value, and a reader can see that. |
| Failures stay visible. | A system that fails, could not run, or lacks a capability appears with its reason. It cannot have results, and it cannot read as a pass. |
| Each system runs alone. | `run_system.sh` starts one system fresh on the same cores and disk, measures it, then removes it. It refuses to start if a stale server is answering on that port. It also keeps `SYSTEM.host.csv` beside the run, and warns if the GPU was busy. Those samples can show that other work was running, but they cannot prove a host was quiet. |
| Versions are pinned. | Images are pinned by digest in the manifest, and each run records what the server itself reports. |
| Secrets stay out of files. | Manifests name environment variables only, and unknown fields are rejected, so a report can be published with its manifest. |

## Run a comparison

Build in release mode, because a debug build would misrepresent RRFlow:

```bash
cargo build --locked --release -p rrflow-eval --bin rrflow-lab -p rrflow-cli --bin rrflow
export RRFLOW_LAB_BIN=$PWD/target/release/rrflow-lab RRFLOW_BIN=$PWD/target/release/rrflow
export RRFLOW_LAB_CPUSET=5-9,15-19 RRFLOW_LAB_CLIENT_CPUSET=0-4,10-14 TMPDIR=$HOME/lab-tmp
mkdir -p "$TMPDIR"
for system in qdrant milvus surrealdb custom-reference rrflow-exact rrflow-turboquant; do
  eval/lab/run_system.sh "$system" runs/vector-small
done
"$RRFLOW_LAB_BIN" combine --manifest eval/lab/vector-small.json --in runs/vector-small \
  --out runs/vector-small/report.json
```

The core sets above fit an NVIDIA GB10: its ten fast cores serve and the rest
drive the load. Use your host's topology. `TMPDIR` must be on the disk under
test, since a RAM-backed `/tmp` would flatter whatever stores its data there.
Keep the run directory: it holds every latency and recall sample, and the report
binds those files by digest.

Run on a host with nothing else working. On a GB10 the CPU and GPU share memory and
power, and a GPU job roughly halved every system's search speed in one campaign
without showing as CPU load. Repeat a run before trusting a surprising number: a
single run cannot tell a slow system from a busy host.

## Add a system

Any system can join through the [custom protocol](../schemas/lab-custom-protocol-v1.json),
usually behind a thin shim. It is a small HTTP API:

| Route | Purpose |
|---|---|
| `GET /health` | Protocol version, name and version. |
| `POST /prepare` | Drop earlier lab data and create an empty collection. |
| `POST /ingest` | Store a batch of identities, vectors and categories. |
| `POST /finish` | Reply `{"ready": true}` once search reflects every point. It is polled. |
| `POST /search` | Reply with the identities of the nearest points, best first. |
| `POST /teardown` | Remove the lab data. |

[`reference_custom_system.py`](reference_custom_system.py) is a complete
implementation in about 90 lines of standard-library Python. To add a system:
write the shim, add a binding to the manifest with its pinned artifact and
storage profile, and add a start case to `run_system.sh`. Its results then appear
beside the others without any new client code. A system with its own built-in
adapter, like Qdrant, implements `SystemAdapter` in `crates/evaluation/rrflow-eval/src/lab/`
and its responses are captured into `tests/lab_adapters.rs`.

If a system lacks a capability, declare that in the binding's `availability`
with a reason. Do not leave it out, since a gap is a result. RRFlow's own HNSW
arm is declared `unimplemented` this way.

## Add a test

The lab's report is independent of any workload. A test declares its own
dimensions, metrics, verdicts and views in [`lab-report-v1.json`](../schemas/lab-report-v1.json),
so a client renders it from that data, and a new test needs no new components.
Today's test packages are the vector comparisons in `vector-small.json` (20,000 points),
`vector-100k.json` (100,000 points) and `vector-medium.json` (200,000 points). The two larger
tests share a corpus source and also run RRFlow with raised read limits through `engine_settings`,
because its installed defaults refuse a search at those sizes. Graph,
lexical and mixed workloads should add their own manifest and driver, and write
this same report.

Change a manifest deliberately: changing it changes its digest, and runs from
before and after cannot be combined.

## Reading a result

The first measured results and their limits are in the
[package journal](../../docs/evidence/change-journals/gate-j/comparison-lab.md).
Every report carries its own limits, and these apply to all of them today:

- Clients are closed-loop, so results understate latency under overload.
- Ingest is one sequential writer, so it does not show parallel write capacity.
- The corpus is a synthetic mixture, not embeddings of real content.
- Memory compares an engine's resident set with a container's cgroup usage.

When an optimization claims to help, rerun the same manifest on the same host and
compare the two reports. Gate J's rule applies: change one thing at a time, and
reject regressions or explicitly review the tradeoff.

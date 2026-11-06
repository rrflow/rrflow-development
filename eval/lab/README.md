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
for system in qdrant milvus surrealdb custom-reference rrflow-exact rrflow-hnsw rrflow-turboquant; do
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

The lab prints transient JSON progress events on stderr for prepared
collections, started and completed ingest batches, index build, and search
cases. Completed batch counts mean the adapter call returned successfully;
they do not establish durable recovery or search readiness. Progress output is
not an audit record or a completed report. Capture stderr explicitly if it is
needed later, and review server error text before sharing it. Progress output
occurs outside query-case timing, but a slow stderr consumer can delay the
run and affect ingest or index timing and the overall host environment.

Container cleanup uses the ID returned by a successful Docker creation. A
name conflict or invalid response fails without deleting a pre-existing named
container. If Docker creates a container but its response is lost or invalid,
the launcher cannot prove ownership and may leave it for an operator to inspect.

Run on a host with nothing else working. On a GB10 the CPU and GPU share memory and
power, so GPU work can slow a search without showing as CPU load. Repeat a run before
trusting a surprising number: a single run cannot tell a slow system from a busy host.

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
with a reason. Do not leave it out, since a gap is a result. The RRFlow HNSW
arm uses the public index and payload-index operations; its native maximum
level, seed, and build policy are explicit in the manifest. A declared arm is
not evidence of a completed run: inspect its status, actual access path and
quality before drawing conclusions.

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

- A system is `partial` when some requested cases fail or are truncated. The CLI
  exits nonzero, while the run and its raw samples remain available for combine.
  Incomplete cases retain attempts, successes, failures and truncation but do
  not enter the normal recall/speed curve or qualify for matched-quality speed.
  Individual failed-request durations carry a fixed category; the case summary
  holds the first error, which is untrusted server text and needs review before
  sharing. A report is not proof that arbitrary server responses contain no secrets.
- Matching benchmark-client metadata does not establish equivalent server CPU,
  memory, storage, or deployment. The server-resource verdict remains `not_run`
  until these conditions can be independently established.
- Clients are closed-loop, so results understate latency under overload.
- Ingest is one sequential writer, so it does not show parallel write capacity.
- The corpus is a synthetic mixture, not embeddings of real content.
- Memory compares an engine's resident set with a container's cgroup usage.

When an optimization claims to help, rerun the same manifest on the same host and
compare the two reports. Gate J's rule applies: change one thing at a time, and
reject regressions or explicitly review the tradeoff.


## Workbench integration to implement

The existing Connectome vector lens compares two searches on one engine; it
is not this cross-system lab. Its next consumer should read the lab's generated
report schema, show each system and case with failures, limits, input and binary
identities, and render the report's declared table, bars and curves. Unsupported
schema versions or view kinds must be reported explicitly. A selected report is
read-only diagnostic evidence, not an engine lifecycle transition or a parity
claim. Validate artifact sizes, digests and paths before accessing raw samples;
reject path escapes and changed bytes. A changed, partial or absent run remains
visible rather than inheriting a previous success. Tests must import real lab
output as well as exercise tampered artifacts and unknown views.

Launching a comparison is a separate capability: preview the project, binaries
and image digests, workload, endpoint scope, resource limits, storage, output
paths and cleanup before authorization. The selected engine owns authorized
effects and receipts; the workbench displays them. Preserve explicit adapter
selection, custom systems and a no-run/unavailable state. No UI action may
silently start a model, upload data, change a project, or declare a winner. A
later implementation must prove launch, interruption, restart, result readback
and comparison on the installed path; importing a report alone proves none of
those. The current candidate provides neither a Connectome lab report reader
nor this launch capability.

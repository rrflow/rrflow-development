# RRFlow comparison lab

**Status:** active evaluation tool; results are diagnostic and qualify no release gate
**Coordinate:** `rrflow://rrflow-instance/data/reference/evaluation/comparison-lab`
**Owner:** maintained comparisons of RRFlow against other systems, and the report any lab test writes

The lab shows RRFlow beside other systems on one frozen workload, so that
optimization work starts from a measured gap instead of a guess. It runs one
typed test package against RRFlow, Qdrant, SurrealDB, Milvus, Lance, or any custom
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
power, so GPU work can slow a search without showing as CPU load. Repeat a run before
trusting a surprising number: a single run cannot tell a slow system from a busy host.

## Bring something in

| To bring in | How | Example |
|---|---|---|
| A system | A shim that speaks the custom protocol | [`lance_system.py`](lance_system.py) |
| A dataset | [`import_corpus.py`](import_corpus.py), then a manifest pinning its digests | [`vector-coco-i2i.json`](vector-coco-i2i.json) |
| Another harness's results | `rrflow-lab import`, which labels them as imported | VectorDBBench |
| Awareness of what is new | [`sources.json`](sources.json) and [`check_sources.py`](check_sources.py) | Every suite and system here, and the REA list |

Adding a system to a manifest keeps the runs already made, because a report's runs share
the test and not the whole manifest. Changing a system's binding means rerunning that system.

### A dataset

The lab reads real vectors from files named by their own digest, which stay outside the
repository. Install from the locked requirements, convert a dataset from
[ANN-Benchmarks' table](https://github.com/erikbern/ann-benchmarks#data-sets), and point
the lab at the directory:

```bash
python3 -m venv ~/lab-import && ~/lab-import/bin/pip install --require-hashes -r eval/lab/import-requirements.txt
~/lab-import/bin/python eval/lab/import_corpus.py coco-i2i-512-angular.hdf5 ~/lab-corpora
export RRFLOW_LAB_CORPUS_DIR=~/lab-corpora
```

The conversion refuses a file whose published neighbors disagree with an exact search, and
the lab refuses a file that does not match its digest. Check a dataset's terms before using
it. Such a dataset has no filter field, so its categories are assigned from a seed.

### Another harness's results

`run_system.sh` can drive a pinned Qdrant, Milvus or Lance with VectorDBBench, on the same
cores and disk as a lab run. Install it from its locked requirements, run it, then restate its
result file in this lab's report:

```bash
python3 -m venv ~/lab-vdbbench && ~/lab-vdbbench/bin/pip install --require-hashes -r eval/lab/vdbbench-requirements.txt
RRFLOW_LAB_VDBBENCH=~/lab-vdbbench/bin/vectordbbench RRFLOW_LAB_VDBBENCH_CASE=Performance1536D50K \
  eval/lab/run_system.sh qdrant runs/vdbbench
rrflow-lab import --from vectordbbench --result runs/vdbbench/qdrant.ef-128.vdbbench.json \
  --harness-version 2.0.0 --recall-target 0.95 --out imported.json
```

It downloads its own dataset the first time. Set `RRFLOW_LAB_VDBBENCH_EF` to try another search
setting. It embeds Lance in its own process, so that case runs on the harness's cores.

The report is labelled `imported` and keeps the source's version. Only named result fields
are read, so its connection settings stay out. Use it to cross-check a lab test's direction:
the two used different data and clients, so they do not form one ranking.

### What is new

`check_sources.py` reports which outside sources have moved past their pin. It changes
nothing, because adopting a newer version starts a new comparison. `status` says whether a
source is in use, planned, only watched, or left out and why.

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
| `GET /evidence` | Optional. What served the searches, in the system's own words. |

[`reference_custom_system.py`](reference_custom_system.py) is a complete
implementation in about 90 lines of standard-library Python. To add a system:
write the shim, add a binding to the manifest with its pinned artifact and
storage profile, and add a start case to `run_system.sh`. Its results then appear
beside the others without any new client code. Report evidence if you can, because an index
that goes unused looks like a good result. A system with its own built-in
adapter, like Qdrant, implements `SystemAdapter` in `crates/evaluation/rrflow-eval/src/lab/`
and its responses are captured into `tests/lab_adapters.rs`.

If a system lacks a capability, declare that in the binding's `availability`
with a reason. Do not leave it out, since a gap is a result. RRFlow's own HNSW
arm is declared `unimplemented` this way, and so is `rrflow-routed`, which would choose
its capability for each request.

Lance is embedded, so run it from a Python holding the locked requirements, and name that
Python in `RRFLOW_LAB_LANCE_PYTHON`:

```bash
python3 -m venv ~/lab-lance && ~/lab-lance/bin/pip install --require-hashes -r eval/lab/lance-requirements.txt
export RRFLOW_LAB_LANCE_PYTHON=~/lab-lance/bin/python
```

## Add a test

The lab's report is independent of any workload. A test declares its own
dimensions, metrics, verdicts and views in [`lab-report-v1.json`](../schemas/lab-report-v1.json),
so a client renders it from that data, and a new test needs no new components.
Today's test packages are the vector comparisons in `vector-small.json` (20,000 points),
`vector-100k.json` (100,000 points) and `vector-medium.json` (200,000 points), plus
`vector-coco-i2i.json`, which uses 113,287 real image embeddings. All but the smallest also run
RRFlow with raised read limits through `engine_settings`, because its installed defaults refuse
a search at those sizes. Graph,
lexical and mixed workloads should add their own manifest and driver, and write
this same report.

Change a test deliberately: changing its corpus, workload or quality target changes its
digest, and runs from before and after cannot be combined.

## Test whether something helps an agent

Vector tests say how fast a system finds nearest vectors. They say nothing about whether it
helps an agent do real work. `agent-tasks` tests that, with
[`tasks/rrflow-package-planning.json`](tasks/rrflow-package-planning.json) as its first package.

Each task comes from a merged pull request: the behavior its brief asked for, at the revision it
started from, with the files it really changed as an answer the agent never sees.
[`extract_package_tasks.py`](extract_package_tasks.py) builds them. The design is frozen in the
manifest before any run, and a run from a different design cannot join a report. Runs are paired
across arms and compared by exact McNemar. An arm with no repository is the floor: if it solves
tasks, they test what a model remembers.

Nothing runs until a manifest names `models` and a `spend` limit, because those are decisions for
whoever pays. Then:

```bash
rrflow-lab tasks-order --manifest eval/lab/tasks/rrflow-package-planning.json
rrflow-lab tasks-run --manifest FILE --task ID --arm ID --model ID --repetition N   --checkout DIR --empty DIR --out runs/tasks
rrflow-lab tasks-combine --manifest FILE --in runs/tasks --out report.json
```

`--checkout` is the task's repository at its `revision`, and `--empty` is an empty directory.
Real runs use `claude --bare`, which keeps a person's memory and settings out and needs
`ANTHROPIC_API_KEY`. Pilot on a few tasks before freezing a design.

RRFlow cannot yet return a repository's content through its own path, so `rrflow-attuned` is a
declared gap. `rrflow-harness-import` searches text this harness imports through public operations,
and it measures lexical and graph retrieval and nothing more.

[`retrieval_signal.py`](retrieval_signal.py) measures that retrieval with no model: it sends each
task to the real `rrflow-mcp` process and scores the files returned, beside plain BM25 over the same
chunks. The [package journal](../../docs/evidence/change-journals/gate-j/reasoning-tests.md) has its
first results and what they do not show.

## Reading a result

The first measured results and their limits are in the
[package journal](../../docs/evidence/change-journals/gate-j/comparison-lab.md).
Every report carries its own limits, and these apply to all of them today:

- Clients are closed-loop, so results understate latency under overload.
- Ingest is one sequential writer, so it does not show parallel write capacity.
- `vector-small`, `vector-100k` and `vector-medium` use a synthetic mixture. The imported
  test uses real embeddings, but its filter categories are random.
- Memory compares an engine's resident set with a container's cgroup usage.

When an optimization claims to help, rerun the same manifest on the same host and
compare the two reports. Gate J's rule applies: change one thing at a time, and
reject regressions or explicitly review the tradeoff.

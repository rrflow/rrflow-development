# The comparison lab can place every system in a pod

**Status:** active implementation evidence; diagnostic tooling, no gate advanced

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/comparison-lab-pod-20261111`

**Owner:** J-04 in [Gate J](../../../roadmap/rrflow-1.0/gate-j.md); the [comparison lab](../../../../eval/lab/README.md); follows [the comparison lab journal](comparison-lab.md)

**Starting revision:** `development/main` at `f04dbb97`.

**Owner direction, 2026-11-11.** Measure RRFlow beside SurrealDB, ArangoDB and Qdrant actually running, "the correct work to get this setup correctly for long term development". Every system is to be contained, on machines that keep running other projects: "it is not suppose to be quiet".

**Integration driver.** `eval/lab/run_all.sh TEST RUN_DIR` producing a report on a host whose Docker daemon is reachable only through its socket. This package is the first of three: pod placement; a runner container that holds the lab and is the pod; and RRFlow in its own container under the same declared limits.

**Current behavior.** `run_system.sh` publishes each container's ports on the host's loopback and bind-mounts a host folder for its data. So the lab client must run on the Docker host, with Docker access, `taskset` and `python3`. On this Windows workstation, WSL's user cannot reach the Docker socket, because it is not in the `docker` group and sudo needs a password. Container names are fixed, so two runs on one host collide.

**Target behavior.** With `RRFLOW_LAB_POD` set:

- A system container joins that container's network namespace and publishes nothing on the host.
- Its data lives in a named volume the run creates and removes.
- Its name is prefixed with the pod's.

Endpoints stay `127.0.0.1`, so RRFlow's loopback-only client rule (`rrd-client` `connect_local`) and the manifests' `loopback` location are unchanged. Qdrant and SurrealDB listen on their published port, so a pod needs no mapping. Their bindings, and every earlier run, keep their digests. A definition whose container port differs from its published port is refused in a pod, with the reason. Probes measure the exact container and volume the run created.

**Files read in full:**
- `eval/lab/README.md`, `run_system.sh`, `run_all.sh`, `probe.sh`, `sample_host.sh`, `rrflow_credentials.py`, `rrflow_settings.py`;
- `crates/evaluation/rrflow-eval/src/lab/{adapter,driver,rrflow,qdrant,surrealdb,corpus,catalog,manifest}.rs`;
- the system and test definitions;
- `tests/lab_catalog.rs`, as far as its launch tests;
- the `launch-args` and argument-parser parts of `src/bin/rrflow_lab.rs`;
- `docs/roadmap/rrflow-1.0-execution/change-authoring.md`;
- this journal's predecessor.

**Changed paths:**
- `crates/evaluation/rrflow-eval/src/lab/catalog.rs`: `Pod`, the placement parameter of `launch_arguments`, and the same-port check.
- `src/bin/rrflow_lab.rs`: `launch-args --pod --volume`.
- `eval/lab/run_system.sh` and `eval/lab/probe.sh`.
- `eval/lab/systems/{qdrant,surrealdb,surrealdb-4}.json`.
- `tests/lab_catalog.rs` and `tests/lab_automation.rs`.
- `eval/lab/README.md` and this record.

**Research and source decisions.** The servers' own settings were read at their pinned versions: `QDRANT__SERVICE__HTTP_PORT` from Qdrant v1.20.0's `config/config.yaml`, and `SURREAL_BIND` from `start.rs` at SurrealDB v3.3.5 and v4.0.0. A shared network namespace is Docker's `--network container:` mode, the pod pattern. No upstream code is adapted.

**Trace and resource decisions.** No new trace. Probes are bound to the run's own container ID and volume, so concurrent runs cannot measure each other.

**Oracles and results:**
- `cargo test --locked -p rrflow-eval --test lab_catalog --test lab_automation`: 52 passed, including the new `a_pod_member_publishes_nothing_and_keeps_its_data_in_the_run_volume`. `every_checked_in_manifest_is_what_its_files_generate` still passes, so no manifest or binding digest changed.
- `cargo fmt --check` and `cargo clippy -p rrflow-eval --all-targets -- -D warnings` are clean.
- Live, on Windows 11 with Docker Desktop on WSL2 and 24 cores, a holder container was the pod. The pinned Qdrant 1.20.0 joined it with the arguments `rrflow-lab launch-args --pod` produced. A client inside the pod read `{"title":"qdrant - vector search engine","version":"1.20.0",...}` from `127.0.0.1:16333`; nothing answered on the host's `127.0.0.1:16333`; and Docker reported network mode `container:<holder>` with no port bindings.
- The Linux `rrflow-lab` from `f04dbb97`, run inside the pod, completed all 20 cases of `vector-small` against Qdrant with no errors.

**Measurements made on the way.** These are characterization only: one run each, on a host shared with 34 containers from other projects (about 7 GiB, two Postgres at 10–12% CPU), and server limits not set.

| | HNSW recall at the ladder's cheapest 0.95 | p50, 1 client | Searches/s, 8 clients | Cheapest request p50, 8 clients | Ingest |
|---|---|---|---|---|---|
| RRFlow HNSW, `f04dbb97`, native in WSL | 0.978 at ef-32 | 6.59 ms | 308 | 8.15 ms | 649 points/s |
| Qdrant 1.20.0 in the pod | 1.000 at ef-16 | 1.30 ms | 2,749 | 0.41 ms | 14,850 points/s |

The Qdrant row ran while an RRFlow TurboQuant run overlapped it. It shows the pod path works, and a clean repeat follows below.

Both rows support one reading. A filtered RRFlow search now succeeds at recall 1.0, where filtered TurboQuant was refused in November. The cheapest RRFlow request, the version read, costs 8.15 ms at eight clients against Qdrant's 0.41 ms, so a fixed per-request cost dominates. That is the first optimization lead.

RRFlow's other two arms then ran alone on `f04dbb97`, native in WSL. The host sampler saw the GPU in use by other work throughout, at peaks of 29% and 53%, and the P3 package's engine tests were compiling on the same machine. Both arms are `partial`, for the reasons shown:

| RRFlow arm | p50, 1 client | Searches/s, 1 / 8 clients | Filtered | November, GB10 |
|---|---|---|---|---|
| TurboQuant, rerank-40 (recall 0.991) | 69.6 ms | 14.0 / 78.4 | refused: `no vector access path satisfies the request contract` | 67.3 ms; filtered refused |
| Exact scan (recall 1.0) | 1,136 ms | 0.9 / 2.3, truncated at the 300 s case limit | succeeds | 1,411 ms |

TurboQuant has not moved since November. HNSW is now RRFlow's fastest arm, about ten times faster. Every arm's cheapest request costs 6.8 to 8.6 ms at eight clients.

**Surfaced failures and corrections:**

- A throwaway orchestrator meant to rerun Qdrant and SurrealDB in the pod passed a POSIX path to the Windows `rrflow-lab.exe`. The launch arguments came back empty and `docker run` refused, so neither ran. It was not lab code. The runner in the next package replaces it.
- Running RRFlow's arms in WSL while Qdrant ran in the pod made the two overlap. The TurboQuant run was stopped and rerun alone. A clean Qdrant repeat moves to the runner.
- `pkill -f` matched its own command line and ended that WSL session; nothing else was affected.

**Acceptance after the measurements:** `cargo test --locked -p rrflow-eval` passed in full.

**Checks not run:** Linux and macOS CI. A pod run of `run_system.sh` itself, which needs the runner container of the next package. Milvus and Lance in a pod.

**Remaining known errors and owners:**
- Milvus's health port is remapped, so Milvus cannot join a pod yet.
- RRFlow and process systems still run uncontained, with RSS rather than cgroup memory; the third package owns that.
- Server limits are not declared yet.

**Roadmap/POA&M change:** none.

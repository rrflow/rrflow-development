# The comparison lab runs from one command on any Docker host

**Status:** active implementation evidence; diagnostic tooling, no gate advanced

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/comparison-lab-runner-20261111`

**Owner:** J-04 in [Gate J](../../../roadmap/rrflow-1.0/gate-j.md); the [comparison lab](../../../../eval/lab/README.md); follows [pod placement](comparison-lab-pod-20261111.md)

**Starting revision:** `development/main` after #350.

**Integration driver.** `eval/lab/pod.sh TEST RUN_DIR SYSTEM...` producing a combined report on a host whose Docker daemon the shell can reach, with nothing else installed.

**Current behavior.** Pod placement existed, but nothing provided a pod. The lab needed a Linux host with Docker access, `taskset`, `python3`, `curl` and Rust. On this Windows workstation, WSL's user cannot reach the Docker socket.

**Target behavior.** `pod.sh` builds the new `lab` target of `.devcontainer/Containerfile`: the `rust` stage with its pins and checksums, plus the Docker CLI copied from Docker's digest-pinned `docker:29.9.3-cli` image. It starts that target as the pod, with the host's Docker socket, the checkout read-only and the run directory mounted.

Inside, `pod_run.sh` builds `rrflow-lab` and `rrflow` in release mode, then runs each named system alone through `run_system.sh` with `RRFLOW_LAB_POD` set. It records the host's other containers as `SYSTEM.neighbors.csv`, writes the revision and changed-file count the host read, and combines the report.

**Files read in full:**
- `.devcontainer/Containerfile`, `compose.yaml` and `devcontainer.json`;
- `scripts/ci/check_workflow.py`, its development-container check;
- the local-development-container section of `docs/operations/ci.md`;
- the lab scripts and catalog read for the previous package.

**Changed paths:**
- `.devcontainer/Containerfile`: the `lab` stage.
- `eval/lab/pod.sh` and `eval/lab/pod_run.sh`, both new and executable.
- `eval/lab/README.md`.
- `docs/operations/ci.md`.
- This record.

**Oracle and result.** `eval/lab/pod.sh vector-small DIR qdrant surrealdb-4 rrflow-hnsw rrflow-turboquant` ran on Windows 11 with Docker Desktop 29.8.2 on WSL2 (24 cores). It built the image, compiled in release mode in 2 m 53 s with a warm cache, and ran all four systems in one pod.

It wrote `report.json`, with `revision=96b2b05c…` and `changed_files=0`. Qdrant, SurrealDB 4 and RRFlow HNSW completed. RRFlow TurboQuant is `partial`, because RRFlow refuses its filtered searches with `no vector access path satisfies the request contract`, and the report says so.

Before this package the same command could not start, because there was no runner.

**Results.** One run each. The host was shared with 24 other containers, recorded per system. Server limits were not set.

| 20,000 points, 384 dimensions, cheapest setting at recall ≥ 0.95 | p50, 1 client | Searches/s, 8 clients | Cheapest request p50, 8 clients | Ingest | Memory |
|---|---|---|---|---|---|
| Qdrant 1.20.0 | 0.89 ms | 3,872 | 0.22 ms | 16,520/s | 162 MiB (cgroup) |
| RRFlow HNSW, ef-32 / ef-64 | 5.92 ms | 310 | 5.96 ms | 1,099/s | 959 MiB (resident) |
| SurrealDB 4.0.0 | 21.2 ms | 293 | 25.6 ms | 2,069/s | 960 MiB (cgroup) |
| RRFlow TurboQuant, rerank-40, unfiltered only | 59.5 ms | 97 | 5.75 ms | 1,153/s | 667 MiB (resident) |

Qdrant reported "0 of 20000 points indexed", so at this size it searched exhaustively.

RRFlow HNSW's search p50 equals its cheapest request's p50, which is a version read, within 1%. So at one client nearly all of RRFlow's search latency is the fixed cost of a request, not the search. Qdrant's cheapest request costs 0.22 ms. Optimization should start with that cost.

**Surfaced failures and corrections:**

- **`pod.sh` exported `MSYS_NO_PATHCONV`**, which stopped Git for Windows from reading the worktree path. Git now gets the same host path Docker does.
- **No Docker Desktop container on this host resolved outside names**, so the image build's apt step failed. Docker's resolver forwards to the PC's default DNS server, which timed out; `nslookup` on Windows timed out too, while `1.1.1.1` answered from a container. The owner set `"dns"` in Docker Desktop's engine settings, and the build then passed. Nothing in the repository changed for it.

**Checks:** `python scripts/ci/check_workflow.py` reports policy OK. `bash -n` passes for both scripts. Documentation and navigation checks pass.

**Checks not run:** an ARM host, because the `rust` stage installs x86_64 toolchains. Linux and macOS CI.

**Remaining known errors and owners:**
- RRFlow's arms run as processes inside the runner, with resident memory, while the others run in containers with cgroup memory. Server limits are undeclared. The next package contains RRFlow in its own container.
- ArangoDB and the graph and text workloads are not in the lab yet.

**Roadmap/POA&M change:** none.

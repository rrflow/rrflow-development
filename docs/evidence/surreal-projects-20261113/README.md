# SurrealDB multi-project probe, 2026-11-13

**Status:** active evidence; one machine, one run; not a benchmark
**Coordinate:** `rrflow://rrflow-instance/data/evidence/surreal-projects-20261113`
**Owner:** what SurrealDB 3.3.0 isolates between projects, as input to the kernel contract's pod schema (S6)

## Setup

- **Machine:** the Windows 11 laptop, with the owner's other containers running.
- **Servers:** two `surrealdb/surrealdb:v3.3.0` containers on RocksDB, each limited to 4 CPUs and 4 GB.
- **Project A:** namespace `lab` on the shared server, loaded with this repository's corpus by the [planner probe](../planner-probe-20261113/README.md).
- **Project B:** six writer threads inserting chunks (about 2,000 rows per second) into namespace `projb`, once on the shared server and once on the separate server.
- **Script:** [`surreal_projects.py`](surreal_projects.py).
- **Raw results:** [`surreal_projects_results.json`](surreal_projects_results.json).

## Results

Project A ran a mix of 300 queries: a symbol lookup, BM25 with a crate filter, a 1-hop graph walk, and a sort with a limit. Times are server-side.

| Project B writes to | p50 | p95 | worst |
|---|---|---|---|
| nothing (quiet) | 0.95 ms | 1.69 ms | 3.3 ms |
| the same server, its own namespace | 1.12 ms | 6.81 ms | 59.7 ms |
| nothing (quiet, again) | 1.07 ms | 2.97 ms | 19.2 ms |
| a separate server | 1.15 ms | 2.85 ms | 18.2 ms |

A user defined on namespace `projb` read `projb` and was refused (401) on `lab`.

## Reading

- Namespaces isolate data and authentication, not performance.
- A separate process kept project A at its quiet numbers, even with both containers on one host.
- Client round-trip time on this host (about 43 ms per HTTP request) hides these differences, so only the server-side times are meaningful.

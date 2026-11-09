# A comparison lab for RRFlow against other systems

**Status:** active implementation evidence; diagnostic results only, no gate advanced
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/comparison-lab`
**Owner:** OBJ-10 / OBJ-11 / J-04a / J-04d / POAM-053 / POAM-054 / POAM-055 / PR253

## Package

The repository owner directed on 2026-11-03 that RRFlow keep comparisons against
Qdrant, SurrealDB, Milvus and custom systems, "where we can show it again rrflow so
we can use it as lab and optimize". That day the owner also approved tuning from
the results.

- **Starting revision:** `3a72a6b4adc43c842f67a59f4fc91d725eac7580`.
- **Pull request:** [#253](https://github.com/rrflow/rrflow-development/pull/253).
- **Current to target:** RRFlow had no way to compare itself with another system.
  `rrflow-lab` now runs one typed, versioned vector test against each system, grades
  recall with an independent exact oracle, and writes a report that declares its own
  metrics and views.
- **Unchanged:** the engine, its public operations and Connectome.
- **Stop conditions:** any needed engine or public-operation change; a result
  presented as parity or superiority; a comparator run that cannot pin its version.

## Files read

Gate J's J-04 sections, `eval/README.md`, `rrflow-eval`'s storage model and schema
refresh, `docs/reference/vector/{search,collections,hnsw-projection,quantization-lifecycle}.md`,
`rrd-contract`'s vector and transaction types, `rrd-client`'s operations and `rrflowdb`
example, `knowledge-connectors.md`, `connectome.md`, and for the findings below
`versioned_read.rs` (`scan_source`) and the install configuration asset. Comparator
wire formats come from responses captured from their pinned images.

## Changed paths

Created `crates/evaluation/rrflow-eval/src/lab/`, `src/bin/rrflow_lab.rs`,
`tests/lab_adapters.rs`, `tests/lab_contract.rs`, `eval/lab/`, the generated
`eval/schemas/lab-*-v1.json`, this journal, and POAM-053, POAM-054 and POAM-055. Changed
`rrflow-eval`'s `Cargo.toml` and `lib.rs`, `Cargo.lock`, `docs/roadmap/rrflow-1.0/gate-j.md`,
the POA&M ledger and the generated indexes.

## Decisions

- **Research:** comparators are reached only over their public APIs, from pinned
  images, and no comparator source is adapted.
- **Trace, resource, debugging:** each run records the observed version, environment,
  raw latency samples by digest, and memory and disk probes the manifest declares.
- **Isolation:** one system at a time on `clyffy-01`, with server cores 5-9,15-19 and
  harness cores 0-4,10-14. Results are diagnostic, and the report has no winner field. Each run
  now keeps host samples of GPU use, memory and load, because core pinning did not prove a host
  was quiet.

## Results

Each table shows the cheapest setting whose mean recall reached the declared 0.95.
Ingest is one sequential writer. Memory compares an engine's resident set with a
container's cgroup usage, so it is only roughly comparable. Comparators are pinned by digest to the
newest stable release when these ran: Qdrant 1.20.0, Milvus 3.0.5 and SurrealDB 3.3.5.

Runs are from `clyffy-01`, in these directories. Milvus was rerun at every size after the readiness
fix in the corrections below.

**20,000 points of 384 dimensions (`vector-small.json`), `runs/vector-small-2`, report
`0ef72aa58c86f2c86de6ced5325c11ab024cab49e7191611564943aed5571c70`**

| System and setting | Plain, 1 client | Plain, 8 clients | Filtered, 1 client | Filtered, 8 clients | Ingest | Memory / disk MiB |
|---|---|---|---|---|---|---|
| Qdrant 1.20.0, ef-16 | 1.67 ms, 602 qps | 3,014 qps | 1.16 ms, 880 qps | 9,851 qps | 15,456/s | 153 / 467 |
| Milvus 3.0.5, ef-32, filtered ef-16 | 2.29 ms, 443 qps | 6,837 qps | 2.45 ms, 412 qps | 6,392 qps | 8,762/s | 341 / 259 |
| SurrealDB 3.3.5, plain ef-32, filtered ef-16 | 15.0 ms, 67 qps | 478 qps | 19.7 ms, 51 qps | 376 qps | 3,510/s | 869 / 275 |
| RRFlow TurboQuant, rerank-40 | 67.3 ms, 14.9 qps | 71.6 qps | refused | refused | 831/s | 1,600 / 290 |
| RRFlow exact | 1,411 ms, 0.7 qps | 2.5 qps | 1,360 ms, 0.7 qps | 2.5 qps | 855/s | 1,331 / 260 |

**100,000 points of 384 dimensions (`vector-100k.json`), `runs/vector-100k-3`, report
`be7a4895105faf0d1990c54c7841925853b451857bcd49ba7cb504fa197e9583`.** RRFlow's arms have read
limits raised to 16 GiB, on an engine carrying #259's fix, SHA-256 prefix `4c08aeaafe452630`. The GPU
was at 0% in all 373 host samples. TurboQuant's runs came just before host samples existed, in
`runs/vector-100k-2` and `runs/vector-100k-order`, and the two agree with each other within 2%.

| System and setting | Plain, 1 client | Plain, 8 clients | Filtered, 1 client | Filtered, 8 clients | Ingest | Memory / disk MiB |
|---|---|---|---|---|---|---|
| Qdrant 1.20.0, plain ef-32, filtered ef-16 | 1.42 ms, 709 qps | 6,371 qps | 1.70 ms, 588 qps | 3,292 qps | 15,704/s | 214 / 459 |
| Milvus 3.0.5, plain ef-128, filtered ef-32 | 2.75 ms, 365 qps | 4,750 qps | 2.77 ms, 359 qps | 4,762 qps | 8,910/s | 516 / 743 |
| SurrealDB 3.3.5, plain ef-128, filtered ef-32 | 21.6 ms, 46 qps | 324 qps | 36.1 ms, 27 qps | 198 qps | 2,979/s | 2,650 / 1,572 |
| RRFlow TurboQuant, rerank-40 | 313 ms, 3.2 qps | 20.7 qps | refused | refused | 633/s | 1,174 / 1,232 |
| RRFlow exact | 7,470 ms, 0.13 qps | 0.5 qps | 7,250 ms, 0.14 qps | 0.5 qps | 637/s | 4,930 / 1,150 |

TurboQuant reached recall 0.99 at rerank-40 and 1.0 at rerank-100 (317 ms, 17.8 qps at 8 clients).
A second clean TurboQuant run with the settings reversed agreed within 2% at one client. TurboQuant's
memory is from that run, which had no refused searches. The run in the table ended at 6,044 MiB,
because each refused filtered search reads every vector. Without raised limits, both RRFlow arms are
refused at this size. A repeat of Qdrant gave 1.26 ms and 807 qps against 1.42 ms and 709 qps, so
treat a difference of about 15% between two runs as noise.

**200,000 points of 384 dimensions (`vector-medium.json`), `runs/vector-medium-2`, report
`a2f98e68e36c3f05be42f059b0f4dabf461e72f291b1f61d4ef880541cb5cba6`**

| System and setting | Plain, 1 client | Plain, 8 clients | Filtered, 1 client | Filtered, 8 clients | Ingest | Memory / disk MiB |
|---|---|---|---|---|---|---|
| Qdrant 1.20.0, plain ef-64, filtered ef-16 | 1.33 ms, 749 qps | 5,345 qps | 2.38 ms, 430 qps | 1,740 qps | 15,576/s | 240 / 601 |
| Milvus 3.0.5, plain ef-128, filtered ef-32 | 2.94 ms, 339 qps | 2,638 qps | 2.89 ms, 345 qps | 3,679 qps | 8,837/s | 792 / 1,964 |
| SurrealDB 3.3.5, plain ef-256, filtered ef-32 | 32.3 ms, 31 qps | 226 qps | 51.8 ms, 19 qps | 140 qps | 2,676/s | 4,673 / 3,267 |
| RRFlow, every arm | refused | refused | refused | refused | 462-571/s | see below |

These are not a fair ranking of RRFlow's search. No public operation can build its
HNSW, so its only arms are a full exact scan and TurboQuant.

Limits on reading these tables:

- Qdrant reported `0 of 20000 points indexed` in the small test, so it searched
  without HNSW there, and `ef` changed nothing. It reported 93,500 of 100,000 at the middle size.
- RRFlow exact hit the 300-second case limit, so it answered 213 to 752 queries of 1,000 at
  20,000 points and 28 to 142 at 100,000.
- The tables at 20,000 and 200,000 points come from before host samples existed. Memory history
  shows no large model on the host then, which is weaker evidence than samples.
- SurrealDB's mean filtered recall hides individual queries: its worst query scored
  0.00 in the small test and 0.10 at the settings shown for the medium test. Its
  ef-32 mean of 0.9508 has an interval reaching below the 0.95 target.
- The corpus is a synthetic mixture, not embeddings of real content.

## Findings

1. **A default install refuses vector search at 100,000 points**, after accepting
   every write ([POAM-054](../../../poam/rrflow-1.0-alpha/poam-054.md)). Exact search
   and TurboQuant builds both hit the 128 MiB output limit.
2. **Raising that limit exposes a second refusal.** With `projected_output_bytes` and
   `source_page_bytes` at 16 GiB, all four cases and the build were refused at the
   8 GiB `page_logical_bytes` ceiling, with 6.3 GiB resident. The store is 2,271 MiB
   on disk. The cause is the read meter charging a whole page per row, addressed in
   [PR #259](https://github.com/rrflow/rrflow-development/pull/259).
   That fix then reaches a third, a 1,000,000-key public contract ceiling that no setting lifts,
   which still refuses 200,000 points. POAM-054 records it.
3. **Clients cannot build RRFlow's HNSW or payload indexes**
   ([POAM-053](../../../poam/rrflow-1.0-alpha/poam-053.md)). This is also why filtered
   TurboQuant is refused: `no vector access path satisfies the request contract`.
4. **Almost all exact-search time is authentication.** A `perf` profile at 20,000
   points puts 87.0% in `authenticated_steps` and 0.81% in scoring. Its first repair
   is [PR #257](https://github.com/rrflow/rrflow-development/pull/257).
5. **TurboQuant's time is mostly outside the index it searches.** The same profile
   puts 38.53% in `score_mse`, which is scalar on ARM, and about 20% in a
   reference map rebuilt on each query. Neither has been repaired.
6. **A search the engine must refuse first reads every vector**
   ([POAM-055](../../../poam/rrflow-1.0-alpha/poam-055.md)), at 7.3 seconds each with one client
   at 100,000 points. It also left the engine holding about five times the memory.
7. **RRFlow scales about linearly and is far behind at matched recall.** Exact search takes 5.3
   times as long at 5 times the points, and TurboQuant 4.7 times. At 100,000 points, plain
   TurboQuant is about 220 times slower than Qdrant with one client and 310 times with eight.

## Oracle and results

- `cargo test --locked -p rrflow-eval`: see the pull request for the tested revision.
  A wrong-answer adapter scores below recall 1, a failing system reports `failed`
  and stays in the report, unknown manifest fields and credentials are refused, and
  corpus digests are pinned across architectures. A system that never becomes ready
  keeps its ingest rate.
- Engine settings reach the engine: on an installed engine a 262,144-byte limit
  refused at exactly `limit 262144 would be exceeded by 262773`, and a 1 GiB limit
  completed. `runs/settings-proof.txt` holds the result.

## Surfaced failures and corrections

- Editing an installed `.rrflow/config.toml` makes `serve` fail with
  `ProjectBindingMismatch`. The owner was first told the limit could be raised that
  way, which was wrong, and was then corrected. `engine_settings` now seals the
  value through `install plan --configuration`.
- A refused TurboQuant build lost its ingest rate, because the driver recorded ingest
  only after readiness. It now records it first, and a test holds that.
- **The first 100,000-point table was wrong, and the owner was told it.** `runs/vector-100k-1`
  reported 472 ms for TurboQuant, 10,953 ms for exact search, 2.97 ms for Qdrant and 61.6 ms for
  SurrealDB, and from those a gap of 160 to 260 times and exact search growing 8 times at 5 times
  the data. Every comparator had come out about half as fast as at 200,000 points. Memory history
  shows a 24 GiB model loading and unloading on the host from about 08:00 to 12:38 UTC, and CPU use
  stayed low, because a GB10's CPU and GPU share memory and power. A rerun with the GPU at 0% gave
  the table above, which supports that cause. No GPU history exists to prove it. The owner has moved
  that work to `clyffy-02`.
- **The lab measured Milvus before its index served.** A collection created with an index is
  already loaded, so searches ran on growing segments until a background handoff. Recall fell as `ef`
  rose, from 1.0 at ef-16 to 0.95 at ef-64, and differed between client counts. Reloading after the
  index finishes fixed it: recall now rises from 0.69 to 0.997 at 100,000 points and matches across
  client counts, which also explains the fall recorded earlier at 200,000 points.
- **A long run lost its last cases to `SessionExpired`.** A session's absolute expiry is capped at
  an hour and renewing cannot move it, and the run was still marked `completed`. The lab now opens
  a new session every 20 minutes, and a rerun of 3,742 seconds saw no expiry. Warmup now stops after
  20 refusals in a row. The report had not counted the failed cases as meeting the target.
- **Two scripts were committed without the executable bit,** which a hand-made `chmod` on the host
  had hidden.
- The first two rerun attempts stopped before measuring: the tool path was wrong, and
  then a stale tool rejected `engine_settings`. A guard stopped the campaign both
  times, and no result came from them.

## Checks not run and remaining errors

- Not J-04 acceptance: no fixed profile, no real-content corpus, no Pinecone, no
  graph, lexical or mixed workload, and closed-loop clients only.
- Milvus reports no version over its API. Its pinned digest was checked against Docker Hub's
  `v3.0.5` tag.
- The 200,000-point RRFlow numbers do not exist. They wait on the owner's decision about the
  1,000,000-key ceiling.
- What limits RRFlow's throughput under load is undiagnosed.
- One run each, except where a repeat is stated, so small differences are noise.
- Connectome's Workbench view is not built. It waits on design review with these
  results.

## Status change

Three new Open rows in the POA&M. No roadmap or objective status changes.

# C-07g: terminal Windows characterization and database integration

**Status:** active evidence; characterization failed and qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-windows-characterization-terminal`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR151](https://github.com/rrflow/rrflow-development/pull/151)

## Scope and source

This evidence-only continuation starts at `0f6682f9cfa10d0bd6d53b05884f8c705b62e65a`. The tested source remains the separate [PR142](https://github.com/rrflow/rrflow-development/pull/142) candidate `8721f17adc6fd97a05708feb1b3e332508365058`, tree `7ba25be80d4023b41ea8dd4e912f97922ce5a129`. The prior [launch record](c-07g-windows-characterization-launch.md) describes the configuration, earlier cold rejection, executable, host and resource limits. This record reports the later terminal state. No engine code, deadline, profile, or gate status changed.

## Terminal evidence

The warm-only launcher ended with exit 101 after 2444828 ms and confirmed cleanup. Independently streaming and counting `progress.jsonl` yielded 100000 `corpus_intent`, 100000 `corpus_acknowledged` and 100000 `corpus_verified` events, plus two lifecycle intents, acknowledgements and reopen observations. The corpus specifies 100000 records, 500000 edges and 100000 384-dimensional vectors. The workload derives expected values independently from request coordinates. Each forced reopen observation records matching original and replay receipt digests, respectively for one and two checked operations. These are setup and bounded recovery observations, not sustained-load results.

The driver reported `api-resourceexhausted-storage-admission:e7cb85f374dc97137ff380312a5e691db60c208c45ecee954ef29c1de1e2eecd`. The retained classification and digest do not identify the specific admission resource or failing operation. The result has zero segments, zero operating points, and `qualification_passed: false`; no measured segment or 24-hour soak is established. This is an actual failure, not a passing 100000-record capacity test. It is also not evidence of lost acknowledged writes. The retained artifacts are local to this workspace at `target/c07-windows-warm-only-8721/` and are not committed.

Artifact SHA-256: `progress.jsonl` `1fc576c25546c44a534fd90ae92264dc43f0b537c5afde689661694e42e16eb7`; `result.json` `11056baf4d5d6804ff17ce4d3292d3dd37844a1dc3eb225a17a433c5541fb967`; `launch.json` `6a7dc899857d250864bb9e9ed74d0b005993fb30b0dd4b5bd137eb2ffe30e4ec`. Canonical campaign digest is `0d475b7b0c2b16a085042eb6369b176139c19e54eb85c26b1536b13c0e0d98db`.

## Separate real integration oracle

On the exact PR142 source, `cargo test --offline --locked -p rrd-client --test rrflowdb_local installed_native_database_relations_atomicity_temporal_reads_and_reopen` passed: one test, zero failures. It opens a real installed `RrdEngine`, serves HTTP in-process, and exercises the local database's persisted rrflowKV data through rrflowQL, relations, transaction behavior, temporal reads and reopen. This is a small test fixture, not the 100000-record workload, a packaged binary qualification, or proof of the entire product.

## Change checklist and verification

Read the launch journal, POAM-035, execution-map evidence template, installed client test, sustained driver and workload classifier, terminal launcher, progress and result. The existing bound of two GiB and four CPUs was not changed; research and upstream adaptation were unnecessary for this evidence-only update. No implementation was deleted, moved or replaced. Independently counted progress events, checked replay receipt digests and calculated artifact hashes; recorded the exact test command above. The detailed resource cause, full native profile, candidate landing and actual 24-hour soak remain unproven. Commands passed: `python scripts/knowledge/render_navigation.py --check`, `python scripts/ci/build_execution_inventory.py --check`, `python scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py` and `git diff --check`. The first navigation run failed because a Windows UTF-8 BOM obscured the title; an early documentation check also rejected a status lacking the required active marker. Both were corrected, and all checks were rerun. The first inventory check reported drift before regeneration. No roadmap or POA&M status is closed.

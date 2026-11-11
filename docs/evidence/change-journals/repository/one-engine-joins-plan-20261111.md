# One-engine joins plan

**Status:** active planning evidence; no runtime behavior changed

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/one-engine-joins-plan-20261111`

**Owner:** the [one-engine joins engineering plan](../../../roadmap/one-engine-joins-engineering-plan.md), supporting P5 and P6 of the [one-engine execution plan](../../../architecture/one-engine-execution-plan.md)

**Starting revision:** `development/main` at `6df41197`.

**Owner direction, 2026-11-11:**
- RRFlow works as one engine, the way SurrealDB or ArangoDB do, with Qdrant-grade vectors.
- It triggers its parts by the reasoning need, hot first and cold if necessary.
- Every threshold is tunable and measured.
- The owner said the code does not reflect that design and demanded that it be planned, researched and implemented.

**Method.** Five read-only investigations of `development/main`: rrflowQL; vector search and the storage tiers; tracing, metrics and tuning; the capability inventory; and external research on SurrealDB, ArangoDB, Qdrant, ACORN, RRF, Neon, RocksDB and TiFlash. Every code claim in the plan carries a file and line. The plan's second section lists what exists and which joins are missing. Its fourth section orders ten packages, each with a failure oracle and acceptance on the real engine.

**Changed paths:**
- `docs/roadmap/one-engine-joins-engineering-plan.md`, new;
- one paragraph in `docs/architecture/one-engine-execution-plan.md`;
- this record.

**Checks:** `render_navigation.py` and `check_documentation.py`.

**Not established:**
- No package is implemented.
- Tier attribution, the cause of the per-request cost and the index gap are inferred from code reading, plus one lab measurement (RRFlow's cheapest request at 5.96 ms against Qdrant's 0.22 ms) and one diagnostic Parse run (a per-file lookup of 99,936 keys). J1 must measure them.
- Three owner decisions are open: read-audit durability, the rrflowCLD format and the default request.

**Roadmap/POA&M change:** none.

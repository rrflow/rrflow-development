# Archived parser stack integration

**Status:** active integration evidence; project qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/archived-parser-stacks-integration-20261121`
**Owner:** Gate D source Parse and POAM-065 regression coverage

## Before-edit brief

Merge the original parser implementation stacks and restore the oversized-file regression oracle at the current engine boundary.

Driver: original-source integration after PR484. Baseline green main 35a9d6fe7cf714d2373415d0dcdf20c568d82c23; join PR484 before readiness. Owner: Gate D source Parse / OBJ project indexing / POAM-065 physical-batch admission. Sources: b56f23af18fb361a13de46a73ec13a786f8978ed, 7fba8ea5a80d77d5aae04213347170a19eec3e80, 0fe2d60fee16160ef96e0def8a6bd56f85ac9d36, 42accbb224a464714f2a078073bee7c098f96892, f1868ff2c429770295324a58488f067fb052064e. Their ancestor commits carry source spans/chunks, worker protocol v2, engine publication/retry and initial executor tests.

Current behavior: facts and protocol already current, publication uses measured physical batches and current split modules. The old explicit oversized-file refusal test is absent from current parse_step tests; restore its invariant through current helpers, including the small file continuing, bounded completion and persisted refusal. No old executor or host implementation is restored. B56 and 7fb have identical crates trees; their evidence trees differ. Old ParseProposal remains retired. Current partial-frame rejection, native worker identity and platform availability remain enforced.

Read: source spans/chunks patch, current facts/parse/worker contracts, source protocol and host patch, old executor oversized/deletion tests and current parse_step, current step physical measurement/refusal and worker/publish modules. PR482 already records native Parse records/executor readback evidence. Continue inspecting every conflict and relevant source assertions; do not claim every historical line reviewed.

Exact scope: rrd-attunement facts/parse/worker and tests; engine attunement parse/records/publish/step/worker plus parse_step test and installation/evidence consumers; CLI parser/installed test consumers; original related records and linked recovery journal/index. Preserve current LSM transaction admission and source binding. Add one ported oracle, not a second lifecycle. No provider, network, comparison or performance workload.

Smallest acceptance: ported oversized-file test; full rrd-attunement tests; engine parse records/parse_step; CLI parser_worker and installed_engine_plans_applies_serves_discovers_authenticates_and_reopens; architecture, docs/navigation, fmt/diff. Existing unchanged long physical-batch test evidence may be reused only with exact code-tree identity and explicit attribution. Target D:/.usr/Temp/rrflow-root-target, four jobs. No new algorithm/research/instrumentation. Stop on unmapped assertion or failure, fix before candidate readiness; CI and merge queue green, then automatic branch deletion. No full engine/UI/qualification completion claim.

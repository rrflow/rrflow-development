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

## Original merges and canonical destinations

| Original source | Merge | Preservation |
| --- | --- | --- |
| b56f23af18fb361a13de46a73ec13a786f8978ed | 397b3170 | Original staged parser stack joined; measured step/publish and bounded worker retained |
| 7fba8ea5a80d77d5aae04213347170a19eec3e80 | 3f5684ec | Alternate work stack, identical crate files to b56; current evidence retained |
| 0fe2d60fee16160ef96e0def8a6bd56f85ac9d36 | c98ae6f2 | Intermediate batch host/records stack joined at current split owners |
| 42accbb224a464714f2a078073bee7c098f96892 | c4fa917f | Facts protocol v2 and removal of old node proposal retained |
| f1868ff2c429770295324a58488f067fb052064e | f21b6053 | Source span/chunk extraction and tests already byte-identical; clean source join |

The runtime tree had no delta after the five merges. Original monolithic
attunement, inventory, Parse, records, installed_lifecycle and architecture
files were not restored. Current owners are attunement's replan/command
modules, installation/inventory, parse/{step,publish,worker,records}, split
records/{facts,mutations,tests}, CLI tests/installed_lifecycle and engine
tests/workspace_architecture. Old rrd-estate local_process child-launch locking
is retained in rrd-engram/local_process/{driver,preparation}. Current
architecture dependency assertions retain streaming-iterator. Linux supervisor
id accessors remain; platform capability checks still withhold Parse where no
native supervisor exists. Native supervisor dead-code annotations are unchanged.
Current protocol rejects a partially written next header instead of treating it
as clean EOF. That later fix and its added tests are preserved.

Source facts.rs and its tests at f186 are byte-identical to current. Protocol
parse.rs and parse tests at 42acc are unchanged; the later worker EOF fix and
70 lines of additional worker tests are retained. Original host's scoped thread
batch has been replaced by the current bounded worker with explicit loss and
stop handling. Intermediate host/record source files match the later b56 stack.
Existing restart, replay, edit-only reparse, deletion retirement and physical
batch oracles live in parse_step.rs. One original oracle was absent and is
ported below; Git ancestry alone was not accepted as proof.

Reads: original spans/chunks implementation patch; protocol-v2 frame and
response changes and current run_worker; original host.rs; old oversized and
deletion executor assertions; current parse_step fixtures, query helper and
large physical-batch oracle; current worker, publish measurement/publication,
step gathering/admission and records module roots; source-specific installation
read-source annotation change, supervisor accessor changes, local-process
launch-lock changes and architecture dependency addition. Existing PR482
journal records the earlier complete original record/executor review. Broad
monolith conflict output was truncated and is not claimed as a full reread.
A first git rm refused staged incoming old files; only the explicitly mapped
incoming files were then removed. No unrelated work was removed.

## Reproduced regression and repair

Ported `a_file_whose_facts_exceed_one_commit_is_rejected_not_retried` from the
original stack into current parse_step.rs. It creates a small file and a
40-function file under a 16,384-byte selected commit limit, checks completion
in at most two steps, reads statuses/reasons and symbols through rrflowQL,
then reopens and reads the same refusal. Before repair, on f21b6053 plus that
test, the exact test command failed: big.rs status was `parsed`, expected
`rejected` (0 passed, one failed). This is a real missing constraint, not a
historical source naming difference.

SourceParsePolicy documents max_commit_bytes as maximum encoded bytes in one
atomic publication group. Current step gathering used it only as an estimate;
its first indivisible group was allowed past that estimate, and final measured
admission checked the physical constant alone. The final publication now checks logical mutation JSON against the minimum of
policy and configured transaction admission, independently of measured physical
bytes against the physical payload ceiling. Original source parse.rs explicitly
calls the policy bound logical JSON; physical amplification must not change
that contract. Refusal reasons distinguish commit_budget from physical_batch.
When not even a refusal fits, admission fails before measuring an empty runtime
commit. No parallel publication path is added. POAM-065 records the
verified deficiency; its broader SDK/capability-bound closure stays open.

## Verification

Dedicated target D:/.usr/Temp/rrflow-root-target; four Cargo jobs. Exact first
oracle: `cargo test -p rrd-engine --lib a_file_whose_facts_exceed_one_commit_is_rejected_not_retried --locked -- --nocapture`.
The initial failure above is retained; repair and owning-suite results follow.
No performance, hosted sync, UI, provider or full alpha qualification claim.

An intermediate repair incorrectly applied the logical policy to physical bytes.
The unchanged 16,384-byte fixture then failed with `runtime commit must contain
at least one mutation`: even its refusal could not fit that physical bound.
That approach was discarded before commit, after reading the original stack's
explicit logical-JSON comment and ParseCommit::size/admit/publish. The corrected
implementation preserves both distinct bounds. An apply_patch context mismatch
after rustfmt made no changes; it was reapplied against the observed formatting.

The corrected oversized-file oracle passed unchanged at the 16,384-byte policy
limit. A second oracle sets the commit budget to one byte and proves an explicit
StorageAdmission refusal before an empty runtime measurement, with no source
state published. It passed. Full rrd-attunement suite passed (63 tests) and
engine parse record suite passed (nine). The full parse_step suite, including
the long physical-amplification oracle, is rerun because publication admission
changed; earlier PR482 results are not substituted for this repair.

Full `cargo test -p rrd-engine --lib parse_step --locked`: 11 passed in
326.99 seconds, including physical-batch amplification, the restored logical
limit refusal, too-small refusal admission, restart, replay, edited-file-only
reparse, deletion retirement, commit-index/context readback and followed-project
relearning. Native CLI parser_worker passed three tests; the installed lifecycle test
passed with persistence, authentication and reopen. All 35 workspace architecture
tests passed. `cargo clippy -p rrd-engine --lib --locked -- -D warnings` passed.
The complete logged acceptance run exited zero. Navigation, documentation,
formatting and whitespace checks passed; no ignored qualification test was
enabled or represented as passing.

Joined green PR484 main db97d68c1ca9c432a186c4de159a5b8f634ec8b6 after acceptance.
That join changes only its journal and index; tested runtime and test files are
unchanged. PR484 remote branch was automatically deleted; its local branch was
deleted normally after main advanced.

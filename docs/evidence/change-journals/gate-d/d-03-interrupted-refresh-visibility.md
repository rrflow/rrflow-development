# D-03 interrupted refresh visibility failure

**Status:** active failing regression; repair not implemented
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-interrupted-refresh-visibility`
**Owner:** D-03, POAM-006 and [draft PR #252](https://github.com/rrflow/rrflow-development/pull/252)

The baseline is public main `3a72a6b4adc43c842f67a59f4fc91d725eac7580`, tree `68edd7b9519fa363faca0220875e2a3bd2bac596`; branch bootstrap `dba9cfaf8015d5b76ee243207d3af73df3634f1e` adds no source. The PR change brief preceded the test. The previous tests exercise interruption during the first Inventory, when there is no completed head. This regression creates 2,048 project files, publishes a complete first Inventory, changes every file, accepts a successor plan and cancels after a refresh group is durably committed. It compares the completed head, the root returned at `KNOWN HEAD`, the same root at the completed checkpoint cursor, the completed snapshot's graph link and readback after reopen. No project-owned real data, runtime implementation, public contract, or roadmap completion status was changed.

Two Windows runs of `cargo test --offline --locked -p rrd-engine --lib interrupted_refresh_does_not_expose_partial_facts_as_completed_snapshot -- --nocapture` failed at the final assertion. In the strengthened run the completed control head remained byte-identical; the snapshot traversal at `KNOWN HEAD` reached the old root identity; querying that identity at the checkpoint cursor returned the original digest; querying it at `KNOWN HEAD` returned the different refresh digest; and the head query returned the same result after reopen. That run failed one test, passed zero, filtered 331, and took 74.11 seconds. The first run showed the same root mismatch in 61.18 seconds. This establishes a mixed-generation read when the old snapshot graph and head-time facts are combined. It does not establish that explicitly checkpoint-pinned reads are incorrect, or that all source queries are affected.

Inspected `engine/attunement/inventory/{records,publication,reuse}.rs`, the `rrd-attunement` source-entry identity and Merkle construction, `engine/query.rs`, the existing interruption/delta tests, public query syntax, and the D-03 owner. Intermediate publication writes source records under stable logical identities before the final head transition; the old head retains its older read stamp. [PostgreSQL's isolation documentation](https://www.postgresql.org/docs/18/transaction-iso.html) distinguishes a query snapshot from a larger application operation: separately committed batches are not a single atomic logical publication. No upstream source was adapted.

The regression file and POAM deficiency are the authored changes, with this journal and the generated Gate D index as evidence. The candidate contains an intentionally failing test and remains draft. A possible repair is version-specific physical source records and links while retaining stable logical identity, but an implementation must first resolve authenticated reconstruction of existing snapshots, current versus historical raw queries, removals, bounded batch publication, unchanged reuse, and D-04/client compatibility. A blanket `KNOWN HEAD` rewrite or permanent refresh denial would change query semantics or trap ordinary work; neither is presented as a fix. No candidate CI, merge-group CI, native capacity/soak or other-platform test has run. The installed parser driver remains separately red at `phase-unavailable`.

`python scripts/knowledge/render_navigation.py`, `python scripts/ci/check_documentation.py`, `python scripts/knowledge/render_navigation.py --check`, `cargo fmt --all --check` and `git diff --check` passed. The changed regression and POAM files and full diff were reviewed; the PR brief records the exact result and the versioned-record proposal as unimplemented. The owner and scope, prior behavior, red oracle, trace/research choice, generated evidence, stop conditions and unrun checks are recorded above; no roadmap or parent POAM lifecycle status changed.

## Selection-event substrate characterization, 2026-11-02

Starting branch head `28afd7a047100c7eb8cf36d5d27f268551384bc1`; the PR brief was amended before changing `crates/persistence/rrd-store/tests/installed_lifecycle.rs`. The new native-store test commits two selection-like events with control checkpoints, rejects a stale control compare-and-swap without publishing the staged event, writes an unrelated later claim, reopens, and reads each historical stamp and the newer unrelated state. It verifies the read budget rejects an insufficient allowance. On Windows, `cargo test --offline --locked -p rrd-store --test installed_lifecycle` passed all six tests, including this one. The test characterizes existing substrate behavior; it does not select inventory generations, authenticate event-range completeness or repair the interrupted refresh.

Further code review covered `crates/persistence/rrd-store/src/repository/runtime.rs`, `src/access/versioned_read.rs`, `src/access/semantic_commit.rs`, `src/keyspaces.rs` and `crates/kernel/rrd-core/src/runtime.rs`. The commit builder stages a runtime mutation and control transition in one transaction and returns a stamped result. `read_versioned` authenticates the versions a range returns against the stamped accumulator. Event identities are generated from cursor strings, and an event-kind query scans that physical kind range. The inspected implementation does not establish a bounded, complete latest-selection lookup at a requested cursor: inclusion proofs of returned events alone cannot prove that a newer matching event was not omitted. The proposed selection index and its completeness/freshness authentication therefore remain implementation work; do not enable source visibility on the basis of this passing characterization. This is not evidence that storage corruption occurred.

No production code or schema changed. The existing interrupted-refresh regression remains red at the recorded branch head; it was not rerun because this change does not affect its production path. No candidate CI, merge group, other platform, C-07 capacity or soak was run by this characterization. `cargo test --offline --locked -p rrd-store` passed on Windows, with manual ignored tests left ignored; this includes the six installed-lifecycle tests. `cargo fmt --all --check`, `python scripts/knowledge/render_navigation.py --check` and `git diff --check` passed.

## Selection-index omission fault injection, 2026-11-03

Starting branch head `bb916ca978e7ef5fd12194d2382ddf9f04091cdf`; the PR brief was extended before adding `selection_event_range_authenticates_inclusion_but_not_omission` in `crates/persistence/rrd-store/src/access/versioned_read.rs`. On a real Windows rrflowKV store the fixture committed two selection-like events, captured the current authenticated read stamp, then deliberately deleted only the newer event's semantic-version index entry using a raw storage transaction. Its log entry, accumulator, earlier event and stamp remained intact. The bounded event-range reader returned the older event without an error and did so again after reopen; its I/O evidence passed its existing accounting validation. `cargo test --offline --locked -p rrd-store --lib selection_event_range_authenticates_inclusion_but_not_omission -- --nocapture` passed this **characterization of a gap**, not a fail-closed acceptance test. No ordinary interruption was shown to delete an index entry; no selection index or production fix was implemented. Future repair must replace this observation with an omission-detection oracle and retain the interruption/replan and historical-view tests.

The reviewed source includes `read_stamp_with`/`validate_read_stamp`, `authenticated_steps`, `scan_source`, `validate_version_entry`, keyspace event/version encoding, the semantic-commit event writer, inventory publication/head and reuse code. [RFC 9162](https://www.rfc-editor.org/rfc/rfc9162.html) specifies log inclusion and consistency proofs; they do not by themselves establish that a filtered index returned its latest matching item. [FoundationDB's limitations guidance](https://apple.github.io/foundationdb/known-limitations.html) identifies indirection for bulk publication, while its [transaction design](https://github.com/apple/foundationdb/wiki/Technical-Overview-of-the-Database) describes versioned reads. Those references inform the required atomic selection and read coordinate; they do not prove RRFlow's index or complete-range authentication. Candidate designs need a commitment tied to the requested stamp and a bounded proof of selection freshness/completeness, plus explicit legacy/head reconstruction and all affected query consumers. Do not silently rewind unrelated HEAD reads or trap a project after edits.

Existing `records::load_prior` supplies a useful, narrower boundary: it obtains the completed head's proposal digest and checkpoint stamp, requires each declared member to be present, reconstructs the proposal, and calls `Proposal::from_committed_bytes`, which checks the whole proposal digest and rebuilds its tree under entry, byte and key budgets. That gives the inventory worker a validation path for the entire selected proposal; it does not establish freshness of a corrupted control head, a bounded per-result membership proof, or make generic `KNOWN HEAD` scans, joins and live queries use that selected checkpoint. Reconstructing every member for every query would scale with the project and can exhaust its configured budgets. A future indexed proof must remain rooted in the selected committed proposal, handle removals and historical selection, and be tested against missing, stale and forged entries rather than treating the existing full reconstruction as a free constant-time lookup. [Authenticated key-value store research](https://arxiv.org/abs/1904.12068) distinguishes integrity, range completeness and freshness; it is a research input, not an adopted implementation.

Changed paths for this characterization are the store unit test, the POAM-006 deficiency detail, and this journal. The source/behavior map, debug choice, before/after oracle, scope, stop condition and external research are in the PR brief and above. No production code, public API, roadmap status, project files or permissions changed. The installed-project interrupted-refresh regression remains red; this characterization does not fix it. On Windows `cargo test --offline --locked -p rrd-store` passed, with its manual ignored tests left ignored; the focused test was rerun and passed after adding the evidence-accounting assertion. `cargo fmt --all -- --check`, `python scripts/knowledge/render_navigation.py --check`, `python scripts/ci/check_documentation.py`, and `git diff --check` passed. Exact-candidate and merge-group CI, other platforms, C-07 capacity and soak have not been run.

Accepted public main `a2b3e9df0440aca1b5bd95a2a6422fadbe2b04f8` was merged normally into this draft as `c2a0381f4dc1b8a1489f6ded693ee53ab9d271b0`; its generated Gate D journal index merged without conflict. The pre-extension interrupted-refresh regression was rerun on that combined Windows source and remained red after 65.63 seconds: the completed snapshot root digest at `KNOWN HEAD` differed from its checkpoint-pinned value. The PR brief was then extended before adding lease recovery, resume, new-head and old-pinned-read assertions ahead of the same failing assertion. The extended run and its result are recorded below; neither merge nor characterization repairs the defect.

The extended `cargo test --offline --locked -p rrd-engine --lib interrupted_refresh_does_not_expose_partial_facts_as_completed_snapshot -- --nocapture` run reached the original mixed-generation assertion and failed there after 68.52 seconds. Before that assertion, its real installed Windows engine reopened after interruption, recovered the expired successor lease, resumed and completed the publication, observed a changed completed head and root digest, removed the progress record, and read the original root from the old checkpoint cursor. Thus this fixture demonstrates that recovery can finish and retain its old pinned view; it does not establish that every source query is coherent or that other edits, platforms, resource scales or selection faults are repaired.

The PR brief was updated again before extending the same driver through a second restart, replay of the identical completion command and idempotency key, and independent readback of both the new head and old checkpoint. The Windows command above reached and failed the same original mixed-generation assertion after 70.70 seconds, so those preceding replay/reopen assertions passed. This is one real installed fixture, not a capacity, soak, cross-platform, or complete index-generation acceptance result.

After another PR-brief update, the driver committed an unrelated claim through the authorized engine transaction API after the interrupted source group. It checked that claim at `KNOWN HEAD` before and after the first reopen and again after successor completion, the second reopen and idempotent replay. The Windows command reached the same final mixed-source assertion and failed there after 66.76 seconds; therefore the preceding unrelated-claim, resume, replay, new-head and old-pinned-source checks passed in this fixture. This constrains the future repair: pinning the entire instance to the old source cursor would lose the later claim. No production visibility logic changed.

The mapped existing representation is important to the replacement: `records::mutations` writes snapshot membership as logical `entry_ids`, writes each `source-tree-entry` under that stable logical identity, names containment edges from logical identities, and schedules removed identities for retirement. `records::partition` groups entries and child edges into intermediate commits while leaving the snapshot root, manifest and retirements for completion. `reuse::filter` checks authenticated returned facts against the requested work set and commits a work-set digest; it does not make the intermediate facts invisible. `Publication::finish` atomically updates the job checkpoint, receipt and control head after the source recheck. Generic `execute_query_scoped` obtains an instance read stamp independently and currently does not select that completed head. Any replacement must reconcile membership, edge identities, removal and unchanged reuse with every consumer; simply making the old regression green by versioning IDs leaves staged generations visible to unqualified scans. The existing full-proposal digest verification is a correctness reference, not a substitute for a bounded per-result proof.

### Candidate selection proof to evaluate before implementation

One candidate is an authenticated ordered history of completed selections keyed by instance, project and completion cursor. Atomically publishing a selection would commit its source proposal digest, snapshot membership root and checkpoint identity with the existing job/head/receipt transition. An authenticated predecessor-and-successor proof at a requested cursor could then establish which completed selection was current, including an empty-history proof. The commitment must be bound to a validated control-journal head captured in the same physical transaction as the query's runtime stamp; returned-event inclusion alone is insufficient. A versioned control-journal entry that carries the authenticated selection root forward on later control transitions is one possible design, but its initialization from already installed estates, historical entry verification, concurrent CAS, retention and recovery have not been proved. A physical snapshot/pin with a validated complete range is another candidate only if the rrflowKV and rrflowMX boundaries can retain and authenticate it across compaction and reopen. A full authenticated log replay can be a bounded diagnostic or initialization task, but its cost grows with history and is not an acceptable implicit per-query default.

The existing whole-proposal SHA-256 does not by itself yield an efficient per-entry membership proof. If a Merkle membership root is introduced, the authorized completion must bind that root to the selected proposal and checkpoint; readers must verify the requested records and edges, including absence/removal, before exposing them. For mixed source and unrelated queries, source selection and ordinary runtime state may require distinct explicitly reported coordinates within one engine-owned plan; never relabel an older source cursor as the entire current runtime stamp. Apply selection consistently to record scans, joins, traversals, exports, live queries, indexes and retrieval, and retain exact historical semantics. Fail closed on missing proof or exhausted configurable budget, with a resumable route that does not permanently prevent ordinary edits or successor plans. Test omitted/newer/forged selection and membership entries, empty and deleted sources, stale input, concurrent completion, interruption at each atomic boundary, replay/reopen, old installations, cross-scope isolation, resource limits and 100,000-entry cost before considering the design ready. These are design conditions, not evidence that a new index or proof exists.

## Current-main Linux ARM64 reproduction, 2026-11-04

Draft head `f39b09c44bc6eadacbf1f7ac57cc7261f93466b5` was normally merged with
public main `120391487a260f5395526719e9d25beafcb55ecf` as
`e6d38d23aca50205e56fde58d56ac874239a735e`; no merge conflict occurred. The
PR brief was updated before the merge. The resulting source tree
`ca39b1a622599a314b466889fa14ca84affdd240` was archived with SHA-256
`40939547a26d193e1d623527db3175b5052069bb302590f756ad5aaf7f3c177d`
and transferred to isolated `/tmp/rrflow-d03-visibility.w3xhZpdi` on
`clyffy-01` (Linux ARM64). With two Cargo workers, the command
`cargo test --offline --locked -p rrd-engine --lib
interrupted_refresh_does_not_expose_partial_facts_as_completed_snapshot --
--nocapture` failed at the same mixed-generation assertion: the observed HEAD
root was `a1bc33d98acd5a26f9e96b4cc2ea43028d364250063614ccefb5916e00389163`,
while the completed root was
`e497527b4853d911eb554de8b7605ba833c962f3a8200028556ed517bb8197c3`.
The one test failed after 16.52 seconds, with 343 filtered out. Raw
`regression.log` SHA-256
`5e8051ba44e40a3d355def53f285a44c34a39639c9e8a60e3ee01c84383cd885`.
This confirms the recorded failure on the integrated source and this native
fixture; it does not establish a repair or platform qualification.

Additional current-source inspection followed `RuntimeRepository::read_stamp`,
`read_versioned`, and `commit_with_control_builder`, `ControlRepository::get`,
`StorageTransaction`, `prepare_semantic_commit`, `read_stamp_with`,
`validate_read_stamp`, and `execute_query_scoped`. Each standalone repository
read opens a physical transaction; `commit_with_control_builder` stages runtime
changes and the job/head/receipt controls in one physical commit, but ordinary
query captures a runtime stamp without selecting the inventory control head.
The current `RuntimeChange` canonical leaf authenticates a mutation and its
predecessor; it does not carry an authenticated current inventory-selection root.
Consequently, a proof that an older event is included cannot establish that
there is no newer selected generation. Also, the builder computes the control
head from the resulting runtime stamp, so any proposed metadata must avoid a
circular dependency on that resulting stamp. These are source observations, not
an implementation of a selection proof.

The next production change must resolve how a bounded selection commitment is
bound at the requested cursor and how selected source facts are filtered across
all consumers. Neither a separate, unauthenticated control lookup nor replacing
the whole estate's HEAD with the old source cursor is sufficient. The existing
red test stays active; #241 and the installed project-to-query driver remain
dependent work. No production code, roadmap status, or C-07 status changed in
this reconciliation. Candidate and merge-group CI are not run on this draft.

## Ordered selection proof component, 2026-11-04

Starting draft head `15dc39ee0fab9529bf5245ea11e2e308e6af5a85`, the PR brief
was amended before the code change. The complete existing
`crates/kernel/rrd-core/src/authenticated_log.rs` was read. It authenticates
individual RFC 9162 leaves, but its inclusion proof does not prove that an
index returned the latest matching selection. In the same existing kernel
module, this candidate adds a domain-separated source-generation selection,
an in-memory history builder that rejects out-of-order/foreign appends, and a
predecessor/successor verifier. The verifier requires valid inclusion proofs,
matching scope/project and tree size, adjacent positions or the actual tree
endpoints, and cursor inequalities. It checks an empty history against the
empty root. A resumed builder checks an authenticated last leaf and rejects a
missing, foreign or forged tail.

This only proves a selection **relative to an already trusted root and a
writer-enforced ordering invariant**. It does not authenticate the root at a
runtime stamp, persist the history, prove a source-entry membership, select a
generation in a query, or change inventory publication. A root provided by an
untrusted caller is not sufficient. Storage transaction, source visibility,
legacy recovery, query consumers and the actual installed driver remain open.
The code is not wired into the engine and the interrupted-refresh regression
remains failing; do not mark #252 ready or claim a fix from this component.

Tests cover empty/before/exact/between/after cases, omitted and nonadjacent
neighbors, altered root/value, wrong project or size, foreign and non-monotonic
appends, resuming with a valid or forged tail, and 17 ordered histories against
an independent linear latest-selection oracle. RFC 9162 specifies inclusion
proofs and FoundationDB's documented subspace indirection informs the desired
atomic switch; neither proves RRFlow's root binding or storage implementation.
No upstream code was copied. References:
<https://datatracker.ietf.org/doc/html/rfc9162> and
<https://apple.github.io/foundationdb/subspace-indirection.html>.

During this work, a new test demonstrated that the shared digest parser could
panic on a 64-byte non-ASCII string, because it sliced before checking byte
boundaries. On ARM64 the red test
`cargo test --offline --locked -p rrd-core
digest_parser_rejects_multibyte_text_without_panicking -- --nocapture` failed
with `end byte index 2 is not a char boundary`; log `digest-red.log` SHA-256
`54eea0cf09282b6c30d29e6c42f774d78fe400a7c6115880db15ad86cc4435b6`.
The parser now validates lowercase ASCII hex before slicing, preserving the
existing canonical digest rule. The actual final Rust file has SHA-256
`93c3eed8a93d1b945e5a41fe9df65be985d0e4ea928fa714e9072f2f5d01ee65`.

On `clyffy-01`, with two Cargo workers and the merged source tree above plus
that exact file, `cargo test --offline --locked -p rrd-core` passed with one
existing ignored unit test; `cargo clippy --offline --locked -p rrd-core
--all-targets -- -D warnings` passed. Logs `core-tests-final2.log` and
`core-clippy-final2.log` have SHA-256
`d6005631216db95bf6994f17fcc04ae9ee24611478d70d70e391f4462d13705f`
and `652278900feca816c697c18413aa54c027e4994e5bf10079080242f600e1b13b`.
The installed-engine interrupted-refresh regression was rerun after an earlier
version of the component (before the additional linear-oracle test) and still
failed the original assertion; `regression-after-proof.log` SHA-256
`17b09202133506995747bd294ede14d3c0ff5b57848732a393ae5c3d662737a6`.
This is expected because no production inventory reader or publisher uses the
new proof. The subsequent change added only the linear-oracle test; no engine
integration occurred. Native Windows/macOS, full candidate CI, merge-group CI,
capacity, soak and the installed project-to-query driver were not run.

The component changes only `authenticated_log.rs` and this existing journal;
there are no moved or deleted paths or new runtime trace events. Its bounds
are limited proof paths and selection identity bytes, not a measured total
resource or performance guarantee. The proof's 256-byte scope/project bound
limits untrusted proof allocation; the existing `ScopeId` and `RuntimeId`
types do not impose that bound. Existing installed identities must be checked
and the compatibility and configuration decision resolved before integration;
this test does not prove all admitted estates fit. No objective, roadmap or POA&M status has
changed. The next required work is a stamp-bound authenticated selection root,
transactional durable updates and source membership, followed by all affected
query paths and the unchanged red regression. If those cannot be made coherent
without a second authority or loss of existing stamps, keep the path disabled
and the PR draft; do not substitute a full-estate cursor rewind.

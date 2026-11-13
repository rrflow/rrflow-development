# D-02 installed code-knowledge slice

**Status:** active implementation record; D-02, OBJ-06, OBJ-07, POAM-006 and POAM-065 remain open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-code-knowledge-slice-20261112`

**Owner:** OBJ-06 / OBJ-07 / OBJ-14; the [installed code-knowledge slice](../../../roadmap/d02-evidence-driven-attunement-engineering-plan.md#installed-code-knowledge-slice-2026-11-12) of the D-02 evidence-driven attunement plan, which finishes P3 and does the parts of P5, J3 and J6 a question over real code needs

**Starting revision:** main `15d1b65c`.

**Current behavior.** A derived attunement plan never selects Parse (`EXECUTABLE_PHASES` is Connect and Inventory). Switched on, Parse's first commit on this repository is refused at the 16 MiB physical write-batch ceiling and every Step retries it. After Parse, `context-assemble` and a `WHERE name = …` lookup over `source-symbol` are both refused at the 100,000-key read budget, because each reads every record before ranking or filtering; the store writes scalar entries at commit and nothing reads them, and BM25 is rebuilt per request. Nothing attunes a project unless a client steps it.

**Target behavior.** Installing and serving this repository, with no client driving it, parses its code, indexes it at commit, answers questions and symbol lookups through those indexes within the default budgets, and relearns an edit.

## Changes

- **Parse is executable and sizes its commits exactly (POAM-065).** `EXECUTABLE_PHASES` includes Parse. The LSM transaction and the in-memory write set gain a measuring mode that stages past the payload ceiling, reports the exact encoded size and refuses to commit; a semantic commit plan built from a measuring reader measures too. `RuntimeRepository::measure_commit_with_control` stages a commit and its control transitions that way and rolls back. Parse gathers per-file groups, calibrates the JSON-to-physical ratio once early in each Step, and before publishing trims to the longest prefix the store measures within `PHYSICAL_BATCH_PAYLOAD_BYTES`; a single file that cannot fit any batch is refused with the measured size.
- **One owner for text analysis.** BM25 configuration and the analyzer move from `rrd-query` to `rrd_store::lexical`, so a term is the same at commit and at query time. A new `identifiers` option (`pieces`, the unchanged default, or `whole_and_parts`) indexes `read_stamp` as `read`, `stamp` and `read_stamp`, and `RrdEngine` as `rrdengine`, `rrd` and `engine`; the public `QueryFullTextConfiguration` carries it as `QueryTextIdentifiers`. This takes the default for D-02's open tokenizer decision.
- **Persistent postings (E-03 / P5).** A BM25 commit binding carries its configuration and maintains, in the same transaction as the records, one posting per distinct term per document, each term's document frequency and the index's totals, in the reserved `TermPosting`, `TermDictionary` and `TermStatistic` key families. Bindings backfill and clear postings like scalar entries. `RuntimeRepository::lexical_search` ranks exact BM25 top-k over current postings and `index_lookup` selects identities from scalar entries. Both answer only at the head read stamp and for a valid time at or after the index's watermark (the latest valid time the visible indexed set changed); otherwise they report not applicable. A version with an explicit end of validity raises the watermark past every valid time, so the index never answers wrongly.
- **Historical read stamps survive catalogue changes.** Defining an index bumped the scope's catalogue revision, and a historical stamp was validated against the current revision, so Parse could no longer read the Inventory its job was planned from. Each catalogue revision now records the runtime cursor at which it took effect, and a historical stamp is valid when its revision was in effect at its cursor.
- **rrflowQL reads the indexes (J3).** Every scalar or BM25 index over a record source is a commit index from the moment it is defined. A query whose leading fields an index binds by equality, or whose only predicate is a `MATCH` on an index's field, is planned as `commit_index:{id}`: the index names the identities to read, those are read through the authenticated identity path at the stamp, and every predicate is evaluated. Binding such a query captures only the schema, as an identity read does, instead of reading the whole family for a watermark. When the index declines at execution, the family is read in full. A fresh artifact index is still preferred.
- **Parse declares its indexes.** Parse registers its tables in their own commit and defines `source-symbol-name`, `source-symbol-path`, `source-span-path`, `source-chunk-path` (scalar) and `source-chunk-text` (BM25, `whole_and_parts`) before writing any fact, so every fact is indexed in the commit that writes it.
- **Context through the indexes (first part of J6).** The lexical stage ranks every kind with a BM25 index from its postings at one captured stamp and reads only the hits; kinds whose only indexes are scalar are read by identity on demand, not scanned; the rest of the estate is read at the same stamp (`data_snapshot_at`) and ranked as before; reciprocal-rank fusion combines them. The stage reports the new `ContextAccessPath::CommitBm25`.
- **The engine keeps a project attuned.** `AttunementFollowPolicy` in the estate configuration, sealed by `install plan --follow-interval-ms`, makes `serve` run `RrdEngine::follow_attunement` on its own thread. A metadata fingerprint of the Inventory walk decides whether files changed; when they did, or on the first pass, the engine drives a pending job, recovers and resumes a job whose lease expired, derives the successor plan from the committed evidence, accepts it when it differs from the newest plan and steps it to the end, all through the public attunement operations as the installed operator. A replan may now follow a Parse checkpoint, because a successor's Parse reconciles Parse's content-addressed facts itself.
- **An undecodable request keeps the caller's identity.** The server answers a payload that does not decode under the envelope's own request context when it carries a valid one, so clients see the typed error.

## Oracles

Each oracle was observed failing before its fix where the fix is a behavior change:

- `rrd-engine` `facts_larger_than_one_physical_batch_are_published_across_steps` fails on main's Parse with `write batch encoded_bytes limit 16777216 would be exceeded`, the real run's error, and passes with exact sizing.
- `rrd-store` `a_historical_stamp_survives_a_later_catalogue_revision` fails under the old rule with `ReadStampMismatch`, the error Parse hit, and passes with revision history.
- `rrd-store` `a_measured_commit_is_admitted_exactly_when_it_fits_the_payload_ceiling`: every batch that measures within the ceiling commits and every one over it is refused and changes nothing. `rrd-lsm` `a_measuring_transaction_reports_a_batch_past_the_payload_ceiling_and_cannot_commit`.
- `rrd-store` `postings_equal_a_full_rebuild_through_backfill_update_and_retirement` and `postings_survive_reopen` compare ranking and scores with BM25 computed from scratch on both storage profiles; `scalar_lookup_selects_exactly_the_current_matching_records` covers rename, retirement, stale stamps and the watermark.
- `rrd-query` `commit_index_reads_equal_a_full_scan_on_every_engine`: with identical data, a store with indexes returns the same rows, scores, matched terms and highlights as one without, reads fewer versions, and falls back correctly before the latest change.
- `rrd-engine` `parsed_facts_are_read_through_their_commit_indexes`, `context_answers_from_parsed_code_through_commit_postings`, `a_followed_project_is_attuned_and_relearned_by_the_engine` (only the edited file is parsed again) and the updated derivation test (`[connect, inventory, parse]`).
- `rrd-server` `an_undecodable_payload_keeps_the_callers_identity`.
- Two `rrd-query` index tests asserted that a stale or unbuilt artifact falls back to a full scan; they now assert the commit index and still check the rows.

## Real run

Release `rrflow` built from this branch (the final run's binary predates only the follower's conflict retry), on a fresh clone of this repository at `15d1b65c` (2,826 tracked files, 7.2 MB of Rust, Python and Go), Windows 11 laptop with other containers running. Commands: `rrflow install identity`; `rrflow install plan --mode existing --allow-project-source-read --follow-interval-ms 5000` (the preview shows `attunement_follow: {interval_ms: 5000}`); `rrflow install apply --expect <plan_sha256>`; `RRFLOW_TRACE=rrd_server::attunement_follow=info,warn rrflow serve`. No client started, stepped or replanned a job; the probe only read.

| Check | Observed |
| --- | --- |
| Install plan and apply | 33 s |
| The engine attunes itself | The follower started the install job, ran Connect and Inventory (about 20 s), derived `[connect, inventory, parse]`, accepted it and stepped Parse to success. One clean run took 6 minutes and 72 Steps. In the final run a Step lost a commit race to the probe's own session creation at 06:04:55; the next passes waited for the lease, recovered and resumed the job, and it succeeded at 06:11:13, 9 minutes after serving began. The follower now retries such a Step instead. |
| Parse coverage | 901 of 902 files parsed; `crates/transport/rrd-contract/src/lib.rs` refused as `facts_exceed_physical_batch:18720270>16777216` |
| Estate on disk | 236 MB for 7.2 MB of parsed source, before compaction |
| Symbol lookup `WHERE name = "control_attunement"` | 5 ms through `commit_index:source-symbol-name`; the definition in `engine/attunement.rs` and its two references. Before: refused after 99,954 keys. |
| "how does the vector planner choose TurboQuant or the exact scan" | `rrd-vector/src/runtime.rs` first; 349 ms cold, 1,911 ms on the second ask |
| "which code commits a parse step checkpoint" | `attunement/parse` code first; 358 ms and 448 ms |
| "measure_commit_with_control" | `rrd-store/src/repository/runtime.rs` first; 346 ms and 336 ms |
| "where is the LSM compaction policy" | **Missed:** rrd-lsm's maintenance and compaction tests rank first and `rrd-lsm/src/database.rs` is not in the top five; 396 ms and 389 ms |
| Edit one Rust file | Relearned by the follower in 30 s; exactly one file reparsed (`rrd-vector/src/cost.rs`); the new function is found by name |
| Restart | The lookup answers in 9 ms through the same index; the follower's first pass reports the plan current and does nothing |

Every question ran through `ContextAccessPath::CommitBm25` within the default 100,000-key budget. Before this package each was refused. Three of the four named answers are first; the miss is a ranking result, recorded rather than tuned.

## Commands

@@COMMANDS@@

## Surfaced and not fixed here

- **Write amplification.** 1.34 MiB of parse facts as JSON measured 10.5 MiB physically: records, versions, the change log, a source delta per index carrying the indexed text, and postings. Parse with indexes is several times slower than without. Recorded in POAM-065; not changed here.
- **One file's facts exceed one batch.** `crates/transport/rrd-contract/src/lib.rs` measures 18.7 MB against the 16 MiB ceiling and is refused; paging a file's facts across commits is not done.
- **An install is bound to the binary that made it.** A rebuilt `rrflow` refuses an existing estate (`ProjectBindingMismatch`), so every build in the real run needed a new install. Recorded in POAM-006.
- **Follow is opt-in.** The bundled default configuration does not enable it; `install plan --follow-interval-ms` seals it. Turning it on by default is a one-line change to `default-estate-configuration-v3.toml` and the owner's call.
- P4's relations, P9/J7 embeddings and the Connectome views are not in this slice.

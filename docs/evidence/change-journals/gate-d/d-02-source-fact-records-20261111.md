# D-02 P3: parsed-source record format

**Status:** active implementation evidence; nothing writes these records until the Parse executor lands; D-02 and POAM-059 remain open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-source-fact-records-20261111`

**Owner:** OBJ-07 / OBJ-14; package P3 of the [D-02 evidence-driven attunement plan](../../../roadmap/d02-evidence-driven-attunement-engineering-plan.md) ("new runtime records `source-span`, `source-symbol`, `source-chunk`"); one pull request of the [P3 series](d-02-p3-parse-executor-20261111.md)

**Starting revision:** main `a75a47d6`, after #334.

**Current behavior.** The engine has no record type for parsed source.

**Target behavior.** `engine/attunement/parse/records.rs` defines the format the executor will write.

- **Tables.** `register` adds four strict graph-node tables to the instance's runtime schema: `source-symbol`, `source-span`, `source-chunk` and `source-parse`. Every property is required and none are additional. Every fact carries `path`, `content_sha256` and `language`. The kind columns are `symbol_kind` and `span_kind`, because a property named `kind` is unreadable through rrflowQL today ([POAM-062](../../../poam/rrflow-1.0-alpha/poam-062.md)).
- **Per-file state.** `source-parse` makes Parse resumable. It records the content digest, parser revision, chunk size, the job that wrote it, a status (`parsed`, `rejected` or `lost`) with its reason, and the fact counts that tell a later edit what to retire.
- **Identities.** A fact's id is derived from its path, content digest, parser revision, chunk size and index, so the same file content always names the same records and changed content never collides with them. A state's id is derived from its path alone, so each file has one current state.
- **Loading.** `load_states` reads the latest non-retired state per path at a read stamp, within the caller's storage-key budget.

**Changed paths.** `crates/authority/rrd-engine/src/engine/attunement.rs` (`mod parse`); `attunement/parse.rs` (new; holds the module until the executor arrives); `attunement/parse/records.rs` (new); this record.

**Oracles, written with the module, not observed red first.** In-module tests: the four tables register once and a second registration changes nothing; a fact's identity changes with each of path, content, parser revision, chunk size and index; a file state reads back unchanged from the record it is written as. Writing and loading through a real store is exercised by the executor's tests.

**Acceptance.** Windows, own target directory: `cargo test --locked -p rrd-engine --lib parse::records` 3 passed; `cargo test --locked -p rrd-engine --test workspace_architecture --test direct_read_source_closure` 33 and 1 passed; `cargo clippy --locked -p rrd-engine --all-targets -- -D warnings` clean.

**Research decision:** content-addressed fact identity follows the plan's research (SCIP incremental indexing, LlamaIndex content-hash caching); no upstream code.

**Checks not run:** the full engine suite locally (nothing outside this module calls it); Linux and macOS locally.

**Roadmap/POA&M change:** none.

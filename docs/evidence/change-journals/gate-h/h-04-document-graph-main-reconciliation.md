# Document and relationship journey: current-main reconciliation

**Status:** active integration evidence; candidate and merge-group CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-document-graph-main-reconciliation`
**Owner:** OBJ-02/07/09, H-04 and [PR #255](https://github.com/rrflow/rrflow-development/pull/255)

PR #255's original candidate was `cc1144c7768395c108d31adbd21f397879323fdd`.
Its [original journal](h-04-document-graph-journey.md) records the failure and
the strict-document example at that revision. The subsequently merged #256
added bounded schemaless record projection; its
[journal](../gate-e/e-01-schemaless-document-query.md) records the separate
real-store oracle and limitations. Current public main at integration was
`bf0b86a4c3adef1d8dfe7183909758ebe81a91a2`.

The old PR's current-tense POAM and SDK-guide text would incorrectly say that
registered schemaless documents remain unqueryable. Auto-merge was disabled
before reconciliation. The ordinary main merge produced no textual conflicts,
but those two statements needed semantic reconciliation: the original failure
is retained as historical evidence; the current records describe bounded
projection and retain the missing dynamic predicates and arbitrary join keys.
The guide's model and read/verify description now also include the document,
supports relationship and four stamped results. The typed example remains
unchanged. No roadmap or POA&M status changes.

Reviewed the complete PR diff, the current POAM and SDK-guide sections, and the
schemaless-query journal and binder. The owning records, implementation paths,
research and original exact-source verification remain in the original PR
brief and journal. No upstream code was adapted and no trace or runtime
authority was added by this reconciliation. Candidate and protected CI must
run on the combined head.

On Windows, `cargo test --offline --locked -p rrd-client --test rrflowdb_local`
passed all five tests, including the installed document/relationship transaction,
rejection and reopen path. `python scripts/ci/test_rrflowdb_standalone.py -v`
passed seven independent-oracle and workflow tests. A new
`cargo build --offline --locked -p rrflow-cli --bin rrflow -p rrd-client --example rrflowdb`
and `python scripts/ci/run_rrflowdb_standalone.py target/debug/rrflow.exe target/debug/examples/rrflowdb.exe`
passed on a synthetic fresh project with real native processes and cursor 9.
The primary SHA-256 was `a288ad1a5a33c5bde029b5d6b1ef24208d8e529062dd94ec6745d781a6779144`;
the example SHA-256 was `89458dcf040be000eaae2a34dc90ca0a14aaf1ee6437f84a4129df431dc2a89a`.
These are source-built Windows artifacts, not signed releases or capacity proof.

`cargo fmt --all -- --check`, documentation policy, navigation generation and
`git diff --cached --check` passed. The generated execution inventory is
refreshed after this record. The exact candidate and merge-group CI, macOS and
Linux integrated process evidence, project indexing, general schemaless queries,
PostgreSQL compatibility, native capacity and soak remain unproven by this
local run. The PR and its CI state are the merge record; this journal does not
close a roadmap or POA&M item.

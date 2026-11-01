# E-01 traversal source identities

**Status:** active draft; candidate and merge evidence pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-01-traversal-source-identities`
**Owner:** OBJ-07, D-04 first project cycle and E-01 graph traversal;
[PR #242](https://github.com/rrflow/rrflow-development/pull/242)

Starting public main `daec1f1a2959f16ce41da9fb691ab1791ed06db5`, tree
`04e61b28076eef7e0b2564f8c300449fb642844c`. The empty bootstrap commit
`1a4d1d185bdc869c252202436c7dd1a74e241d02` was pushed to
`https://github.com/rrflow/rrflow-development.git`, ref
`refs/heads/agent/rrflowql-inventory-identities`, before implementation so
the draft PR's change brief could precede code.

The local D-04 installed-estate journey committed an actual Inventory graph.
Its rrflowQL traversal failed at the source snapshot ID when the digest began
with a digit: `expected traversal start ID, found 49`. The tokenizer separates
numeric tokens and the traversal parser accepted only word tokens. Typed
traversal canonical rendering also wrote those IDs bare. This prevents a real
source identity from being addressed; changing the stored identity or seeding
a substitute would not repair the path.

The proposed repair accepts an existing double-quoted string token for the
traversal start ID and uses the existing runtime-reference validator. It
retains bare word IDs, renders a quoted ID when its exact bytes cannot be one
word token, and leaves numeric tokenization, graph storage, authorization,
read stamps and query execution unchanged. Its focused oracle covers
numeric-leading and leading-zero digests, numeric-only, punctuation, escaped
and Unicode IDs, invalid identities, and canonical reparse. A persistent
rrflowKV graph and reopen test checks the actual query path. Source-level
scope: `crates/compute/rrd-query/src/syntax.rs`, existing syntax and graph
tests, the query reference, POAM-005 and this journal. No previous
implementation is removed or moved.

Before implementation, `cargo test --offline --locked -p rrd-query --test
syntax_parser traversal_start_ids_round_trip_exactly_when_quoted` failed on
the first quoted digest with `expected traversal start ID, found string`.
The first compile used the D-04 worktree's target directory on Windows; its
2m39s build was not a passing test. Candidate CI and protected merge remain pending.
This repair does not establish project parsing, prompt grounding, LFG/SWAG,
100,000-file behavior or alpha completion. No roadmap status is changed.

After the local repair, the focused syntax test passed. The focused
`cargo test --offline --locked -p rrd-query --test graph_traversal
quoted_digest_start_reaches_the_persisted_graph_after_reopen` also passed: it
commits the exact digit-leading start ID, its target and relation to rrflowKV,
queries through the native adjacency plan at a captured stamp, reopens the
store and obtains the same target. `cargo test --offline --locked -p
rrd-query` then passed the owning suite, including the existing temporal graph
differential. A newline/tab identity was added afterward and the focused
syntax test passed again. These Windows local results are not candidate CI.

Strict `cargo clippy --offline --locked -p rrd-query --all-targets -- -D
warnings` passed. Documentation policy and workflow policy checks passed.
The exact local query-source patch was also applied temporarily to the dirty
D-04 worktree: its authenticated embedded installed-estate test reopened an
actual Inventory and traversed from the real snapshot digest to the same
source entry used by Parse input admission. The command was `cargo test
--offline --locked -p rrd-engine --lib
parse_source_is_bound_to_the_committed_inventory_after_reopen -- --nocapture`.
It passed with one test. This is not yet an ordinary branch merge or a shipped
process test; #241 must reconcile the patch with this PR's reviewed commit.

Change checklist: the owning README, objective and E-01/F-03 roadmap and query
reference were consulted; the existing parser/tokenizer, runtime ID contract,
syntax corpus and native traversal fixture were inspected before the brief.
The changed parser, syntax and graph tests, query reference, POA&M detail and
generated journal index were reread with the complete diff. No provider,
model, filesystem mutation or new runtime/storage authority is introduced.
Formatting and `git diff --check` passed; generated navigation and execution
inventory were regenerated and checked. No candidate CI, merge-group CI,
native non-Windows test or full installed prompt cycle has been run for this
revision. The owning roadmap and POA&M lifecycle statuses are unchanged.

Public main advanced to `db2a8d04fc4c3b24b2b7ba33a04434d8d7c5d6a2` after
the local tests. The repository conflict inspector reported a clean merge; the
ordinary merge produced `d468d967cb8b2cfd2d23dabf10ee5c8711e00d1e` and changed
no `rrd-query` source or test. Navigation passed. The execution-inventory
check initially reported drift from the newly merged files; regenerating it
and repeating `--check` passed for 2,276 paths. Workflow policy passed again.
This integration does not stand in for candidate CI or protected merge.

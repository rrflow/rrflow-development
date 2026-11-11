# D-02 P3: symbol facts from tag queries

**Status:** active implementation evidence; nothing calls the facts until the worker protocol carries them; D-02 and POAM-059 remain open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-p3-source-symbols-20261111`

**Owner:** OBJ-07 / OBJ-14; package P3 of the [D-02 evidence-driven attunement plan](../../../roadmap/d02-evidence-driven-attunement-engineering-plan.md); one pull request of the [P3 series](d-02-p3-parse-executor-20261111.md)

**Starting revision:** main `a75a47d6`, after #334.

**Current behavior.** A parse yields every named node with its byte range and kind, and no definitions, references or names.

**Target behavior.** `rrd_attunement::facts::source_facts` runs one bounded `parse_tree` and returns `SourceFacts`: node and error-node counts, and symbols from each grammar's own `TAGS_QUERY` (tree-sitter-rust 0.23.3, -python 0.23.6, -go 0.23.4). A symbol is a role (definition or reference), the tag's kind, the name's byte range, the tagged node's range and its zero-based line. A name tagged by two patterns, such as a Rust method also matching the function pattern, is kept once by its first, more specific pattern. References to one name and kind collapse into one symbol with an occurrence count and the first line. Facts are byte ranges only: the engine will slice names from bytes it read and verified, so a worker can never supply source text. Node count, depth, elapsed time, cancellation and the serialized size are all bounded by the existing `ParseLimits`.

**Changed paths.** `crates/compute/rrd-attunement/{Cargo.toml, src/lib.rs, src/facts.rs (new), src/parse.rs (Parsed carries the grammar), tests/facts.rs (new)}`; `Cargo.lock` (`streaming-iterator 0.1.9`, already locked through tree-sitter, becomes a direct dependency because query matches are only readable through its trait); `crates/authority/rrd-engine/tests/workspace_architecture.rs` (the allowlist names it); this record.

**Oracles, written with the module, not observed red first.** `tests/facts.rs`, six cases: Rust definitions named from exact ranges, a method not also reported as a function, collapsed references with count 2 and first line 7, Python and Go definitions from their tag queries, syntax errors counted, and digest, language, node, output-size and cancellation limits enforced. The independent oracle is the real run at the end of the series: definitions per Python file against Python's own `ast`.

**Acceptance.** Windows, own target directory: `cargo test --locked -p rrd-attunement` 55 passed, 0 failed; `cargo test --locked -p rrd-engine --test workspace_architecture` 33 passed; `cargo clippy --locked -p rrd-attunement --all-targets -- -D warnings` clean.

**Research decision:** tree-sitter tag queries as shipped by each grammar, the convention tree-sitter's `tags` command and GitHub code navigation use; no hand-written queries and no upstream code.

**Checks not run:** Linux and macOS locally; candidate CI covers them.

**Roadmap/POA&M change:** none.

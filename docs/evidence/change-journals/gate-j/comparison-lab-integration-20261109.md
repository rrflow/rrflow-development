# Comparison lab integration onto main

**Status:** active change-journal evidence; the lab's eight stacked packages land as one integration, no measurement or alpha claim is made here
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/comparison-lab-integration-20261109`
**Owner:** [Gate J](../../../roadmap/rrflow-1.0/gate-j.md), OBJ-10/OBJ-11; the owner's 2026-11-09 direction to pull the lab, rework it and absorb it

## What was absorbed

The comparison lab existed as eight stacked pull requests that never
reached `main`, each 130 to 270 commits behind it:

| Line | Pull requests | Content |
|---|---|---|
| vector | #253 → #260 → #268 → #272 → #274 → #277 | the `rrflow-eval` lab module, adapters for Qdrant, Milvus, SurrealDB, Lance and RRFlow, imports from VectorDBBench, the one-file-per-system and per-test catalogue, signed definition packs, one-command automation with settings and approval, and reference results beside published numbers |
| reasoning | #253 → #260 → #262 → #267 | planning-task tests with a paired, preregistered design, the knowledge import, study configuration from a schema, and capture-integrity audits |

Each package keeps its own change brief in its pull request and its own
journal under this directory; this record covers only the integration.

## How it was integrated

Starting revision: `development/main` at `a9f1b0fb`.

1. The vector line's head (#277, `tmp/agent/lab-reference-results`) was
   merged onto main. Five files conflicted. The two engine files and the
   Gate E journal came from #278 (the HNSW generation read path), which the
   lab tree had carried so its measurements could use that path. #278 is its
   own package with two unresolved wrong-answer regressions, so the
   integration keeps main's `engine/vector/search.rs`, `rrd-vector/hnsw.rs`,
   `tests/vector_index.rs` and vector reference pages, and drops the
   `hnsw_read_path` and client `vector_read_path` tests and #278's journal.
   The lab's measurements recorded in its journals were taken on that
   composite tree; they describe that tree, not this one, and the journals
   say so. POA&M rows 053 to 056 and the generated indexes merged cleanly.
2. The reasoning line's head (#267, `tmp/agent/lab-study-requirements`) was
   merged on top. It had branched from an older #260, so twelve files
   conflicted. For the shared lab infrastructure (`manifest.rs`,
   `rrflow.rs`, `run_system.sh`, the `vector-*.json` manifests and the lab
   README's existing sections) the vector line's current version was kept:
   the reasoning line's copies were older states of the same files, not
   changes of their own. The reasoning line's additions were kept whole:
   `knowledge.rs`, `provider.rs`, `study.rs`, `tasks.rs`, their tests, task
   and study definitions, schemas, scripts and README section; its CLI
   subcommands and schema registrations were unioned into `rrflow_lab.rs`
   and the contract test. Two adapter methods the tasks depend on,
   `RrflowAdapter::connect_to` and `import_chunks`, were ported from the
   reasoning line onto the vector line's adapter, which already had the
   batching helpers they use.
3. One reasoning-line test, `rrflow_hnsw_requires_explicit_native_build_parameters`,
   asserted a manifest field (`rrflow_hnsw`) that the catalogue line
   replaced with per-system definitions in `eval/lab/systems/`. The field no
   longer exists, so the test was removed; the catalogue's own tests cover
   the native parameters. The checked-in schemas were regenerated with
   `rrflow-lab schemas --out eval/schemas` and the manifests with
   `rrflow-lab resolve-all`, so the generated-artifact tests hold.

## Evidence

Local, Windows x86_64, dedicated Cargo target:

- `cargo build --locked -p rrflow-eval --all-targets`: clean.
- `cargo test --locked -p rrflow-eval --no-fail-fast`: every test target
  passes after the regeneration above (the first run failed exactly the
  three generated-artifact tests and the obsolete test named here).
- `cargo clippy --locked -p rrflow-eval --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- Documentation policy and navigation check: OK.
- No file outside `crates/evaluation/`, `eval/`, `docs/` and the POA&M
  changed against main; the lane classifier selects `engine`, `scripts`
  and `docs` for this change, not the full matrix.

Not run here: a live lab run against any system; the macOS, Windows and
ARM partitions (candidate CI decides which lanes apply). The measured
numbers in the package journals are not reproduced by this integration.

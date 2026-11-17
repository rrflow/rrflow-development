# Fjall comparison baseline restored as a feature-gated backend

**Status:** package journal, 2026-11-17
**Owner:** repository

## Change brief

- **Current behaviour:** Fjall and the native-versus-Fjall comparison were removed in `b4e711e27` (2026-09-07); rrflowKV's benchmarks are native-only, `rrd-store` cannot run any workload on a second engine, and an architecture test forbids Fjall in the workspace.
- **Target:** the comparison is available for testing, without Fjall entering any production build.
- **Scope:** `rrd-store` feature `fjall-baseline` (optional `fjall =3.1.8`), `src/fjall_baseline.rs`, `tests/fjall_baseline.rs`, `examples/storage_comparison.rs`; the Fjall assertion in `workspace_architecture/storage.rs` narrowed to the real invariant; the harness document.
- **Failure oracle:** the transcript test fails if Fjall's transaction semantics differ from rrflowMX or rrflowKV in any line; the architecture test fails if Fjall is declared by another package, is non-optional, or any package enables the feature.

## What was read first

The removed code: the old Fjall `Store` mixed semantics into 18 Fjall keyspaces and could not be ported, since semantics now sit in repositories over the 10-method `StorageTransaction`. The contract was taken from `RrflowMxStore`, the reference side of the existing conformance differentials, and the durability mapping (`Authoritative` to `SyncAll`, `Buffered` to `Buffer`) from the removed code.

## Result

`cargo test -p rrd-store --features fjall-baseline --test fjall_baseline`: 4 passed. `cargo test -p rrd-engine --test workspace_architecture`: 34 of 34 after the new files were tracked. A two-trial run of `storage_comparison` verified the whole 1,024-claim corpus on both engines and produced both engines' measurements and ratios; that run was a debug build on Windows and is not a benchmark.

## Not run

The comparison at benchmark scale on Linux, which is the only place peak memory and allocated bytes exist. The AI-access matrix against Fjall. The workflow job.

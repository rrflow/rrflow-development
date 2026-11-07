# Taking digest hex formatting out of authenticated reads

**Status:** active immutable change-journal evidence; POAM-035 and C-07 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/digest-hex-read-cost`
**Owner:** OBJ-02 / C-07g-h / POAM-035 / PR257; Gate C retains completion authority

## Package

- **Advances:** OBJ-02 and C-07's authenticated read cost, under POAM-035.
- **Starting revision:** `a2b3e9df0440aca1b5bd95a2a6422fadbe2b04f8`.
- **Pull request:** [#257](https://github.com/rrflow/rrflow-development/pull/257), found through
  the comparison lab in draft #253 under the owner's 2026-11-03 direction to tune from its results.
- **Current to target:** a `perf` profile of exact search at 20,000 points put 87.0% of its time
  in `authenticated_steps` and 0.81% in scoring, with `parse_digest` at 26.31% and `encode_digest`
  at 23.01% inclusive, largely in `core::fmt`. The encoder now uses a table and the decoder reads
  bytes. `parse_digest` also sliced by byte position, so a 64-byte value holding a multi-byte
  character panicked at `authenticated_log.rs:365`. It now fails closed.
- **Unchanged:** what is verified and how often, proof structure, storage format, wire formats,
  accepted inputs, outputs and error messages. The trust model does not change.
- **Stop conditions:** exact p50 falling by less than 10%, recall changing at all, or a digest input
  changing behavior other than the panic becoming an error. See the result for the 8-client case.

## Files read

`docs/evidence/change-journals/gate-c/c-07g-proof-node-reuse.md`,
`docs/poam/rrflow-1.0-alpha/poam-035.md`, `authenticated_log.rs`, `digest.rs`,
`access/proof_nodes.rs` and `access/versioned_read.rs` (`authenticate_changes`).

## Changed paths

Changed `crates/kernel/rrd-core/src/digest.rs` (`hex`, new `from_hex`) and
`crates/kernel/rrd-core/src/authenticated_log.rs` (`encode_digest`, `parse_digest`, tests). Created
this journal and changed the generated gate C index.

## Decisions

- **Research:** none needed. The change is local and adapts no upstream code.
- **Trace, resource, debugging:** the `perf` samples in `runs/profile-1` on `clyffy-01` are the
  resource evidence. No trace is added.

## Oracle and results

- The malformed-digest test panicked on the starting revision, at `authenticated_log.rs:365:42`,
  and passes now. An equivalence test holds the encoder to `format!("{byte:02x}")` for every byte.
- `cargo test --locked -p rrd-core --no-fail-fast`: 78 passed, 0 failed, 1 ignored.
- `cargo test --locked -p rrd-store --lib`: 86 passed, 2 ignored.
- Clippy with `-D warnings` and fmt are clean on both crates.

Before and after, on `clyffy-01`, alternating, with the GPU at 0% in every host sample. It uses
20,000 points with plain search, because a filter adds nothing to this question and takes time.
That departs from the brief's unchanged `vector-small.json`, and it leaves the corpus as it was.

| Exact search, p50 | Before, `7114913805e93b3f` | After, `f79bda27a8fb2f1f` | Change |
|---|---|---|---|
| 1 client, run 1 and run 2 | 1,400.29 and 1,401.22 ms | 1,036.30 and 1,021.16 ms | 26.6% faster |
| 8 clients, run 1 and run 2 | 3,180.82 and 3,214.16 ms | 3,147.74 and 3,151.03 ms | 1.5% faster |
| Queries per second at 8 clients | 2.5 and 2.5 | 2.5 and 2.5 | none |
| Recall | 1.0 | 1.0 | none |
| Resident memory | 1,566 and 1,573 MiB | 1,561 and 1,567 MiB | none |

TurboQuant, one run each, recall unchanged at 0.9915 and 1.0: p50 went from 66.76 to 65.99 ms at
rerank-40 and from 110.24 to 101.61 ms at rerank-400, with one client. At 8 clients it went from
120.70 to 117.64 ms at rerank-40, and rerank-400 moved from 456.59 to 478.82 ms, within the
spread a single pair of runs can show.

## What the result means

The fix helps a single client by more than a quarter, which clears the 10% condition. It does not
help under load. At 8 clients exact search stays at 2.5 queries per second on either build, so
something other than digest formatting limits it there. That bottleneck is unidentified, and it is
the next thing to find if throughput under load matters. This is reported rather than hidden: the
brief's condition did not name a client count, and read at 8 clients it is not met.

## Surfaced failures and corrections

An earlier comparison campaign on `clyffy-01` was slowed by other GPU work on the same host. These
runs alternate builds and carry host samples, and none shows the GPU busy.

## Checks not run and remaining errors

- Candidate CI has not run.
- No profile was taken after the fix, so the remaining share of authenticated time is not stated.
- The 8-client limit is not diagnosed.
- POAM-035 and C-07 stay open. Exact search still authenticates every change one at a time.

## Review

- **Full-file reread and diff review:** complete for both changed files.
- **Change checklist:** complete. The brief came before implementation; no contract, format or
  authority changed.

## Status change

None. Pull request, CI run and merge evidence are recorded on
[#257](https://github.com/rrflow/rrflow-development/pull/257).

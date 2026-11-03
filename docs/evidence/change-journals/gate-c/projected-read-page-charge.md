# Charging a projected read once for each page it covers

**Status:** active immutable change-journal evidence; POAM-035 and C-07 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/projected-read-page-charge`
**Owner:** OBJ-02 / C-07 / POAM-035 / PR259; Gate C retains completion authority

## Package

- **Advances:** OBJ-02 resource bounds, C-07 growth qualification and POAM-035. It also
  serves OBJ-13, because a TurboQuant build goes through this read.
- **Starting revision:** `ed3027b79e41dfcdf5b27e57f07346f11f6da347`.
- **Pull request:** [#259](https://github.com/rrflow/rrflow-development/pull/259), found
  through the comparison lab in draft #253 under the owner's 2026-11-03 direction to tune
  from its results.
- **Current to target:** a projected read charged the whole logical size of the validity,
  offset and value pages again for every row, so a read of about 2.1 MB of rows was charged
  125,344,200 bytes and a search of 200,000 points met the 8 GiB `page_logical_bytes`
  ceiling. One row group's rows now charge each page's logical bytes once, as the key spine
  already did.
- **Unchanged:** every ceiling's value, returned rows, page request counts, cache touches,
  memory held, segment format and cache policy.
- **Stop conditions:** a returned row changing, request counts or cache behavior changing,
  or the fix needing a ceiling raised. None was reached.

## Files read

`docs/reference/storage/rrflowkv-current-format.md` (limits, evidence, cache),
`docs/poam/rrflow-1.0-alpha/poam-035.md`, `segment/reader.rs` (budget, stream, cursor,
meter), `segment/mod.rs` (`load_page_with_access`, `KeySpine`, cache lookup),
`segment/source_memory.rs`, `segment/physical_policy_lab.rs` (cache policy oracle),
`transaction/scan.rs` (`bounded`), `rrd-store/src/access/versioned_read.rs` (`scan_source`),
`tests/projected_output_limits.rs`, `tests/hybrid_segment.rs`, `tests/page_cache.rs`, and the
error labels in `rrflow-cli/tests/support/sustained/workload.rs`.

## Changed paths

Created `crates/persistence/rrd-lsm/tests/projected_read_page_charge.rs` and this journal.
Changed `crates/persistence/rrd-lsm/src/segment/reader.rs` (`SegmentProjectedCursor`,
`RunCursor`, `Segment::load_projected_value_page` and `load_metered_page`),
`docs/reference/storage/rrflowkv-current-format.md` (what a page charge means), and the
generated indexes.

## Decisions

- **Research:** none needed. The change adapts no upstream code.
- **Trace, resource, debugging:** preserved. `ProjectedReadEvidence` already reports page
  requests, logical bytes, cache activity and physical reads, so no trace is added. Only
  the meaning of `page_logical_bytes` changes, and the format reference states it.
- **Charge once rather than keep pages:** see the first correction below.
- **Risk accepted for review:** this loosens one resource bound. When a page is not in the
  cache, for example when concurrent scans evict it, a row really does read it again, and
  that read no longer adds logical bytes. `max_page_requests` (4,000,000) still bounds such
  a read and physical reads still appear in the evidence, but the 8 GiB ceiling no longer
  limits this repeated work. The old charge limited it only by being wrong by a factor of
  the rows per page.

## Oracle and results

The six tests in `projected_read_page_charge.rs` were run unchanged against the starting
reader, in a scratch worktree at `ed3027b7`. Three fail there and pass after the fix:

| Test | Starting reader | Head `2f84fdf0` |
|---|---|---|
| `a_scan_is_charged_once_for_each_page_it_visits` | failed: 125,344,200 logical bytes for 2,000 rows in 33 row groups | passed |
| `interleaved_runs_are_charged_once_for_each_page` | failed: 124,155,200 bytes for 2,000 rows in 34 row groups | passed |
| `superseded_versions_are_not_charged_for_their_value_pages` | failed: 125,396,464 bytes | passed |
| `a_key_scan_is_never_charged_for_value_pages` | passed, as a guard | passed |
| `the_page_ceilings_still_refuse_one_unit_below_the_charge` | passed, as a guard | passed |
| `charging_once_does_not_change_what_a_scan_holds_or_releases` | passed, as a guard | passed |

At head `2f84fdf0fbf7d254678d1cf65c1cbd92a1c1accd`:

- `cargo test --locked -p rrd-lsm --no-fail-fast`: 252 passed, 0 failed, 3 ignored.
  `tests/page_cache.rs` is unchanged and passes.
- `cargo test --locked -p rrd-store --no-fail-fast`: 255 passed, 0 failed, 2 ignored.
- `cargo test --locked -p rrd-query --no-fail-fast`: 72 passed.
- `cargo test --locked -p rrd-engine --lib vector --no-fail-fast`: 55 passed.
- `cargo clippy --locked -p rrd-lsm --all-targets -- -D warnings`: clean.

Real runs on `clyffy-01`, one system at a time, on an engine with SHA-256 prefix
`4c08aeaafe452630` built from that head, with `projected_output_bytes` and
`source_page_bytes` raised to 16 GiB:

- **200,000 points of 384 dimensions, `runs/vector-medium-charge-once`.** The read passes
  the 8 GiB ceiling that refused it before. The TurboQuant build then stops at a different
  limit, which this package does not touch:
  `ConfigurationLimit("runtime versioned read exceeded key budget 1000000: observed 1000001")`.
  [POAM-054](https://github.com/rrflow/rrflow-development/pull/253) in #253 records it.
- **100,000 points, `runs/vector-100k-1`, report SHA-256
  `5cd3d8b0775456dfb04aad1e94e301147babf7116bb589a579cffedb00aff83d`.** These are RRFlow's
  first answers at this size. Exact search returned recall 1.0 at a p50 of 10,953 ms plain
  and 10,699 ms filtered. TurboQuant returned recall 0.99 at rerank-40 with a p50 of 472 ms.
  #253's journal holds the comparison and its limits.

## Surfaced failures and corrections

- **The first design failed an accepted test.** The first brief had the cursor keep its
  value pages, as it keeps the spine. That removes real per-row work, but it also removes
  the repeated touches behind `same_scope_hits`, so
  `scan_resistant_cache_preserves_reused_pages_across_a_mixed_family_scan` failed at
  `page_cache.rs:124`, and `physical_policy_lab.rs` treats zero as a failed cache policy.
  The test was not edited. That version would change accepted C-06 evidence, so it is
  outside this package, and the brief was revised to charge once. Commit `3ddaac94` keeps
  that version for the record. On `clyffy-01` it reached the same key budget at 200,000
  points, in `runs/vector-medium-page-charge`.
- **A test run looked complete and was not.** `cargo test` stopped at the first failing
  binary, so later targets had not run. The results above use `--no-fail-fast`.
- **The fix did not reach 200,000 points.** It removes the ceiling it set out to remove, and
  the next limit is a public contract constant, which is the owner's decision.

## Checks not run and remaining errors

- Candidate CI has not run. The SDK suites and the generated-surface check were not run
  locally, because no contract or generated surface changed.
- Whether keeping pages helps at 8 clients is unmeasured.
- Ingest was not compared under control. The 200,000-point run ingested 462 points per
  second here and 472 to 571 on an older revision, in separate campaigns. This package
  changes only the read path, and the difference is unexplained.
- POAM-035, C-07 and POAM-054 stay open. Exact search and TurboQuant builds still read every
  stored vector in one read.

## Review

- **Full-file reread and diff review:** complete for the reader's diff, the test file and
  the format reference.
- **Change checklist:** complete. The brief came before implementation and was revised in
  the pull request when the design changed; no second store, format or authority; no
  contract change; no ceiling changed; the accepted cache test is unedited.

## Status change

None. No roadmap, objective or POA&M status changes. Pull request, CI run and merge evidence
are recorded on [#259](https://github.com/rrflow/rrflow-development/pull/259).

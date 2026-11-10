# C-07: size the owned-volume fill budget from hosted-runner evidence

**Status:** active fixture repair; no native matrix, soak or C-07 acceptance claimed

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-macos-fill-bound-20261109`

**Owner:** C-07f, OBJ-02, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)

**Gate/package and outcome advanced:** C-07 native capacity fixture; OBJ-02 installed durability evidence. The candidate `installed domain crash (macos-latest)` job stops failing on a fixture budget while the owned volume still accepts creates.

**Starting revision and pull request:** public main `953fe774` (merge of #251); branch `agent/c07-macos-fill-bound-20261109`.

**Change brief.** Current: `Filler::fill` starts one 25-second wall clock that the data phase, its sync barriers, every joint settlement round and the metadata fill share. Target: the same clock bounded by one constant `FILL_BOUND` of 120 seconds, sized from retained hosted-runner evidence; bound panics report bytes, files and elapsed time; expired-clock tests derive their clock from the constant through `expired_fill_clock()`. Unchanged: the 8192-entry bound, the aggregate byte bound, capacity-code classification, settlement rounds, every recovery oracle, every engine, client, controller and CI deadline. Stop conditions: any oracle other than the wall budget would change, or the macOS job on this branch fails with the volume still accepting creates.

**Files and symbols read in full:** `crates/adapters/rrflow-cli/tests/support/capacity.rs` (`fill_data_with_probe`, `allocate_metadata_payload`, `settle_pressure`, `Filler::fill`, `Filler::fill_metadata`, `resume_after_fill`, `capacity_case`, the `filler_tests` module), `capacity_metadata.rs` (`fill` and its tests), `capacity_apfs.rs`; `scripts/ci/run_native_capacity.py` timeouts; `.github/workflows/ci-reusable.yml` job timeout (75 minutes); POAM-035 native capacity sections.

**Paths changed:** `crates/adapters/rrflow-cli/tests/support/capacity.rs` (constant, helper, three bound sites, two tests), `crates/adapters/rrflow-cli/tests/support/capacity_metadata.rs` (import, bound site, one test), `docs/poam/rrflow-1.0-alpha/poam-035.md` (one paragraph), this record.

**Research decision and sources.** No upstream code adapted. Evidence is retained candidate CI output only:

| Run | Job outcome | Metadata phase |
|---|---|---|
| 42656889121 (#236) | failed | `files=589 creates=604 barriers=19 create_us=22634180 settle_us=912592 elapsed_ms=25000` |
| 42685064671 (#283 merge queue) | failed | same bound, log not yet retained at writing time |
| 42416257573, 42403138769, 42400526130, 42332508492 (2026-11-07) | failed | `files=0 creates=0 ... elapsed_ms=25000` and `bounded joint pressure`: the clock was spent before the metadata fill began |
| 42668299956 (passing) | passed | `files=876 creates=1018 barriers=147 create_us=13553391 settle_us=2464841`, data phase 3702 ms |

Per-create latency on the near-full APFS image is 15-40 ms; a healthy fill needs 600-900 files before the first native denial, so a 25-second budget sits inside the normal fill envelope. Five of the 200 candidate runs before 2026-11-09 failed this way. With the bounds that remain, the worst case is 8192 creates at 40 ms, about 5.5 minutes, inside the 75-minute job limit.

**Trace/resource/debug decision:** preserve. The existing `native_capacity_data_phase` and `native_capacity_metadata_phase` stderr records already carry the latency data used here; the joint-pressure panic now carries progress too. No new trace surface.

**First-failure oracle and result:** the failing run above. On this branch the owned-volume native case runs only on the candidate macOS job; the Windows unit tests cover the bound sites and the expired-clock paths.

**Acceptance commands and exact results (Windows, own target directory):**

- `cargo test --locked -p rrflow-cli --features crash-test --test installed_crash -- capacity::`: 21 passed, 0 failed, 1 ignored (runner-owned image), 7 filtered out; 7.38 s. Includes `cannot_exceed_byte_cap_or_restart_expired_deadline`, `metadata_refills_share_the_original_byte_and_time_bounds`, `settle_pressure_stable_state_stops_and_expired_clock_has_no_effect`, `metadata_expired_clock_and_entry_bound_prevent_further_effects`.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked -p rrflow-cli --features crash-test --test installed_crash -- -D warnings`: the first pass failed on one unused `Duration` import after the literal moved behind the constant; corrected, then passed. The capacity tests were rerun after the correction: 21 passed, 0 failed, 1 ignored.
- `python scripts/knowledge/render_navigation.py`: OK, 1 index file updated. `python scripts/ci/check_documentation.py`: OK.

**Surfaced failures and corrections:** the first test build omitted `--features crash-test`; rerun with it. Unused import corrected as above.

**Checks not run and reason:** the native macOS owned-image case needs the runner-owned APFS image; it runs in the candidate job for this branch, not locally. No Linux or Windows native capacity rerun: their bound sites changed only in value and message.

**Remaining known errors and owner:** the macOS `hdiutil create` `Resource busy` failures and the 2026-10-24 attribution gaps remain open under POAM-035 and C-07f.

**Full-file reread and diff review:** both support files reread after the edit; the diff touches only the constant, the helper, the bound assertions and the three expired-clock tests.

**Change checklist:** owner bound (C-07f/POAM-035); current and target behavior stated; scope limited to two fixture files and two records; smallest oracle first (unit tests), then clippy and format, then documentation checks; evidence retained; no status change claimed.

**Roadmap/POA&M status change:** none. POAM-035 gains one paragraph linking this record; C-07 stays open.

**Pull request, CI run and merge evidence:** recorded on the pull request when its candidate run completes.

# D-01 explicit source-read CI correction

**Status:** active corrective evidence; native isolated candidate CI remains pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-01-explicit-source-read-ci-correction`
**Owner:** PR291 installed-lifecycle CI repair

Starting native candidate: commit `209e1368378e1e987154525cce4cdd206cd7c25d`,
tree `351ae5bde8df0b3ce713c3db3642d32ab4368dc5`. The repair scope was
published and verified on the existing draft PR291 before edits.

## Scope and preserved history

This correction repairs PR291's isolated lifecycle harness and a formatting-only
metadata defect. It makes no runtime or public contract change. The original
[source-read implementation journal](d-01-explicit-source-read-install-choice.md)
remains in Git ancestry with its original claims. Its baseline bytes had SHA256
`ecfe7c2229484cdd72fa180afc1646be085447dd1add472c3e1a524c1ddcc1ad` and
9,730 bytes. The corrected file has SHA256
`d726ec64685a28a02ac2174906bdfda90ea08302973aa05f17f278bee2b02fad` and
9,726 bytes. The only content change removes two trailing spaces from each of
the Status and Coordinate metadata lines (four bytes total); prose claims are
unchanged.

## Candidate CI evidence and scope

The published PR291 CI amendment records the first candidate's actual result:
Linux's ordinary installed lifecycle test passed 8/8; Linux and macOS isolated
preflight rejected the compiled harness because the exact required case list
still named seven tests. Linux, macOS, and Windows isolated preflight jobs
rejected the compiled harness because the exact required case set was stale.
Three workspace-architecture jobs rejected the two trailing-space sequences
above. The new installed test's runtime reads of two
canonical contract fixture files were found by source inspection; preflight
stopped before isolation, so this was a predicted incompatibility, not an
observed isolated fixture failure. The original candidate logs remain in the
private local evidence set `install-source-access/ci-first-job-*-raw.log`:

| Log | SHA256 |
| --- | --- |
| `ci-first-job-128351967330-raw.log` | `d5196483ba45b6b47522a921a8491937e922625cb20b1430608fa5a07312c376` |
| `ci-first-job-128351967343-raw.log` | `790ceb4e050472081ecb2b5ed7871ca25da13788cd9d2f94a5e702dc6d5e53d2` |
| `ci-first-job-128351967333-raw.log` | `6590a2a078cd4e884a040e2c12d53aa3e890a0babcc4b788c4ea61c3931aa5f4` |
| `ci-first-job-128351967386-raw.log` | `43837ffdc42c942396cb5173996a153ef39ba2aefe3dbd1c879d0530bbc579e9` |
| `ci-first-job-128351967413-raw.log` | `f1b942140036077e69bf01021ea391a09844e4c4ca8a098a5239e02beb7a1f54` |
| `ci-first-job-128351967439-raw.log` | `d7f1a210167bbe0d3b4b53b3b26c002e3d99423f5c27c54f67dfb0a95410512a` |

## Corrective changes

- Add exactly `installed_source_read_choice_is_explicit_and_reopens` to the
  runner's required lifecycle set.
- Add the new name to the independently enumerated Python case set and reject
  its omission and duplication in both harness listing and test-result
  oracles. Existing exact-set, cardinality, no-skip, isolation and resource
  controls remain in place.
- Embed the unchanged canonical endpoint configuration and allowance bytes
  from `rrd-contract` with compile-time `include_bytes!`. The isolated test
  executable no longer reads these fixture files from a runtime checkout.
  All authorization, source eligibility, no-dispatch, success, and reopen
  assertions remain unchanged.
- Remove only the four whitespace bytes from the original journal metadata.

## Validation and remaining proof

The pre-edit `python scripts/ci/test_offline_lifecycle.py` ran 27 tests: 20
passed and 7 skipped. After the changes it ran the same 27 tests: 20 passed and 7 skipped,
including exact-set, missing-case, and duplicate-case controls for the new
name in both the harness listing and execution results. Cargo checks used
`CARGO_BUILD_JOBS=1` and the authorized `source-bound-worker-worktree/target`
cache. The following bounded checks passed; host-captured output, exit status,
and source-hash records are retained in the ignored private local evidence set
`install-source-access`:

| Command | Result |
| --- | --- |
| `python scripts/ci/test_offline_lifecycle.py` | 20 passed, 7 skipped (27 run); exit 0 |
| `cargo test --locked -p rrflow-cli --test installed_lifecycle installed_source_read_choice_is_explicit_and_reopens -- --nocapture` | 1 passed, 7 filtered; exit 0 |
| `cargo test --locked -p rrd-engine --test workspace_architecture` | 33 passed; exit 0, including tracked/untracked trailing-whitespace check |
| `cargo test --locked -p rrflow-cli --test installed_lifecycle --no-run --message-format=json` | Exit 0; compiler-artifact record identified one lifecycle harness, SHA256 `77a1828cde81dea7c3ff4921dcb891e706b8c2919a3eace01fc9d1ecd1f1dabc` |
| `python scripts/ci/run_offline_lifecycle.py --check-harness <compiled lifecycle harness>` | Exit 0; exact eight-case listing accepted, `required_cases` 8 |
| `cargo fmt --all -- --check` | Pass; exit 0 |
| `python scripts/ci/check_generated_surfaces.py` | Pass; exit 0 |
| `python scripts/knowledge/render_navigation.py` and `--check` | Pass; exit 0 |
| `python scripts/ci/build_execution_inventory.py` and `--check` | Pass; exit 0 |
| `python scripts/ci/check_documentation.py` | Pass; exit 0 |
| `git diff --check` | Pass; exit 0 |

The compiled harness preflight verifies the exact eight-case listing only; it
is not an OS isolation run. No new Linux, macOS, or Windows enforced isolation
result is claimed here. Those platform CI runs remain required and
parent-owned. No production API, runner isolation, resource bound, authorization
or installed-test assertion was weakened.

## Next action

Parent owns final candidate rebuild, PR/CI publication, platform-isolation
receipts, and downstream UI pin refresh. Do not reinterpret local preflight or
ordinary process tests as enforced isolation evidence.

## Review record

The complete diffs for the runner, its Python tests, installed lifecycle test,
and both journal records were reread. Planner reviewed the four primary source
files and found no remaining issue. `cargo fmt --all -- --check`, the
workspace-architecture whitespace oracle, generated-surface check,
documentation-policy check, navigation/inventory checks, and `git diff --check`
passed. No roadmap or POAM status changed. No external research, runtime trace,
or performance claim was needed. Local `--check-harness` preflight is not an
isolated platform result; the remaining Linux/macOS/Windows candidate CI stays
parent-owned.

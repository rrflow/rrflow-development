# Recovered lab and standalone database behavior

**Status:** active recovery evidence; historical source reconciliation remains incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/recovered-lab-and-database-paths-20261120`
**Owner:** [POAM-011](../../../poam/rrflow-1.0-alpha/poam-011.md), [Gate H](../../../roadmap/rrflow-1.0/gate-h.md) and [Gate J](../../../roadmap/rrflow-1.0/gate-j.md)

## Brief and method

[PR451](https://github.com/rrflow/rrflow-development/pull/451) contains the
brief before this record was written. Baseline main is
`fe3217570e46a03a8277de6e2376bdba65acb148`; opening revision is
`d59b356d7f7818e3ec14e9a0269d80992b12c183`. This continues the owner's recovery
session and the [initial inventory](historical-pr-recovery-20261120.md).
Scope is this record, its generated index and the POAM-011 evidence link.
No engine implementation, prior journal, gate status or Git history changes.

Recovered heads below are immutable source inputs, already retained in the
verified recovery bundle described by that initial record. Patch comparison
and selected full-file reads are distinguished below. A matching added line
is inventory evidence; the source review and tests establish only the named
behaviors. The current canonical layout remains; an abandoned old layout is
not restored to make a closed PR appear merged. Unexplained source loss or a
failed oracle stops the affected claim. Existing contracts, local source and
native tests suffice for this audit; no external implementation, new research
claim, runtime telemetry or benchmark is introduced.

## Lab vocabulary, definitions and layout

| Input | Recovered head | Current boundary and disposition |
|---|---|---|
| PR364 | `12722a07b7978c814e383e5cd6c6e7d2a6a3528e` | Old `crates/evaluation/workbench/lab/rrflow-lab-core` becomes `crates/evaluation/rrflow-evaluation-core`. IDs, digests, availability, requirements, canonical JSON and report types remain. Current metric/schema extensions remain too. |
| PR365 | `854ffd4ff86db6b5bf9c335db0787ff17b596305` | The old `eval/workbench/lab` relocation was superseded: data/scripts are at `eval/lab`, schemas at `eval/schemas`, and Rust lab crates under `crates/evaluation/lab`. No missing runtime file was found in the compared 94-file lab set. |
| PR274 | `c870dd77732683f0e0aea16f0a6d6899ead3fa40` | Installed definitions, settings, signed-pack operations and automatic planning remain in the split lab crates and existing `rrflow-lab` CLI. No second lab or definition authority is added. |

For PR364, read current core `lib.rs` (157 lines), `report.rs` (350 lines),
its Cargo manifest and the historical implementation patches; the generated
lockfile was dependency inventory, not a full line review. The old report
text is unchanged. Read the later evaluation-core, evaluation-split,
lab-tool and evaluation-layer-rules journals. They record the earlier failed
layout candidate and subsequent canonical split. Imports identify current
consumers, but not every consumer's full source was reread.

For PR365, `pr365-current-lab-comparison.json` in the scratchpad records 94
mapped files: 62 byte-identical, 17 differing only by relocation and 15 with
additional differences. Read all 15 changed-file diffs and the 30 remaining
non-lab patches. Differences include corrected relative roots, estate-to-engram
terminology, generated HNSW build policy, max-level/seed settings and catalogue
digests. The old task catalogue agrees after the terminology change. The old
26-line `gate-j/workbench-lab-data-20261112.md` journal is absent from current
main; it described that superseded layout. Its recovered contents were read,
and the bundle retains it. Its old generated index row is likewise absent.
This is an explicit historical-record disposition, not a claim that the file
was retained or a reason to restore a second layout.

Read complete current `rrd-operator-knowledge` knowledge projection source,
lab `cli/store.rs` discovery, `eval/lab/pod.sh`, and lab contract tests lines
1-200 and 900-988; read the script root-discovery changes and the complete
Unix-only runner test file. The latter is excluded on Windows, not a passing
native runner test. This review did not reread all bytes of all 94 files.

For PR274, read complete current definitions `home.rs` (474 lines), lab
`settings.rs` (639), `cli/store.rs`, `cli/settings.rs`, `cli/definitions.rs`
(393), `cli.rs`, argument definitions and the added command dispatch ranges.
Whole-function comparison found 24 of the 25 added CLI functions textually
retained, ignoring visibility; `known_system` changes its return type's crate
path. Home changes only imports. Settings adds current HNSW max-level/seed
configuration. Read the complete old catalogue/content/pack patches and
current pack diff; the trusted-key JSON is byte-identical. `run_all.sh` retains
its added script text; `run_system.sh` retains the additions apart from a
comment. Read the complete old-to-current automation-test diff, not all 1,276
old test lines. Added HNSW cases and the now-unavailable routed fixture explain
the substantive test changes. Read the old automation journal and README
patch; the journal is retained exactly. Its user-folder ACL and single-key
limitations remain stated and are not treated as product qualification.

On source `906d68422f859b89d5547851d90e84912eed7b7a`, Windows x64,
`CARGO_BUILD_JOBS=4`, target `D:/.usr/Temp/rrflow-selective-recall-audit-20261120/target`:

- `cargo test --locked -p rrflow-evaluation-core -p rrflow-lab -p rrflow-lab-benchmarks -p rrflow-lab-datasets -p rrflow-lab-definitions -p rrflow-lab-evals -p rrflow-lab-systems -p rrflow-qualification-storage`: 265 passed, zero failed/ignored, 30 targets. Log `pr364-365-evaluation-suites.log`.
- `cargo test --locked -p rrflow-lab-experiments`: seven passed. Log `pr364-experiments.log`.
- `cargo test --locked -p rrd-engine --test workspace_architecture`: 35 passed. Log `pr364-workspace.log`.

These include generated schema, comparator, lab discovery, installed-definition,
no-provider-spend planning and storage-golden checks. They do not run the
Unix-only runner, live model providers or a competitive benchmark. Lab source
is unchanged between this tested revision and this package's baseline.

## Native database and SDK harness recovery

PR189 head `c15ed0b923dfb56ff947be0e14c04b10d8c650a1` combines the cache
experiment, native private manifest, inventory and standalone database work.
Read its source/workflow/policy patches, complete current
`scripts/ci/sdk_private_manifest.py` and `run_rrflowdb_standalone.py`, and
old-to-current diffs for the SDK server and four affected Python test files.
The 184-line private-manifest module is byte-identical. Its Rust staged
publication implementation and denial test remain; later changes add cold
fixture setup, engram naming and current vector-index grants. Current script
changes extend independent expected readback to documents and their
relationships and isolate the fixture's instance registry.

Read all 13 linked historical journals, the remaining authored documentation
patches, the cache workflow diff, and current macOS cache-policy function.
The added policy line now admits the exact earlier toolchain-isolation block
before the unchanged locked build; negative tests enforce that order. All
other old added lines in these source/workflow/documentation surfaces remain,
allowing the explicit terminology change. The current standalone script's four
changed stamp-check lines extend the check to all four results. Generated
inventory rows are intentionally replaced by the stable descriptor and current
Git-derived output, as the owning inventory journals explain.

On `1ab971781a32ed857ec242cef96e392309bc65b6`, the integrated PR448 tree:

- From `scripts/ci`, `python -m unittest test_execution_inventory test_sdk_private_manifest test_rrflowdb_standalone test_cache_warm_policy`: 32 ran, 31 passed, one platform skip, zero failures; `pr148-189-harness-tests.log`.
- `cargo build --locked -p rrflow-cli --bin rrflow -p rrd-client --example rrflowdb`, then `python scripts/ci/run_rrflowdb_standalone.py` with the two executables under the scratch target: both exited zero. The actual fresh install, authenticated task/document and relationship operations, abort, wrong-state denial, offline verify and reopen passed at cursor 9. Logs `pr189-standalone-build.log` and `pr189-standalone-real.log`.
- The primary executable SHA-256 is `769e0af7991e63a2e311ce6d2e5329828fb35bec78e0eee35e4ae5d483d0eea6`; the example is `629f6bfb7bae1aa2c273fb2f748e204fbbb844f76899abb52fe0556af6898e2a`.

On `23573d30726fd9ef10ea75acdcd63d389c4b9627`, which only records further
results over that integrated source, `cargo test --locked -p rrd-client
--example sdk_conformance_server` passed the one native staged-publication
negative test (`pr189-sdk-staging.log`). This is not a new six-language SDK
corpus, separate-user access test, cloud deployment or capacity qualification.
No fresh macOS cache timing or cache-hit claim is made.

## Planning and historical failure records

| Input | Head and examined boundary | Disposition |
|---|---|---|
| PR24 / PR25 | `f661cc965093f9ed4b3ae7a222da35c65599b43e` / `d134734bd765545475d5d6020b22749feae8045e`; all authored patches, current complete PostgreSQL contract and three associated journals; current README identity, system overview 149-171, H-08 package table and dependency-spine clarification | PostgreSQL profile, unsupported-feature denials and independent release requirements remain. Later SQLite scope and package dependency clarification extend them. These are requirements, not a working PostgreSQL server or qualified compatibility. |
| PR148 / PR197 | `326957e9716225b9e18246dc0298c1cfde9dca1d` / `e3eaa78b3272bd19b58f93cfd6c8a6365ac4a01d`; complete current source-bound design, related design/inventory journals and historical added-line/current-diff comparisons | Immutable jobs, explicit successors, evidence/retry identity and current Git inventory remain. PR197's changes are design text. The inventory's native tests above pass; they do not prove the future plan runtime. |
| PR316 | `5741a7660e244e8bd71425c029d93a41f0420a35`; all new journal bodies, remaining owner/index patches, both changed HNSW tests | Source-plan/parser reconciliation, HNSW query and cache/standalone evidence remain. `cargo test --locked -p rrd-engine --lib hnsw_read_path` passed 14 cases on `0c121584822e652fa499d270616927d1c61ca3c5` (`pr316-hnsw-tests.log`). This is bounded read-path evidence, not implementation of every design record. |
| PR151 | `9aea9987b21e4437c5f6d96a5c5832da9e00939f`; all authored Windows launch/progress/terminal and failure patches | Historical allocation/read-pin failures and host-interference limits remain. They are not relabeled successes. No capacity campaign or historical artifact-hash verification was rerun here. |
| PR286 | `6795422b7b9ed1187af60f707576473f99866084`; complete flush-stall journal | Retained journal differs only in two corrected POAM links. POAM-087 holds the observed maintenance/search stall; no exclusive-lock or latency repair is claimed. |
| PR300 / PR344 | `d26a1322dd5d87ec19b417420a5074c077e09de1` / `81059c9df60c64567414d14d0ed70f6dd10dc1a7`; complete old patches and journals, current native-capacity lines 214-307 and 339-428, offline-lifecycle 280-342 | Current harness retains helper ownership and bounded release before retry. The later pristine-image-per-lane policy supersedes the old zero-wait create behavior. Python native-capacity/offline-lifecycle suites ran 78 cases, 65 passed and 13 platform skips (`pr300-344-native-harness.log`); this is Windows harness evidence, not a real macOS capacity pass. POAM-074 remains open. |

The PR151 records retain the failed 100k campaign and later 48,604/47,016
post-reopen verification observations. Concurrent host work limits interpretation;
none establishes accepted C-07 qualification. Historical file-plan row churn is
a generated projection, not lost implementation. The old source-plan additions
are traceable through current design/owner records but do not promote D-02.

## Remaining work and checklist

PR315's diagnostic source comparison exposed a real valid-time/observation-time
bug; [PR448](https://github.com/rrflow/rrflow-development/pull/448) and the cold-memory journal on that PR own that implementation repair. Its first full engine run exposed the separate
Windows replacement failure repaired and merged by PR450. The combined suite
passed 591 cases, zero failed, 12 existing ignored. This package does not
transfer those results to a different untested engine revision.

PR231's large controller/endpoint-worker change, the remaining parser/clock
stash deltas, most of Connectome PR10 and ancestor-only inventory items are
not exhaustively reconciled. The separately reviewed child knowledge and SSH
components do not establish the full old UI. No broad absorption, feature
parity, current visual qualification or cloud synchronization claim follows.
Official-repository refs remain subject to their separate no-push restriction.

Checklist: before-edit PR brief; immutable inputs and canonical destinations;
reviewed source versus inventory distinguished; retained failures and skipped
platforms; exact native process result; linked journal and generated index;
no status promotion. Documentation policy, navigation and diff checks plus
exact candidate/merge-group CI are required before landing. Continue with
unreviewed controller and parser/clock behavior, then the missing UI paths.


The first documentation check rejected a local link to PR448's new journal:
that independent candidate had not landed in this package's baseline. The
reference now names PR448 rather than pretending its file exists on main.
The original failure remains in `recovered-lab-db-docs.log`; the corrected
check is recorded separately. No product code changed to satisfy this check.

Corrected documentation policy passed 1,287 statuses and 1,283 classified
coordinates (`recovered-lab-db-docs-final.log`). Navigation check passed 21
indexes, 1,284 nodes and 4,521 edges with no drift; diff whitespace passed.
Exact candidate and merge-group results remain pending at authoring.

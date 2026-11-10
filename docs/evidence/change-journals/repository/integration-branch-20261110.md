# One integration branch: every open candidate absorbed, nothing cut

**Status:** active integration evidence; build-and-test absorption, no gate acceptance claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/integration-branch-20261110`

**Owner:** the repository owner's 2026-11-10 direction that every open candidate lands through one branch with no content removed; `docs/operations/ci.md`

**Starting revision:** main `da3edfb8`.

**Current behavior.** Thirteen candidate pull requests were open against main on 2026-11-10, each rebased on a different main, several touching the same generated projections, test manifests and the POA&M ledger. Merging them one at a time through the merge queue re-ran the full matrix per candidate and re-conflicted the survivors after every landing.

**Target behavior.** One branch, `agent/integration-20261110`, that contains every candidate's commits by merge (not cherry-pick, not rewrite), with each conflict resolved as a union and the generated projections regenerated once from the merged catalogue.

**Absorbed, in order, with the conflict each one raised.**

| PR | branch | head | conflicts and resolution |
| --- | --- | --- | --- |
| #236 | `agent/adopt-existing-rrflow` | `ab465a11` | none |
| #241 | `agent/d04-source-spans` | `98a4ba85` | `rrd-contract/src/lib.rs` export list: union of both identifier sets |
| #247 | `agent/existing-project-inherited-access` | `be57a00c` | `engine/estate_layout.rs`: #236's adoption arms kept beside #247's `estate_directory_error` |
| #249 | `agent/lfg-engine-admission` | `e0df0db7` | OpenAPI digest constant and every generated SDK/doc projection: taken from the candidate, regenerated below |
| #261 | `agent/c07-targeted-native-preflight` | `26706814` | none |
| #269 | `agent/f06-parquet-cold-storage` | `38284c4b` | `Cargo.toml` dependency union (`parquet` and `wat` both kept); `installed_lifecycle.rs` and the offline harness `REQUIRED_CASES`: union of the adaptive-selection case and the cold-history case; projections regenerated |
| #304 | `agent/vector-source-binding-20261107` | `331871c2` | none |
| #305 | `agent/adaptive-attunement-selection-20261107` | `47c950ce` | none |
| #306 | `agent/lab-integration-20261109` | `585ef354` | POA&M: #304 and the lab branch both allocated POAM-053. Main already cites POAM-053 as the lab's "clients cannot build RRFlow's HNSW or payload indexes" record, so that number stays with the lab entry; #304's "named HNSW generations lack an authenticated source identity" becomes POAM-058, and its journal, the gate-e index and `docs/reference/vector/search.md` cite the new number. POAM-054 to POAM-057 carried unchanged |
| #307 | `agent/ci-pr-governance-20261109` | `7771ceab` | none |
| #309 | `agent/c07-macos-fill-bound-20261109` | `ccbe86cc` | none |
| #311 | `agent/ci-repository-invariants-20261110` | `f07193c1` | none |
| #312 | `agent/connectome-repin-20261110` | `8fe5bf31` | none; gitlink `connectome` at `19ee075c` |
| #300 | `copilot/fix-installed-domain-crash` | `d26a1322` | its retry loop was already superseded on main (five attempts, exponential backoff); the one remaining behaviour, detaching and removing a leftover image that is an ordinary file inside the fixture before each retry and refusing an escaped path, is carried as a direct commit with two tests |

Not absorbed, with the reason: #231 (its job-list operation already exists on main), #234 (empty), Connectome #42 (superseded by Connectome #45, merged and repinned by #312), and the lab drafts #253/#260/#262/#267/#268/#272/#274/#277 (contained commit-for-commit or by record in #306).

**Collisions found by the merged build, and how each was settled.**

- Taking the candidate's whole `crates/transport/rrd-contract/src/lib.rs` for #249 and #269 (to carry their OpenAPI digest) dropped the exports #236 (`ADOPTED_ESTATE_DIRECTORY`, `MAX_ADOPTED_ESTATE_ENTRIES`, `validate_adopted_estate_entries`), #241 (`SourceParseBinding`, `SourceParsePolicy`) and #249 (`LfgAnalysisPolicy`) had added. The merged engine did not compile until they were restored. A line-level audit against every candidate's base on main then found no other hand-written added line absent from the branch; the only differences are the POAM-058 renumbering, verified verbatim, and the Connectome gitlink.
- #305's `attunement_adaptive_plan` test built `AttunementPhasePlan` without the `parse_policy` field #241 added; the test now sets it to `None`.
- #236 and #247 disagree by design: #247 characterized the pre-#236 installer rejecting an existing inherited-permission `.rrflow` folder before preview and asked that the error name the directory, and its brief said not to adopt #236's relocation. #236 is the owner-reported shape fix and adopts that folder. On the merged tree #247's Windows characterization `inherited_estate_preview_reports_the_rejected_directory` therefore fails because preview now succeeds. It is re-targeted as `inherited_estate_preview_adopts_the_folder_without_effects`: the same shape previews as an adoption with no entries and still has no file effects. #247's in-place preservation oracle stays as the ignored POAM-006 test; #247's directory-naming diagnostic stays in `estate_directory_error` for the non-adoptable paths.

**Regeneration.** SDK endpoint generators, `scripts/ci/check_generated_surfaces.py --write`, then the OpenAPI digest pinned into every record that cites it; `python3 scripts/knowledge/render_navigation.py`.

**Acceptance.** `cargo fmt --all -- --check` clean. `cargo test --locked -p rrd-engine --lib --test workspace_architecture --test direct_read_source_closure`: 447 passed, 13 ignored; 1 passed; 33 passed. `cargo test --locked -p rrd-contract`: all suites passed (public contract, attunement, adaptive plan, endpoint policy). `cargo test --locked -p rrd-query -p rrd-vector -p rrd-lsm` and `-p rrflow-eval`: passed (storage contract goldens, lab runner, lab definitions). `cargo test --locked -p rrflow-cli --test installed_lifecycle`: 11 passed, and its `--list` equals the offline harness `REQUIRED_CASES` exactly (11 of 11). `cargo clippy --locked --workspace --all-targets -- -D warnings` clean. `python scripts/ci/test_native_capacity.py`: 46 tests OK (5 skipped). `check_generated_surfaces.py`: 83 HTTP operations, OpenAPI `274b9c3536c1`, 7 signal projections verified. `render_navigation.py`, `check_documentation.py` (1108 statuses), `check_workflow.py` (16 substantive jobs), `test_ci_lanes_policy`, `test_select_ci_lanes`, `test_candidate_admission`, `test_linux_arm64_policy`, `test_offline_lifecycle`, `test_pr_governance`, `test_sdk_attunement_policy`, `test_macos_capacity_preflight`: OK. One full engine-library run showed `begin_timing_emits_after_admission_release_without_runtime_changes` failing once; three further full runs passed it, so it is recorded as a parallel-run flake, not fixed here.

**Research decision:** none. No upstream code.

**Checks not run:** Linux and macOS native suites, the candidate CI matrix, the offline lifecycle harness end to end (its parsers and required-case set are checked), the real-server `rrd-client` vector index tests, Connectome, soak and capacity. The Windows engine cannot see `docs/` in the retired-identity scan; Linux CI is that oracle.

**Roadmap/POA&M change:** POAM-058 added (renumbered from #304's POAM-053); no status changes.

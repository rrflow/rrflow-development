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

**Regeneration.** SDK endpoint generators, `scripts/ci/check_generated_surfaces.py --write`, then the OpenAPI digest pinned into every record that cites it; `python3 scripts/knowledge/render_navigation.py`.

**Acceptance.** ACCEPTANCE_PLACEHOLDER

**Research decision:** none. No upstream code.

**Checks not run:** CHECKS_NOT_RUN_PLACEHOLDER

**Roadmap/POA&M change:** POAM-058 added (renumbered from #304's POAM-053); no status changes.

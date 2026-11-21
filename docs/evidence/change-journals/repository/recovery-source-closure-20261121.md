# Recovery source closure and preserved exceptions

**Status:** active integration evidence; original-history exceptions remain explicit
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/recovery-source-closure-20261121`
**Owner:** repository development gate

## Outcome and ownership

Finish the original-source recovery record after PR488, with PR489 owning the recovered integration snapshots and capture-provenance repair. Baseline cebc33df1ac3ab0e54209a3544f4c882b7531653. Driver: this repository recovery session. Owner: repository development gate, existing architecture and alpha owners; no new product capability or qualification status.

## Change brief

Join original f2da9bb620d539d937e912512764658b18a361d0, including d0b897157759ec0aeba059eece1bf1f885afabd9. The only unjoined non-merge source is a historical provider permission file granting temporary script and sibling-checkout access. Preserve its parentage while keeping that obsolete file absent, as current AGENTS and ADR0004 own provider-neutral hooks and repository-closed source. No permissions are activated. Review any unexpected merge delta before resolution.

Complete the package-specific provenance map for 32 checkpoint blob versions outside recovered commit histories, plus the binary-containing history and generated runtime stash. Current reusable destinations: engine/runtime/coldlight.rs and diagnostic/storage.rs; vector/index snapshots, quantization lifecycle/catalogue/runtime and their split tests; query index/read_path.rs; existing run-real-project-workbench.ps1 and H06 workbench journal. Do not import parentless checkpoint roots containing runtime state, old PLAN/STATUS authorities, raw generated database captures or historical executables. Preserve the original recovery bundles unchanged. Record the exact original histories that remain unmerged; do not call a source port an original-head merge.

Reviewed: seven old vector checkpoint deltas against their closest recovered source versions (87,23,223,11,26,22,194 lines), current quantization publication rejection/revision-neutral restoration and 71 engine vector tests from the recorded baseline; both CLD checkpoint diagnostics and current complete coldlight.rs; four Connectome reference and three eval README deltas; three workbench-script delta sets, nine intermediate workbench-journal additions; original provider settings full 12 lines. Old global planning bodies are retained history, not new authority. Earlier package journals own parser, cold-history and vector source behavior already tested. This is bounded provenance review, not every line of all historical files.

Exact effects: this journal, a mechanical JSON source/checkpoint disposition, generated navigation, original source merges and removal of the incoming obsolete provider settings. Existing runtime and test files should remain unchanged. Stop if a new reusable implementation/assertion is discovered; port it at the current owner and validate before readiness. No research/design/performance change or comparison workload. GitHub rejects the 176054272-byte historical executable; no force push, history rewrite or attempted upload of that history.

## Proof and handoff

Acceptance: inspect all final diff paths; verify all 49 saved closed-unmerged root PR heads and all three child heads are reachable after the integration candidates land; verify all 24 non-checkpoint named root bundle refs; verify checkpoint/source mappings; focused CLD and quantization authority checks only if needed for a new delta; architecture, docs/navigation and JSON validity. No ARM, live provider, hosted sync, full parity or UI result is inferred. Candidate and merge-group gates required. After landing, fetch/prune, delete merged local branches and run the session-end checker in both repositories. Preserve bundles and explicit unresolved original-history limits. No user-owned runtime data is changed.

## Integration and explicit exceptions

Original f2da9bb620d539d937e912512764658b18a361d0 was joined at cbcaa8819700a66010fe343221216827af66942c. Its original parents e575732d0 and d0b897157759ec0aeba059eece1bf1f885afabd9 are retained. The incoming `.claude/settings.json` was removed before the merge commit: its 12 lines granted obsolete temporary-script and sibling-checkout access. No runtime or test file changed in that join. The first removal refused because the file was newly staged; the reviewed incoming file was then removed explicitly. No other path was removed.

The [mechanical source dispositions](recovery-source-dispositions-20261121.json) enumerate 32 distinct checkpoint blob versions, 24 named non-checkpoint bundle refs, 49 saved closed-unmerged root PR heads, three saved child PR heads, and 61 remaining original objects. Reachability here means original Git ancestry, not a claim that every historical behavior is qualified. PR489 separately carries eight original integration snapshots and the receipt-provenance repair.

Seven original commits remain unmerged because their history contains blob 42640d0a193e3390fa590debd9abed098f3eec7e, a 176054272-byte generated `rrflow-runner.exe`. Deleting it in a descendant does not remove it from history. GitHub's [large-file limit](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github) blocks that history; this package does not rewrite it or attempt to upload it. Exact commits and per-source dispositions are in the JSON. Reusable query decoding, workbench script, CLD diagnostics and canonical engram source are present at current owners; generated executables, raw prompt inputs and runtime captures are not republished. This is not an original-head merge claim.

The complete source/manifest patches of af19567b7150e318c0d9f846f6a4138ae3456901 were reviewed. They update engram dependency/module/configuration-asset paths and also corrupt `RrflowCldLifecycleState` into `RrflowCldLifecyclengram`. Current canonical paths and the corrected lifecycle type remain. The 9b60e959 manifest patches were also read: current query `raw_value` and workspace `float_roundtrip` support remain. Current query read-path behavior and tests are recorded in PR484's journal. No new missing implementation or assertion was identified in these patches.

Two remaining objects, eb27cfa18a6e52b8ee0d24ab9c0829f47a9eb2f2 and its index parent dbb334c8041fe4c6cfcb66a1b96ad40593f23e32, hold only five generated `.rrflow` files, including a 3461888-byte WAL. They are preserved in the original bundle, not applied over the current local database or published. The other 52 objects are generated merge/test/simulation snapshots. Their source parents are accounted for; combined resolution-path inspection found generated navigation/planning changes and CI inspector wiring already present. This does not claim a full reread of every historical prose body.

Parentless agent checkpoint roots and the Codex tree are not imported wholesale: they include generated runtime state and retired parallel planning authorities. Seven vector checkpoint patches were reviewed in full against their nearest recovered source. Current revision-neutral restoration, generic TurboQuant rejection, artifact codec checks and lifecycle activation remain. Two diagnostic checkpoint changes are represented in current CLD counters. The script and documentation checkpoint review coverage is explicit per entry; PLAN, STATUS and the old global journal were inspected for ownership, not fully reread. The final workbench script is byte-identical to e305e80's version; no comparison or capacity workload was executed.

Original recovery bundles remain unchanged outside the source tree: root SHA-256 7e23b1cc9d9c902c0809e763e3e96a2c6db92395eb5c279c101fd97494cbbaf9; child SHA-256 fb9b239df56cd9752ed6b1898151d934b996f15367e2120812412b888e3e3b6a. They are preserved evidence, not a new work stash. Existing product gaps, including the ignored interrupted-refresh oracle, remain open in their owners. No objective, release or parity status advances from this record.

## Validation

- `cargo test -p rrd-engine --test workspace_architecture --locked`: 35 passed, zero failed, zero ignored, against cbcaa881 source plus this package's documentation. No runtime delta required another CLD or quantization execution; earlier focused source-package results are not represented as fresh tests here.
- Python JSON/object validation uses `git cat-file -t` for every checkpoint blob and `git merge-base --is-ancestor` for each saved head/ref against this candidate and PR489's exact d1b66d407fd17dc150f58dcbee6f8a24f5b1bcc4 candidate; child heads are checked against child main. Final main-only reachability and cleanup are required after the candidates merge.
- Final documentation, generated navigation, inventory and whitespace results are recorded below before readiness. No ARM, live provider, hosted sync, UI, benchmark or full capability qualification was run by this package.

- JSON/object validation passed: 49 root PR heads, three child heads, 24 named bundle refs, 32 readable checkpoint blobs; all 61 exception objects remain outside the candidate ancestry as declared.
- `python scripts/knowledge/render_navigation.py`: passed, 21 indexes, 1335 nodes, 4668 edges, no further index edits.
- `python scripts/ci/check_documentation.py`: passed, 1338 statuses and 1334 classified coordinates.
- `python scripts/ci/build_execution_inventory.py --check` initially reported derived inventory drift after switching packages. Regenerated with the owning script, then reran the check; no hand-maintained inventory or product status change.
- `git diff --check`: passed. This package's final source delta remains zero runtime/test files. Candidate and merge-group CI are still required; post-merge verification cannot be claimed before landing.

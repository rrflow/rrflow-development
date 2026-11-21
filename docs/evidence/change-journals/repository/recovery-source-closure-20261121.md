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

# C-07 native launch and failure evidence consolidation

**Status:** active integrated evidence; candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-native-evidence-consolidation`
**Owner:** OBJ-02 / C-07g,h / POAM-035 / [PR169](https://github.com/rrflow/rrflow-development/pull/169)

Driver: this session. Before integration PR169 recorded the exact inputs and stop conditions: prior evidence head `6bf653593001038dc7d6689a584ac0cec6f70fe1`, public main `5ca589a62cbbe2a131af31ebb79b06a76129d634`, PR151 `d17db422052c88a3d1f28b7a9ce2116c3c32db2a` and PR167 `385399af2b81697bd67cda2220c582f76199e019`. Main was ordinarily merged at `c66ddae35aaacfba5f32ca668d3feaf9538b3326`; PR151 was ordinarily merged at `9f96d6f493d3b44f1aad24bc6ae43d27009d7604`; PR167 is the final integration parent. The sole semantic conflict was two additive POAM-035 evidence passages; both were retained. Generated navigation and inventory conflicts were regenerated. No engine behavior, evidence identity, resource limits or release status changed.

Trace map: PR151's Windows launch, progress and failed terminal records, and their original POAM links, remain individually linked in the Gate C journal index. PR167's integrated 5d554483 native launch journal remains separately linked. PR169's c02f33c8 launch journal and its earlier refresh remain separately linked; its eventual result requires another immutable record. Existing canonical owners remain POAM-035 and the Gate C journal index. No prior failure is rewritten or claimed as success. The source evidence PRs remain open pending actual integrated merge and replacement-evidence review.

Read in full: repository change-authoring procedure, user-provided AGENTS.md and the PR169 consolidation brief. Inspected both POAM conflict sides, source PR descriptions, incoming journal paths, generated navigation and the complete merge diff. No upstream code adaptation or new runtime trace; the parent journals retain the source/campaign/native receipts. This integration and its local Git/Python checks ran on the same host as the active native campaign, so its timing is not isolated.

`python scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py`, `python scripts/knowledge/render_navigation.py --check`, staged `git diff --check`, and `python scripts/ci/build_execution_inventory.py --check` passed.

The running native outcome, exact candidate CI and merge queue, full native matrix and 24-hour soak remain pending. No C-07, objective or POAM completion claim is made.

Checklist: explicit scope and evidence trace bound before integration; both POAM additions preserved; generated conflicts reconstructed; source identities and original failures retained; tests and errors recorded; full diff reviewed; exact CI and merge pending.

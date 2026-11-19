# Connectome pin: two workbench rules that a restructure had lost

**Status:** active submodule pin update; no H-06 qualification claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-repin-workbench-lost-rules-20261119`

**Owner:** H-06 Connectome Workbench; checkout boundary in `docs/reference/client/connectome.md#repository-checkout`

**Starting revision:** main `d48e3a225`, which pins `connectome/` at `20431e2e`.

**Change.** The gitlink moves to Connectome main `c3451dccce472509b0b402ab05b7d5db592b7d94`, the merge of rrflow/connectome-development#103, which follows #102. Both restore rules that the workbench's earlier checks asserted and that its restructure into lanes had lost. Each was found by reading what the old check demanded, after a code comparison had recorded the behaviour as present.

- **#102, cloud source addresses.** A source must be an HTTPS address without credentials, and a Google Docs relay is accepted only on `docs.google.com`. Before, any text was saved as a source and later passed to `window.open` and `fetch`; `javascript:alert(1)` was accepted. The rule is applied when a source is added and again when a saved one is used.
- **#103, template import.** An import is decided after its file has been read, on what the workbench holds then, and writes nothing if its workbench has gone. Before, it built on the template list from when the import began, so a template added during the read was dropped and a removed one came back.

**Evidence, from the Connectome repository.** Both pull requests passed every job on the candidate and in the merge queue: frontend build and smoke, the native crate on Linux, macOS and Windows, and the real-engine integration job against the engine at `cc5ebe0fd`.

**Scope.** The `connectome` gitlink and this record. The bound engine revision is unchanged, so `docs/reference/client/connectome.md` is unchanged.

**Not proven.** The rest of the old workbench checks' behaviours (guided routes, scripted events, template authoring and export) are present on Connectome's `main` by their labels; they were not each driven in a browser.

**Acceptance.** The candidate run on this branch and `git submodule status` resolving the new revision.

**Roadmap/POA&M change:** none.

# LFG and merged worker reconciliation

**Status:** active draft integration evidence; qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/lfg-main-reconciliation-20261106`
**Owner:** OBJ-06, G-01b/G-02a and POAM-009; PR 249

Starting candidate `187051cdcc1c6f66548444934060726f7eb7f883` contains
[the prior joined implementation and installed fixture evidence](lfg-selected-worker-installed-controlled-20261104.md).
Its second parent lineage contains PR 249 head `f17d62b94d6178033186fbc8efb0f7c8dbe69a7a`.
Merged public main `f4d74af4861e8f2f008e3c3bd8fcb4e1503c0294`, which
includes PR 229; PR 246 was already merged. The tested merge index tree was
`a2651b1e35887535966da0efbbe3368b06c5bb1a`, before adding this journal.
The brief and scope are recorded on PR 249. The target is one engine-owned
LFG advisory and selected-worker path; no new authority, implicit activation,
external provider call, or altered user project state is intended.

The merge conflicted in the contract OpenAPI digest, the inference-reference
status, six other reference digest citations and twelve generated SDK/signal
projections. Each conflict block was inspected: except the status line, the
opposing contents differed only in their SHA-256 digest. The combined owner
and all generated surfaces were then checked against the exporter. The actual
combined digest is `7ff11cd88d5fa1e5c1f4b4810b7daeb5f90037de3c801522ce625725a3e79d40`;
no generated projection needed rewriting. The reference status now says the
worker is merged, LFG remains draft, and the complete journey is incomplete.
This integration adds no new engine behavior beyond the earlier candidate.

## Evidence and checklist

- Complete files read for this increment: `docs/roadmap/rrflow-1.0-execution/change-authoring.md`, the prior installed-composition journal, and the LFG/SWAG bridge and LFG server startup/learning portions used for the live test. Conflict hunks, the PR brief, merge metadata and generated-check implementation were inspected. The earlier journals record the broader implementation review; this increment does not claim a new line-by-line review of the entire candidate.
- First oracle: `git merge-tree --name-only` showed 20 conflicting paths from the tested joined candidate against the merged worker main; all were reconciled as described. `python -X utf8 scripts/ci/check_generated_surfaces.py` passed: 79 operations and seven signal projections match the owner.
- `cargo test --offline --locked -p rrd-engine --lib reasoning_budget::lfg_flow:: -- --nocapture`: 7 passed, 2 ignored. `cargo test --offline --locked -p rrd-engine --lib -- --test-threads=4`: 362 passed, 6 ignored. `cargo test --offline --locked -p rrd-contract`: passed all active tests.
- A source-built local LFG server at `70252e70277080bc4f55898db1ec25f978b5c787`, binary SHA-256 `fd9e8234c2e4efb00a6479fd964d5ce5f9e5f0e3660e35eb1cea9390cd3b1d45`, was started on an isolated loopback port with `LFG_LEARNING=off` and isolated state. With `RRFLOW_TEST_LFG_URL` set, `cargo test --offline --locked -p rrd-engine --lib real_lfg_ -- --ignored --nocapture` passed both tests. They exercise live LFG and the in-process engine with controlled selector and worker fixtures and reopen readback. The test log SHA-256 is `3b75d3c7181dca8e6734ddeba4dfc645719f230b21f8920dfeaaa186f5ad6c45`; the owned LFG process was stopped. Raw local logs are not published.
- A separate local SWAG bridge exchange used the same LFG binary and a project-derived prompt; the bridge accepted the bound advisory. It did not involve RRFlow or a model. One elapsed time is not a benchmark. No upstream source was adapted in this increment; existing bounded diagnostic traces are preserved.
- `python -X utf8 scripts/knowledge/render_navigation.py --check` and `python -X utf8 scripts/ci/check_documentation.py` passed before this journal. `git diff --cached --check` against the first parent reported CRLF lines in worker evidence already on merged main; checking the staged tree against main passed. The existing evidence bytes were not rewritten.
- Changed paths relative to main are the LFG code, contracts, tests, existing evidence and regenerated projections enumerated by `git diff --cached --name-only f4d74af4861e8f2f008e3c3bd8fcb4e1503c0294`, plus this journal and its index. Nothing was moved or deleted by this increment. The index must be regenerated before commit.
- Not run here: real standalone SWAG as RRFlow controller, a model, project-source edits during the live call, installed/native qualification on this exact merged tree, platform matrix, soak, full candidate or merge-group CI. The standalone SWAG `/v1/decision` and RRFlow chat-completions contracts remain different. No end-to-end quality, alpha, C-07 or competitive claim follows.
- Checklist: baseline and scope identified; conflicts inspected; existing authorizations and source identities preserved by the joined implementation; focused and owning tests plus contract generation run; resource and research decisions documented; candidate remains draft for full diff review, remaining gaps, and CI. No roadmap or POA&M status is advanced. PR and CI status are read from GitHub, not inferred from this record.

The first journal navigation attempt failed because the Windows writer added a UTF-8 BOM before the title; documentation also reported the missing generated index entry. The BOM was removed and both checks were rerun.

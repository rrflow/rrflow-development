# RRFlow roadmaps

**Status:** active roadmap index
**Coordinate:** `rrflow://rrflow-instance/data/roadmap-index/rrflow-roadmaps`
**Owner:** roadmap discovery; linked from `docs/README.md`

A roadmap records ordered delivery outcomes and acceptance evidence. It links
to architecture and decisions but does not redefine them.

The current [alpha objective](../objectives/rrflow-1.0-alpha.md) defines the
result. The current [POA&M](../poam/rrflow-1.0-alpha.md) tracks verified gaps
against the roadmap gates.

The canonical roadmap is accompanied by a subordinate
[code execution map](rrflow-1.0-execution-map.md) and its generated
[whole-repository file plan](rrflow-1.0-file-plan.jsonl). These records map
files, symbols, tests, and stop conditions; they cannot change checklist status.
The completed C-06i compression slice is bound by the
[adaptive page-compression plan](c06i-rrflowkv-adaptive-page-compression-engineering-plan.md).
It specifies a direct segment-v6 replacement with authenticated per-page
adaptive LZ4, bounded decode, explicit ownership, and integrated evidence. The
completed
[persisted-filter integration plan](c06i-rrflowkv-persisted-filter-integration-engineering-plan.md)
records the direct segment-v4-to-v5 replacement and first selected production
policy. The completed
[physical-policy selection plan](c06i-rrflowkv-physical-policy-selection-engineering-plan.md)
remains the measured input; cache policy and value placement still require
separate decisions. The completed C-06h
[adversarial qualification plan](c06h-rrflowkv-adversarial-qualification-engineering-plan.md)
and C-06g
[projected-read plan](c06g-rrflowkv-projected-read-engineering-plan.md) remain
supporting history. None can change canonical dependency order or completion
status.

| Release | Durable warp | Checkout record | State |
|---|---|---|---|
| RRFlow 1.0 | [`rrflow://rrflow-instance/data/roadmap/rrflow-1.0`](rrflow://rrflow-instance/data/roadmap/rrflow-1.0) | [`rrflow-1.0.md`](rrflow-1.0.md) | pre-alpha; Gates A and B complete, Gate C 5/7; C-06 active and partial |

| Supporting record | Durable warp | Checkout record | Authority |
|---|---|---|---|
| RRFlow 1.0 code execution map | [`rrflow://rrflow-instance/data/execution-map/rrflow-1.0`](rrflow://rrflow-instance/data/execution-map/rrflow-1.0) | [`rrflow-1.0-execution-map.md`](rrflow-1.0-execution-map.md) | file/symbol work packages only; roadmap checkboxes remain canonical |
| C-06i rrflowKV adaptive page-compression plan | [`rrflow://rrflow-instance/data/work-package/c-06i-rrflowkv-adaptive-page-compression`](rrflow://rrflow-instance/data/work-package/c-06i-rrflowkv-adaptive-page-compression) | [`c06i-rrflowkv-adaptive-page-compression-engineering-plan.md`](c06i-rrflowkv-adaptive-page-compression-engineering-plan.md) | completed segment-v6 adaptive-LZ4 production slice and fixed-machine integration evidence; C-06 remains open |
| C-06i rrflowKV persisted-filter integration plan | [`rrflow://rrflow-instance/data/work-package/c-06i-rrflowkv-persisted-filter-integration`](rrflow://rrflow-instance/data/work-package/c-06i-rrflowkv-persisted-filter-integration) | [`c06i-rrflowkv-persisted-filter-integration-engineering-plan.md`](c06i-rrflowkv-persisted-filter-integration-engineering-plan.md) | completed segment-v5 persisted-filter production slice and evidence |
| C-06i rrflowKV physical-policy selection plan | [`rrflow://rrflow-instance/data/work-package/c-06i-rrflowkv-physical-policy-selection`](rrflow://rrflow-instance/data/work-package/c-06i-rrflowkv-physical-policy-selection) | [`c06i-rrflowkv-physical-policy-selection-engineering-plan.md`](c06i-rrflowkv-physical-policy-selection-engineering-plan.md) | completed candidate-screen evidence and retained-policy handoff |
| C-06h rrflowKV adversarial qualification plan | [`rrflow://rrflow-instance/data/work-package/c-06h-rrflowkv-adversarial-qualification`](rrflow://rrflow-instance/data/work-package/c-06h-rrflowkv-adversarial-qualification) | [`c06h-rrflowkv-adversarial-qualification-engineering-plan.md`](c06h-rrflowkv-adversarial-qualification-engineering-plan.md) | completed supporting package history |
| C-06g rrflowKV projected-read engineering plan | [`rrflow://rrflow-instance/data/work-package/c-06g-rrflowkv-projected-read`](rrflow://rrflow-instance/data/work-package/c-06g-rrflowkv-projected-read) | [`c06g-rrflowkv-projected-read-engineering-plan.md`](c06g-rrflowkv-projected-read-engineering-plan.md) | completed supporting package history |

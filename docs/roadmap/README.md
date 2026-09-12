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
The active C-06h package has a
[human-readable engineering plan](c06h-rrflowkv-adversarial-qualification-engineering-plan.md)
that explains the machine-enforced qualification plan and observed bounded
evidence. The completed C-06g
[projected-read plan](c06g-rrflowkv-projected-read-engineering-plan.md) remains
supporting history. Neither can change canonical dependency order or
completion status.

| Release | Durable warp | Checkout record | State |
|---|---|---|---|
| RRFlow 1.0 | [`rrflow://rrflow-instance/data/roadmap/rrflow-1.0`](rrflow://rrflow-instance/data/roadmap/rrflow-1.0) | [`rrflow-1.0.md`](rrflow-1.0.md) | pre-alpha; Gates A and B complete, Gate C 5/7; C-06 active and partial |

| Supporting record | Durable warp | Checkout record | Authority |
|---|---|---|---|
| RRFlow 1.0 code execution map | [`rrflow://rrflow-instance/data/execution-map/rrflow-1.0`](rrflow://rrflow-instance/data/execution-map/rrflow-1.0) | [`rrflow-1.0-execution-map.md`](rrflow-1.0-execution-map.md) | file/symbol work packages only; roadmap checkboxes remain canonical |
| C-06h rrflowKV adversarial qualification plan | [`rrflow://rrflow-instance/data/work-package/c-06h-rrflowkv-adversarial-qualification`](rrflow://rrflow-instance/data/work-package/c-06h-rrflowkv-adversarial-qualification) | [`c06h-rrflowkv-adversarial-qualification-engineering-plan.md`](c06h-rrflowkv-adversarial-qualification-engineering-plan.md) | active readable package plan; machine enforcement remains in `rrflow-1.0-active-change.json` |
| C-06g rrflowKV projected-read engineering plan | [`rrflow://rrflow-instance/data/work-package/c-06g-rrflowkv-projected-read`](rrflow://rrflow-instance/data/work-package/c-06g-rrflowkv-projected-read) | [`c06g-rrflowkv-projected-read-engineering-plan.md`](c06g-rrflowkv-projected-read-engineering-plan.md) | completed supporting package history |

# H-06 guided learning and controller UI requirements

**Status:** active context correction; UI execution remains unqualified
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-06-guided-learning-controls`
**Owner:** OBJ-11, H-06 and G-owned LFG/SWAG execution

Baseline `7382f29f328d402bd01041d7ba9f7ccbc845d141`, tree `1d68d0f96f2bb6638564e816f2ed855ed7261b29`.
[PR137](https://github.com/rrflow/rrflow-development/pull/137)'s brief was
extended before edits. The owner requires Connectome as the UI package for the
whole experience: approachable setup, contextual learning, tooltips, suggestions
and advanced drill-down, with configurable LFG and SWAG System One control.
Previously the basic/advanced view table did not explicitly bind this teaching
experience or controller configuration/readback to H-06 acceptance.

Changed README UI identity, the existing Connectome turnkey subsection and
Gate H acceptance; added this journal and generated navigation/inventory only.
No runtime, API, submodule, version or gate status change. Prior journals remain
immutable. Targeted reads: README UI passage, Connectome authority/turnkey/table/
acceptance sections, LFG/SWAG identity and configuration boundaries, Gate H
turnkey acceptance, existing PR brief. These are partial owner reads, not a full
code audit. Broad output truncation was resolved with bounded reads.

Research decision: user-owned experience requirements and existing LFG/SWAG
semantics suffice; no new algorithm or third-party behavior is asserted.
Trace/resource/debug instrumentation is inapplicable to documentation edits.
Oracle: the existing owners require plain-language basic controls, contextual
explanations grounded in actual evidence, same-object technical drill-down,
global/project inheritance and actual engine configuration/readback. They
explicitly distinguish availability from execution and link to G qualification.
Full changed diff reviewed. No UI or controller execution is claimed.

Checklist: brief/owner/baseline before edits; existing boundaries inspected;
explicit before/after oracle and research decision; no new authority; one linked
record and generated artifacts; results below. No runtime tests, UI launch,
provider calls or soak launched. Remaining implementation and real-user-flow
proof stay H-06/G work. PR remains draft/unmerged, with no new CI pass claim.

Validation passed: `python -X utf8 scripts/ci/check_documentation.py`
(614 statuses, 612 coordinates); `python -X utf8
scripts/knowledge/render_navigation.py --check` (17 indexes, 613 nodes, 2443
edges, zero drift); `git diff --check`. Execution inventory regenerated and
checked for 1736 records before commit. These validate documentation only.

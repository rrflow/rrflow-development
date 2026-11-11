# POA&M: defects found by the live engine probe and a CI rig failure

**Status:** active implementation evidence

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/poam-probe-defects-20261110`

**Owner:** [RRFlow 1.0 alpha POA&M](../../../poam/rrflow-1.0-alpha.md)

**Starting revision:** `main` at `47f018db`.

**Change.** Adds POAM-071 to POAM-074. No code changes.

**Evidence.** POAM-071 to POAM-073 come from one Windows run of a release binary through the public API on 2026-11-11, and POAM-071 also from an engine test held on an unpushed branch. POAM-074 comes from candidate run 42954943269, job 130307847339.

**Acceptance.** `python scripts/knowledge/render_navigation.py` and `python scripts/ci/check_documentation.py` passed.

**Not established.** The probe ran once on one platform. POAM-074 is one occurrence and its cause is not proven.

**Research decision:** none. **Roadmap/POA&M change:** four open items added; no status changed.

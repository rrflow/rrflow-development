# C-07 stale recovery parent reconciliation

**Status:** active candidate; exact-head candidate and protected CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-stale-recovery-parent-reconciliation`
**Owner:** OBJ-02 / C-07 / POAM-035 / [PR214](https://github.com/rrflow/rrflow-development/pull/214)

[PR210](https://github.com/rrflow/rrflow-development/pull/210) merged to development main at `6fe8b7590d913beda9b009551e5d2d2644e406d9` after [protected merge-group CI](https://github.com/rrflow/rrflow-development/actions/runs/40987247016) passed. Its source tree `3bd842a6132a2815fab0133988898ec2483b6293` equals its candidate tree, and its candidate revision is already an ancestor of this draft. This landing does not qualify the previously failed 4k point, 100k native matrix or 24-hour soak.

The local [stale transaction correction](c-07-stale-transaction-recovery.md) was committed at `7114d421d1e24826a6a553dbe6c8665230ce6a8c`, tree `af7961dcf64d2953e9f55f2069973d1d7cae7c3c`. The PR brief preceded reconciliation. A read-only `git merge-tree --write-tree --name-only` predicted that exact tree when merged with landed main. The ordinary no-commit merge had no conflicts; `git write-tree` matched the prediction and `git diff --cached --name-status` was empty before this journal and its index. No implementation moved, was deleted, or changed in this reconciliation; the earlier source correction and all landed parent tests remain the canonical paths. Git ancestry does not replace behavioral or candidate CI evidence.

Checklist: reviewed actual main, merge parents, candidate tree, prior tests and open POAM-035; no upstream source was adapted. Rerun generated-surface, workflow, navigation, documentation, inventory and diff checks before committing. The prior local installed, engine and server test results remain tied to the identical source in `7114d421`; exact new-head candidate CI, protected merge-group CI, native 100k capacity and soak evidence remain required. No roadmap, POA&M or alpha status changed. No direct main push, force push or promotion occurred.

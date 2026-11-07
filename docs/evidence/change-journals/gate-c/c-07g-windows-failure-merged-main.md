# C-07 Windows failure evidence: integrate merged main

**Status:** active integration evidence; refreshed candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-windows-failure-merged-main`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR151](https://github.com/rrflow/rrflow-development/pull/151)

[PR151](https://github.com/rrflow/rrflow-development/pull/151) bound this evidence-only refresh before the merge. Starting candidate `f936ffe2269dfd751feaee4551f91763793a2be9`, tree `c1eeaf0dcd27631e526a9679b676c41b578f611a`, ordinarily merges public development main `68c06059fbba600af5f06a6de30667c07e35ae6b`. Driver is this session. Git reported conflicts only in the generated Gate C index and execution inventory; both are regenerated. The merged POAM-035 retains main's shared-manifest and APFS findings and the independently retained `90bb821e` terminal failure. No original journal or retained artifact is rewritten, no code is moved or deleted, and no runtime effect or resource limit is changed.

Read the README, C-07 and POAM-035 owners, original launch, progress and [terminal record](c-07g-shared-pins-windows-terminal.md), PR brief, merged POAM and generated conflicts. No additional external research, code adaptation, trace or native measurement was necessary for this documentation merge. The terminal evidence remains: 100,000 acknowledgements and initial independent verifications, two reopens, 47,016 post-reopen verifications, pinned-byte denial, zero measured segments and confirmed cleanup. The denied numerical amount and cause remain unknown; concurrent compilation limits timing interpretation. The cold profile and required 24-hour soak did not run.

Navigation/inventory are regenerated and checked; `python scripts/ci/check_documentation.py`, `python scripts/ci/check_workflow.py`, staged diff and post-commit inventory checks are required before push. The refreshed PR needs its own candidate CI and merge; no test, completion, native capacity, performance or soak claim is added. C-07 and POAM-035 stay open.

Checklist: owner, baseline, scope, retained identities and stop conditions are bound in the PR; generated conflicts, original evidence and merged POAM are reviewed; candidate CI and merge remain required.

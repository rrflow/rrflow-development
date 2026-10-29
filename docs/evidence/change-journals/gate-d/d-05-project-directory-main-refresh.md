# D-05: project-directory path main refresh

**Status:** active integration evidence; candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-05-project-directory-main-refresh`
**Owner:** OBJ-08 / D-05 / POAM-037 / [PR154](https://github.com/rrflow/rrflow-development/pull/154)

The [project-directory path package](d-05-project-directory-path.md) was implemented and checked at `8e38bb67`. After PR149 landed as development main `2e3a67d26295a763155a5129960d070914626704`, this candidate merged that exact main normally at `61ff743a`. The merge adds the accepted installed project-root capability. Generated Gate D index and file inventory conflicted; the package side was selected only for those replaceable generated files, then both were regenerated from the combined source. The first inventory generation correctly failed on an unmerged index; staging the regenerated index and rerunning resolved it. No implementation conflict or source deletion occurred.

After the merge, `cargo test --offline --locked -p rrd-client --test knowledge_trees` passed the real installed authenticated HTTP configuration/replay/reopen test. Generated navigation/inventory checks, documentation and workflow policy, and `git diff --cached --check` passed. Source scanning, automatic relationship creation, full candidate CI and landing remain unproven; no roadmap or POA&M status changes.

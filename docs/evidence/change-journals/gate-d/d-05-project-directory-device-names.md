# D-05: Windows device-name correction for project directories

**Status:** active implementation evidence; candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-05-project-directory-device-names`
**Owner:** OBJ-08 / D-05 / POAM-037 / [PR154](https://github.com/rrflow/rrflow-development/pull/154)

The [project-directory candidate](d-05-project-directory-path.md) at `6ab896dcc75fbf8d20daae17f102bc393c9388a9` rejected ordinary ASCII Windows device names but accepted `COM`/`LPT` names with superscript digits. The PR brief was updated before this correction. [Microsoft's Win32 naming rules](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file) explicitly reserve superscript 1, 2 and 3 as well as the ordinary numbered devices and extensions. This is a bounded syntax correction, not proof of native resolution or ingestion.

The focused `project_directory_paths_are_portable_and_project_scoped` test first failed with `unsafe project directory accepted: "docs/COM¹"`; the same test passed after the existing validator rejected all three superscript forms, case-insensitively, including extensions. The path remains project-relative configuration only. General filesystem naming, symlink-safe opening, outside-project roots, source observation and automatic relationships remain separate work. The original candidate CI cannot qualify this changed source; new candidate CI is required.

The changed `knowledge_tree.rs` validator and test, PR brief, earlier package and linked Microsoft source were reviewed. No engine or public schema shape changed. `cargo test --offline --locked -p rrd-contract project_directory_paths_are_portable_and_project_scoped --lib` passed after its initial failure; the full contract suite passed 132 tests, `cargo test --offline --locked -p rrd-client --test knowledge_trees` passed one installed HTTP/restart test, and strict all-target contract Clippy passed. Generated-surface parity, navigation/inventory, documentation, workflow, formatting and diff checks passed on Windows. No gate or objective status changed.

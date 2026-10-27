# C-07g: refresh manifest sharing after fixture repair landed

**Status:** active integration evidence; fresh candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-manifest-post-fixture-main`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR142](https://github.com/rrflow/rrflow-development/pull/142)

PR142 started this integration at `8721f17adc6fd97a05708feb1b3e332508365058`; its PR brief was updated before an ordinary merge of development main `393e6b86ee58bb4ea4b77d14a871af339404ebb4`, which contains the independently merged PR150 fixture repair and PR149 retained source root. Merge-group run 40524747207 passed for PR150; the old PR142 candidate's green checks remain associated with its prior revision. No source implementation was removed, no thresholds changed and no qualification was claimed. The only generated inventory merge was resolved mechanically and both owners' records were retained.

On Windows with `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target/c07-independent-review/target`, `cargo test --offline --locked -p rrd-lsm --lib segment::reader::lifetime_tests` passed five tests; `cargo test --offline --locked -p rrflow-cli --features crash-test --test installed_crash capacity::filler_tests` passed 13. An initial fixture command omitted `--features crash-test` and failed before execution; a subsequent `capacity::tests` filter selected zero tests and is not a pass. The corrected actual filler test filter passed. Navigation and inventory checks, documentation and workflow policy checks, and `git diff --check` passed before this journal and were rerun after regeneration.

Reviewed current PR142 and PR150 briefs, prior integration journal, affected fixture and manifest lifetime tests, main merge and generated diff. This is a baseline integration, without new algorithm, upstream adaptation, runtime trace or change to existing diagnostic ownership. Fresh candidate CI, merged downstream readback and accounting corrections, native capacity and the actual 24-hour soak remain open. No roadmap or POA&M completion status changes.

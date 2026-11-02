# C-07 native pipe capture: hosted macOS diagnostic outcome

**Status:** active diagnostic outcome; protected failure unresolved
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-pipe-macos-outcome`
**Owner:** C-07 / POAM-035 / [PR215](https://github.com/rrflow/rrflow-development/pull/215)

The [launch record](c-07-native-pipe-macos-launch.md) froze temporary workflow
head `f5aa0912793c37d3751c8a299fa31b26a1c540bf` for one targeted hosted macOS
diagnostic and documented restoration of the benchmark workflow in ordinary
commit `1cc5feb215ff19de52dd7c61a771f63c5830c67e`. The [diagnostic run](https://github.com/rrflow/rrflow-development/actions/runs/40999521873)
completed successfully; its native APFS repetition step ran 03:54:07–03:59:18
UTC, 5 minutes 11 seconds. The GitHub artifacts API reported an empty artifact
list, consistent with its upload step finding no bounded failure receipt.
The original six-mode, 24-case APFS repetition completed on this different
draft source; this is a non-reproduction, not a diagnosis or correction of the
protected failure. No conclusion is drawn about a particular command or child.

The [PR208 protected merge group](https://github.com/rrflow/rrflow-development/actions/runs/40996295111)
subsequently completed with failure. Its macOS domain-crash step had reported
`command output did not close`; its Windows estate-process step separately
timed out after five minutes. The targeted macOS diagnostic did not include the
PR208 candidate and did not test the Windows sweep. Neither is cleared by this
outcome. Required candidate and protected merge-group checks and C-07 capacity
and soak evidence remain outstanding. No gate or product status changes.

Checklist: before this record, updated the PR brief with source, run, result,
scope and stop conditions; read the launch record, GitHub job status, selected
native log and artifacts API; created this immutable outcome and regenerated
the nearest navigation and execution inventory. No product code, native test,
workflow, threshold or external process was changed. No upstream source was
adapted; the hosted logs are diagnostic evidence rather than resource or alpha
qualification. The PR remains draft because the responsible command and safe
cleanup for the original failure are not established.

`gh run view` and the artifacts API provided the job result and artifact
inventory; counting the native `capacity_cleanup: detached` messages returned
24. `python scripts/knowledge/render_navigation.py` updated the index;
`python scripts/ci/check_documentation.py` passed; `git diff --check` passed.
This evidence-only follow-up has no new runtime test; PR215's original
implementation tests are recorded in its linked implementation journal.

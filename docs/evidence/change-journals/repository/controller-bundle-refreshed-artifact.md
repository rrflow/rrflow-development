# Refreshed controller bundle: actual artifact and native restart

**Status:** active immutable engineering evidence; public release qualification open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/controller-bundle-refreshed-artifact`
**Owner:** installed-lifecycle engineering delivery; J-03/J-05 prerequisites

## Scope and identity

On 2026-10-30, [PR212](https://github.com/rrflow/rrflow-development/pull/212)
advanced its previously published combined-engine build and relocation brief.
Starting/build/assembly revision: `8cf235ac06a92bc16ae74e746ff1928fb8415b47`,
tree `b32e747d881251293246ff11a4183509d1f202f3`, clean source. This follows the
[integration record](controller-bundle-combined-engine.md); original artifacts
and model observations retain their original identities.

The new local `rrflow-controller-trial-windows-x64-8cf235ac.zip` is 33,976,833
bytes, SHA256 `8fc7a5056bde73f29ffb435393d6254cac152ff0f30aea4158e597fb9749f9a3`.
Its primary `rrflow.exe` is 114,013,696 bytes, SHA256
`2ef231864e983673ac82eeadc158be993770917cd1f0ae81b52bb4a951c457a1`.
It contains no weights, credentials, project estate, task or GUI.

## Commands and actual observations

Exact commands, source identities, exit codes, timings, packaged build receipt,
member manifest and verification outputs are retained in the
[evidence manifest](../../controller-bundle-refresh-20261030/evidence-manifest.json).
The two readiness reports and archive-test log are gzip-compressed exact bytes; the manifest
binds both compressed and decompressed hashes. Private generated operator
credentials and estate data are excluded. All exported report bytes were
checked against the actual local operator credential before publication.

- `python scripts/build_rrflow.py --profile release --target-dir <release-cache>
  --output <receipt>`: exit 0, 1,162.08 seconds. Canonical two-pass provenance
  build, `CARGO_BUILD_JOBS=2`, no Cargo profile override or supplied build manifest.
- `python scripts/package_controller_trial.py --build-receipt <receipt>
  --target-dir <release-cache> --output <new-zip>`: exit 0, 43.08 seconds.
- `python scripts/test_controller_trial_package.py --archive <new-zip>
  --sha256 <digest-above> -v`: all 18 checks passed, including actual-artifact
  corruption/rejection cases; exit 0, 16.05 seconds.
- The retained operator procedure `relocated-primary-check.py` verified the
  archive before extracting frozen bytes to a new directory. Actual primary
  version matched the build receipt. Installation identity, plan and apply
  succeeded. First serve and reopened serve both returned authenticated readiness;
  quick verification after each stop passed all 13 checks. Instance, installation
  plan, project inventory and runtime manifest identities matched across reopen.
  Procedure exit 0, 7.59 seconds.

Every primary child used a whitelisted Windows environment with only System32
on PATH and no provider credentials. Both owned server processes were terminated
and awaited; this is not graceful-shutdown evidence. This same-host experiment
does not qualify a clean machine. No model request or worker effect occurred.

## Review and limits

Read the applicable AGENTS, README status, repository change-authoring procedure,
deployment reference, integration record, standalone runbook, assembler/verifier,
archive tests and build-provenance interfaces before the build; reviewed the full
operator procedure and exported receipts before publication. Changes add only
this record, exact evidence, a deployment-reference link and generated discovery.
No runtime or packaging implementation is replaced. Research/adaptation: none;
existing canonical build and installed-lifecycle owners supply the oracle.
Resource/trace decision: bounded explicit archive members, content hashes,
60-second command/readiness limits and owned process cleanup; local full build
logs remain retained under `controller-bundle-refresh` for this date.

Checklist: pre-effect brief, exact source/tree, unchanged authority, independent
archive verification, real relocated native checks, credential exclusion,
full evidence/diff review and linked discovery. Stop conditions were dirty or
mismatched source/bytes, overwriting an archive, unsafe members, undeclared
dependencies or runtime failures. None arose in this build/trial. An evidence
export command initially used a mistyped working directory and was rejected
before execution; rerunning at the verified worktree exported the same reports.
Staged whitespace checking then identified the exact Windows CRLF test log;
gzip preserves those original bytes without treating carriage returns as source
whitespace. The original local log and first failed check are retained.
Documentation, navigation, execution inventory, workflow and whitespace checks
are recorded with the evidence-publication command results.

No new BF16 observation, UI interaction, worker execution, changed-input workflow,
model comparison, reviewed learning, 29 GB qualification, container, signing or
clean-machine campaign was run. The prior artifact's unsupported-effort rejection
remains in its original journal. The protected native macOS failure remains an
open prerequisite; PR212 stays draft behind PR222/206 and its own candidate and
protected checks. No merge, roadmap, POA&M or release-status completion is claimed.

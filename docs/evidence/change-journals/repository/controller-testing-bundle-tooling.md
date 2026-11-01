# Portable controller trial assembly tooling

**Status:** active implementation evidence; actual archive trial and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/controller-testing-bundle-tooling`
**Owner:** J-03/J-05 engineering preparation; installed-lifecycle owner
**Pull request:** [212](https://github.com/rrflow/rrflow-development/pull/212), dependent on [206](https://github.com/rrflow/rrflow-development/pull/206)

## Baseline and change checklist

Baseline `54fb845dffd702e1504c7f9a0cc2a3c5757d2bf2`. The PR brief was published
before implementation. The bounded prerequisite is delivery of the existing
engine and optional real-controller trial tooling without checkout path discovery.
No runtime behavior, CLI namespace, provider policy or training dataset changes.
Stop assembly on a mismatched executable/receipt, dirty source, incompatible
runtime/contract source, unexpected member or undeclared checkout dependency.

Read the complete AGENTS, assembler, verifier, tests and standalone runbook;
inspected README, installed-lifecycle, build provenance script, proof driver,
deployment navigation, change-authoring procedure, Gate J and CI verify job.
The prior installed-live journal owns the development binary observations.
The existing release build continues against the untouched baseline worktree;
these assembly changes have a separate worktree and source identity.

Created `scripts/package_controller_trial.py`, `scripts/verify_controller_trial.py`,
`scripts/test_controller_trial_package.py` and
`scripts/inference/controller_trial_README.txt`. Changed only receipt-relative
binary resolution in `scripts/inference/installed_controller_proof.py`.
Registered rejection tests in `.github/workflows/ci-reusable.yml`. Added the
deployment reference, this journal and their navigation. No moved/deleted source.

The assembler reuses the existing provenance owner and contract example, checks
actual binary identity, compiles its helper, packages an explicit bounded file
set, verifies before exclusive publication, and preserves distinct build and
assembly identities. The verifier checks externally expected archive digest,
exact members, Windows-safe paths, file types, sizes, member digests and portable
receipt binding. It never extracts or executes. The helper remains optional;
default install/readiness uses the primary executable alone.

## Research, resource and trace decisions

No external algorithm or dependency is adapted. Existing repository contracts
are sufficient. Bounds are 32 members, 2 GiB total payload, and 128 KiB manifest
and receipt. No runtime trace is added; real installed operation receipts remain
the proof owner. The optimized primary build uses one Cargo job to bound memory.
No model outputs, fake quality labels or credentials are packaging test inputs.

## Focused results and retained limitations

`python scripts/test_controller_trial_package.py -v`: 12 tests passed. These
negative archives use licence bytes solely to test corruption/rejection; they
are not claimed executable artifacts. Cases cover external hash mismatch,
missing/extra/altered members, changed digest, case collision, symlink, unsafe
paths, malformed/oversized records, duplicate JSON keys and false qualification.
`python scripts/ci/check_workflow.py`: passed, four workflows and nine substantive
jobs. `python -m py_compile scripts/package_controller_trial.py
scripts/inference/installed_controller_proof.py`: passed.

An initial patch expected the wrong CI step label and was rejected atomically;
the actual existing label was read and the patch corrected. Initial reads also
exposed absent guessed index/build-reference paths; existing tracked owners were
located instead. No tests failed in this implementation stage.

The release build, real archive round trip/corruption, relocated install and
actual BF16 trial have not completed at this record's creation. They require a
separate evidence node after execution; these script tests cannot substitute.
No repeat Rust suite is warranted by this Python/tooling-only change. Full
candidate CI, dependency landing, platform signing, SBOM, other native targets,
container delivery and full Gate J remain open. No roadmap/POAM completion change.

Implementation files and diff were reviewed for the stated boundary before
committing. Generated navigation, documentation and inventory checks are recorded
in the subsequent artifact-stage journal with their actual results. The PR stays
draft until the dependency and acceptance evidence are ready.

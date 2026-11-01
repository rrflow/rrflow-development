# Actual portable Windows controller trial

**Status:** active engineering artifact evidence; one model fault observed, release qualification open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/controller-testing-bundle-artifact`
**Owner:** J-03/J-05 preparation; G-01/G-02 proposal qualification remains separate
**Pull request:** [212](https://github.com/rrflow/rrflow-development/pull/212), dependent on [206](https://github.com/rrflow/rrflow-development/pull/206)

## Baseline and authoring checklist

This continues the preimplementation brief and
[tooling journal](controller-testing-bundle-tooling.md). Baseline source is
`54fb845dffd702e1504c7f9a0cc2a3c5757d2bf2`, tree
`e410aebf5123ed591eb230b0dff523c986d13518`. Assembly source is
`98d0de932fecce1a3103c31cb9bab75712a62d48`, tree
`1c674dc9258aaf52030aba6e789b8bd4c590d086`. Binary and assembly identities are
deliberately distinct; runtime/contract/configuration source compatibility was
checked, not asserted from ancestry alone.

The complete new assembler, verifier, tests and runbook were reread, along with
the receipt-relative proof change and actual retained results. Build provenance,
installed proof recovery/dispatch, CI gate, deployment owner and Gate J were
inspected at their relevant boundaries. Subsequent tooling commits add actual-ZIP
corruption tests, require matching helper compiler/build settings and Rust/Cargo
configuration, and use discovered loopback ports in the runbook. This stage adds
the exact receipts below and links the deployment reference; no runtime code or
model/policy changes, no moved/deleted source and no training.

## Build and archive identity

The two clean-source `scripts/build_rrflow.py --profile release` passes completed
in 39m21s and 19m53s. The wrapper checked identical observed compilation closure,
unchanged source, embedded provenance and actual executable digest before writing
its receipt. The outer PowerShell session returned 1 after treating Cargo stderr
progress as `NativeCommandError`; its log ends with the accepted receipt, without
a compiler/build-identity failure. This shell result is retained. Independent
execution of `rrflow.exe --json version` returned 0, exactly matched that receipt,
and matched the clean source and executable SHA256. No compilation retry erased
the shell result.

The native executable is 113,481,216 bytes, SHA256
`fb9b5727e6ea0de668c723e63d008bfd976a02ca020aad218b122b61142afee0`.
`scripts/package_controller_trial.py` completed with exit 0 after compiling the
declared contract helper, validating the actual inputs and publishing exclusively.
The archive is 33,850,986 bytes, SHA256
`46b85f824d712cd8e8cceda4a6b0cd01bb1585f3f25a1f8c64ef8b7657f05b6f`.
Its nine payload files plus manifest contain no model weights, profile, task,
credentials or estate. The local filename is
`rrflow-controller-trial-windows-x64-54fb845d.zip`; no hosted release is claimed.

## Acceptance at the actual artifact

`python scripts/test_controller_trial_package.py --archive <actual-zip>
--sha256 <digest-above> -v`: 18 tests passed, no skips, in 18.940s. Six operate on
the actual archive and corrupted copies: round trip, executable byte change,
missing executable, extra file, escaping path and rehashed inconsistent receipt.
The remaining twelve are the isolated rejection tests. The actual development
receipt was separately rejected by the release-only assembler without output.

After verified extraction outside the engine checkout, every extracted member
matched its manifest digest/size. The packaged proof script, relative receipt,
compiled helper and primary executable ran without source/build tooling. The
actual configured BF16 trial returned 200: 1,805 reported prompt tokens, 20 output
tokens and 1,879 ms adapter duration. Restart read identical attempt/budget;
same-attempt replay was rejected. This single-candidate observation is not a
performance or routing-quality benchmark.

The extracted primary then passed version, authenticated `ready`, stopped/reopen
and `verify --level quick` with its child PATH containing only Windows System32
and with no provider credentials supplied. The Python operator harness remained
outside that child PATH. This is a relocation/default-readiness observation on
the existing Windows host, not a clean-machine or network-denied Gate J test.

## Real failure retained alongside passing cases

Four additional fault histories used actual provider responses. Session closure,
Stop and response disconnect passed their installed/reopen assertions. Stop
occurred after provider response arrival at the relay; no remote GPU termination
claim follows.

Configuration replacement invalidated the first held proposal. The fresh second
request reached the provider, which returned `effort: "observed"` although the
binding allowed only default effort and no named effort values. The engine
correctly rejected that choice with HTTP 412 and `invalid endpoint selection
choice`. The success-expecting proof assertion therefore FAILED and stopped the
case. No output was repaired, relabeled or retried until green. POAM-009 links
this verified model reliability deficiency without changing its open status.

Credential-free recovery retained the original invalidated attempt and the shared
budget with two spent steps, 4,688 ms charged, `attempt_failed`, no active attempt
and conservative execution uncertainty. It made no model call. The packaged
recovery command reads the original request's attempt; it does not separately
read the successor attempt in this case. The actual second response/error and
current budget are retained. The remaining three independent cases ran in new
estates; the failed estate was preserved. Six release-trial provider calls in
total created no training labels or permission.

## Evidence, research and remaining gates

[Receipt manifest](controller-trial-receipts/manifest.json) binds 14 exact copies,
including archive/build identities, all actual check results, the invalid model
response and recovery state. Provider secret values and token/secret fields were
checked absent. Original logs and estates remain in the operator's SWAG evidence
directory. Local `.gitattributes` preserves receipt bytes/CRLF. Source, hashes,
public operations and retained failures supply the trace; no new runtime logger.

PR206 candidate `40973523756` passed all partitions, but protected enqueue reported
the required aggregate check expected, including after a successful rerun of that
real aggregate job. Head SHA, event and GitHub App identity matched. Following
[GitHub's required-check guidance](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks),
a fresh eligible pull-request run `40978912042` was requested on the unchanged
head. All 23 jobs passed, and protected enqueue succeeded at 00:53 UTC on October
30, position 1. Protected merge-group checks and actual landing still remain.
The repository also rejected automatic merge as disabled. No check was fabricated, protection changed or
administrative bypass used. A later skipped draft run shared the head, but its
causal role in the rejection is unproven.

No external algorithm was adapted. Full Rust suites were not repeated locally for
Python packaging-only changes; actual optimized execution and candidate CI are
the relevant evidence. Final navigation, documentation (758 statuses), execution
inventory (2,053 paths), workflow policy, both whitespace checks and all 14 exact
receipt hashes passed. Complete changed-file/diff review found no runtime
authority or sibling-checkout dependency in the payload. No roadmap or POAM gate
is closed: model reliability, source-edit successor plans, worker execution,
Connectome/RROFLOW integration, reviewed learning, container delivery, other
native platforms, clean-machine isolation, signatures, SBOM and full Gate J remain
open. The historical checkpoint remains provisional.

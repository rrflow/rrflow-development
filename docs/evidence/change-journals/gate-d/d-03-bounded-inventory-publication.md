# D-03 bounded Inventory publication on a real repository

**Status:** active implementation evidence; candidate qualification remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-bounded-inventory-publication`
**Owner:** OBJ-14 / D-03 / POAM-006; PR224 driver: active SWAG implementation session

## Bound change and first failure

[PR224](https://github.com/rrflow/rrflow-development/pull/224) declared this
repair before implementation. Baseline `2cfc0bd9f71845ceb45fb0c0d6cf853f0484efd2`;
staged source tree before this journal/index:
`6b589035f2b89b43bc620f0cd691f9ae9180dd37`.
The previous [durable Inventory evidence](d-03-durable-inventory-checkpoint.md)
proved smaller installed projects, not arbitrary repository size.

An actual installed public RRFlow clone, revision
`d1b8a26a73cfb8f01791737913b4098988c1acdd`, tree
`c4cc385adc54705cd5b22cdc26ade218b2ebd4a0`, with 1,870 tracked paths,
rejected Inventory with HTTP 429: encoded batch bytes 16,777,956 exceeded the
native WAL ceiling of 16,777,216. The configured 64 MiB memory allowance does
not change that format limit. Reopen proved cursor 2, the original Connect
checkpoint, no source tables and no Inventory completion; quick verify passed.
The failed estate is preserved. No source exclusions or physical limits were
changed to make this corpus pass.

## Canonical implementation and review

Read the owning README, D-03, POAM-006 and change-authoring procedure; the
installed source walk/proposal, selected-plan/replan implementation, runtime
record/schema/read reconstruction, native WAL/batch and TransactionWriteSet,
semantic commit/control builder, admission and installed tests. Reviewed the
complete changed files and diff. Scope is limited to:

- `engine/attunement/inventory.rs`: prepare and admit the complete logical
  proposal before publication, retain one writer permit and source recheck.
- `inventory/records.rs`: partition complete entry/parent-edge groups;
  snapshot/change manifests, root edge and retirements remain in final publication.
- New `inventory/publication.rs`: preserve the small atomic path; partition
  only after typed, known-before-publication `WriteBatchLimit`. Never retry an
  uncertain I/O, authorization or conflict result. Each group commits with a
  guarded job CAS and private job/phase/plan/proposal/read-bound progress.
  Validate progress on reopen; skip its prefix only when proposal, prior head
  and current authenticated runtime identity match. Intervening changes restart
  preparation rather than trusting a stale prefix. Only final publication adds
  the complete snapshot, head, phase checkpoint and command receipt, and removes
  exact progress. Deadline, lease, authorization and invocation checks remain.
- `engine/transaction_admission.rs`: composed charges grow monotonically;
  publishing a smaller group cannot release the retained preparation reservation.
- `engine/tests/attunement.rs`: large installed publication, interruption/reopen
  and changed-source correctness oracles. These authored fixtures are not training
  data, evaluation labels or worker outcomes.

Partial canonical observations can exist after interruption. They are not a
complete project snapshot. Consumers must use the published head and its
authenticated read identity; an earlier complete snapshot retains its old read.
There is no second inventory database, staging-file lifecycle, public operation,
plan authority or native format change. Existing version/audit retention owns
partial observations. Final manifest/retirement size still must fit one physical
batch; this repair does not claim unlimited project size.

## Installed executable result

Unoptimized copied primary SHA-256:
`9105fe3a3dd207174cbb26f65e5537b8e06e1f274edaa1c45e0d70f17ab4bb69`.
The installed locator correctly rejects this different executable against the
old estate with `ProjectBindingMismatch`. No binding was rewritten. Safe binary
upgrade remains an explicit deployment gap; this is not an upgrade acceptance.

A separate clone of the exact same revision/tree completed install/serve,
authenticated Connect and Inventory through the frozen generated Python SDK.
Inventory took 14.042 seconds, producing cursor 4281 and manifest
`94f0ce79b129dd48bd05008ee97b6757bb95450d15c5ecaffd2da0ab9767266d`.
After process termination/reopen, a controlled README edit received an accepted
successor plan and another Inventory in 17.046 seconds: cursor 8559, manifest
`bb656a0761aa4183b521097cc2a30f1657ee646d89f503206809db1af7876021`.
A third process read the exact second job back; offline quick verification
passed. The README was restored afterward, so that final source edit correctly
makes the second plan stale. Timings are one Windows correctness run, not a
performance qualification or a live integrated controller claim.

The first fresh-clone plan attempt returned `ProjectBindingMismatch`; retained
evidence does not identify the exact changing filesystem input. The same
uninstalled clone subsequently passed without relaxing validation. A background
Git configuration change is a hypothesis, not a proven cause.

## Validation and retained failures

Windows commands used `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2` and the
existing external Cargo target cache, not an external source dependency.

- `cargo test -p rrd-engine --lib --locked`: 275 passed, four existing ignored,
  before the final changed-source test addition. Includes a 2,048-file fixture
  spanning multiple physical commits and reopen; interruption after a durable
  group resumes with exactly 2,052 unique entry observations and no premature
  snapshot/checkpoint. Root, directory, README and excluded `.rrflow` account
  for the four additional entries.
- `cargo test -p rrflow-cli --test installed_lifecycle --locked`: seven passed.
- `cargo clippy -p rrd-engine --all-targets --locked -- -D warnings`: passed
  before the final test addition. `cargo build -p rrflow-cli --bin rrflow --locked`
  produced the primary used above; its build envelope predates that test-only edit.
- `cargo test -p rrd-engine --lib --locked inventory_edit_during_publication_rejects_stale_completion_and_allows_replan`:
  passed. A real source edit after a committed observation group rejects final
  publication; reopen/recovery still rejects stale resume, then accepts a new
  source-bound successor without inventing a checkpoint.
- Final `cargo test -p rrd-engine --lib --locked`: **276 passed, four existing
  ignored** (58.11 seconds), including both interruption variants. Final strict
  all-target engine Clippy, Cargo formatting, generated-surface parity (70 HTTP
  operations), documentation ownership/link checks and diff whitespace passed.
  Navigation and the 2,093-record execution inventory were regenerated.

Retained failures: the genuine physical batch rejection; a large-test artificial
clock preceding the previous step's completion; an interruption-test count
omitting the excluded marker; the initial rejection-readback oracle expecting
no built-in schema; different-binary admission; and fresh-clone plan rejection.
Corrected tests retain their actual denial/readback/uniqueness requirements.

Local raw evidence is under
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/`:
`installed-repository-*`, `resumed-repository-*`, `bounded-publication-*` and
`bounded-repository-*`. Source/binary hashes, corpus revisions, stage durations,
job/checkpoint coordinates, failed logs and successful readback are retained;
credentials and session tokens are not printed in those results.

No new upstream implementation was adapted. Native timestamp primary-source
research did not justify enabling metadata reuse; content rereads remain.
Existing control/runtime journals and bounded test logs provide diagnostics;
no tracing service was introduced. Logical admission is not a measured RSS cap.

## Remaining qualification and checklist

Declaration, owner review, first-failure oracle, canonical implementation,
installed proof, failure preservation, source/diff review and linked evidence are
complete for this bounded repair. POAM-006 records the new physical-size and
upgrade limitations. No roadmap requirement closes. Safe metadata reuse/no-work
checkpoints, broader native races and immediate cancellation, final-manifest and
retirement scale, memory accounting, old incomplete-observation retention and
Linux/macOS remain unqualified. Final source recheck is bounded but is not an
OS-atomic filesystem snapshot. Explicit Cancel still serializes behind the writer.
Parse/later workers, Connectome task completion, RROFLOW and reviewed learning
remain incomplete. Candidate/protected CI and merge are pending; PR224 remains
draft. Stop on fabricated completion, bypassed physical bounds or a second authority.

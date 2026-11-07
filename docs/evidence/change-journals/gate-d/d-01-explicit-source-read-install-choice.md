# D-01 explicit source-read install choice

**Status:** active implementation evidence; dependent native integration and candidate CI remain pending  
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-01-explicit-source-read-install-choice`  
**Owner:** PR291 native install source-read option

## Gate/package outcome

This package advances the installed source-aware worker prerequisite: a new
installation can explicitly include `ProjectSourceRead` in its sealed initial
grant list. It does not close D-01 or POAM-016, and it does not prove the full
project-indexing journey by itself. The installed process test now demonstrates
the controlled source-aware worker journey for an opted-in estate, including
typed pre-dispatch denials, serve/reopen, fresh authentication, protected
readback, and offline verification.

## Starting point and scope

- Starting commit: `6df9990725135bc21ffe064328cd2e06eec8a9ea`.
- Starting tree: `c26f57b6269aaefd7297908527cf1640f6046d20`.
- Review brief: PR291, published before implementation; parent verified its
  body against `install-source-read-brief-published.md`.
- Changed implementation: CLI install planning, engine planning/sealed apply,
  contract preview scope, engine installation tests, and installed lifecycle
  test. Changed references: deployment lifecycle and security authority.
- This is a new-estate plan choice, including an existing-project install
  target. It is not a post-install grant editor or arbitrary security grant
  manager. No existing estate was modified.
- Default-off planning retains the legacy serialized default preview for the
  same executable identity. Only the exact default grant list and that list
  plus `ProjectSourceRead` are accepted by current sealed planning/apply;
  generic historical plan validation remains broad for supported older plans.
- Opt-in preview exposes `source_read_resource_scope: "instance_prefix"`.
  The scope field is omitted for the default preview, is validated against the
  exact initial grant list, and cannot be changed without changing the
  digest-bound plan authority. Source export, context export, and
  `MemoryContextRead` remain separate controls.

## Files and owner material reviewed

Read the repository `AGENTS.md`, README owner references, PR291 brief and
handoff, `docs/roadmap/rrflow-1.0-execution/change-authoring.md`, the full
installation planner/apply/recovery/binding path, CLI parsing and apply path,
contract validation and contract fixtures, public client/server worker path,
installed lifecycle and source-bound worker tests, deployment/security
references, Gate D journal index, and POAM-016. The complete changed diff was
reviewed after implementation.

## Implemented behavior and evidence

- `rrflow install plan --allow-project-source-read` is the single explicit
  default-off switch. Planning includes the source-read grant only when the
  flag is supplied. Apply still requires the exact accepted digest.
- Engine installation tests cover the two canonical initial grant lists,
  deterministic scope, stale expected digest with no effects, and rejection of
  a rehashed noncanonical list before effects. Historical generic plan
  validation remains compatible with its four-grant contract fixture.
- Installed lifecycle test creates two isolated installations using public
  plan/apply: default denial and explicit opt-in. The opt-in project has a
  minimal Git config before initial inventory and leaves `source.txt`
  untracked. A later `.gitignore` excludes the selected path without changing
  its bytes; the public worker API returns `FailedPrecondition` and the
  loopback listener remains empty. Removing that rule permits a same-source
  positive control. Missing source grant and separately disabled source or
  context export return `PermissionDenied` with no worker dispatch. The
  successful bounded call sends the selected bytes to the controlled loopback
  endpoint, then serves/reopens, reauthenticates, reads back the retained
  protected result and source digest, and passes offline verification after
  server shutdown.
- The endpoint is local controlled protocol fixture only. No live model,
  provider, training, or quality claim is made.
- The frozen prior executable rejects the new CLI flag during parsing with
  exit 2 and no project effects.
- Parent's fixed-executable comparison found old/new default preview stdout
  byte-identical: 17,884 bytes, SHA256
  `570c9ba40ba42d4a429cfaa660fc0c5db6f589b30ba83b76b24269951382e68a`.
  The opt-in preview reports `instance_prefix`; the comparison used the fixed
  debug executable as a surrogate only and did not imply an old-estate upgrade.
  The initial retained baseline file was UTF-16LE BOM/CRLF, while the executable
  emits the same JSON bytes as UTF-8; the parent retained that encoding
  correction separately and kept the original receipt.

## Checks

All Cargo checks used `CARGO_BUILD_JOBS=1` and reused the authorized
`source-bound-worker-worktree/target` build cache. Source files were hashed
before and after each final run and no source changed during those runs.
Host-captured process output, exit status, and source-hash records for attempts
6 onward are retained in the ignored private local evidence set
`install-source-access`.

| Command | Result |
| --- | --- |
| `cargo test --locked -p rrflow-cli --test installed_lifecycle installed_source_read_choice_is_explicit_and_reopens -- --nocapture` | 1 passed, 0 failed, 7 filtered; exit 0 |
| `cargo test --locked -p rrflow-cli --test installed_lifecycle installed_source_read_choice_is_explicit_and_reopens -- --nocapture` after helper refactor | 1 passed, 0 failed, 7 filtered; exit 0 |
| `cargo test --locked -p rrd-engine --lib engine::tests::installation` | 44 passed, 0 failed, 1 ignored, 333 filtered; exit 0 |
| `cargo test --locked -p rrd-engine --lib engine::tests::reasoning_budget::worker` | 23 passed, 0 failed, 355 filtered; exit 0 |
| `cargo test --locked -p rrflow-cli --test installed_lifecycle` before the test-helper refactor | 8 passed, 0 failed; exit 0 |
| `cargo test --locked -p rrd-contract` | All contract test binaries passed; exit 0 |
| `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | Exit 101; found two too-many-arguments diagnostics in test helpers. Helpers were grouped into `InstalledWorker` and `WorkerAttempt` without lint allowances. This broad workspace attempt was not repeated. |
| `cargo clippy --locked -p rrflow-cli -p rrd-engine -p rrd-contract --all-targets --all-features -- -D warnings` | Pass; exit 0 after the helper refactor |
| `git diff --check` | Pass |
| `rustfmt --check --edition 2021` on all nine changed Rust files | Pass after applying rustfmt |
| `python scripts/knowledge/render_navigation.py --check` | Pass after journal index regeneration |
| `python scripts/ci/build_execution_inventory.py` | Wrote 2,638 current, generated, and planned path records; exit 0, no additional tracked diff |
| `python scripts/ci/build_execution_inventory.py --check` | Pass: 2,638 path records; exit 0 |
| `python scripts/ci/check_documentation.py` | Pass: knowledge ownership, 1,006 document statuses, 1,004 classified coordinates, parent indexes, and local links; exit 0 |

The contract command ran 24 test binaries; exact per-binary counts are in its
retained host-captured output. For attempts 6 onward, stdout/stderr were
captured through PowerShell redirection: stdout contains Cargo output, while
native stderr may be formatted by PowerShell as `NativeCommandError`; those
stderr files are not byte-for-byte OS stream captures. This also applies to
the Clippy captures. Earlier focused attempts
1–5 have separate short notes, not process-output logs. No stdout/stderr was
retained for those attempts.
Attempt 5's first summary overstated denial causes: it established only that
the listener remained empty while the API returned some error. This was
corrected in `attempt-5-review-correction.txt`; the tightened typed-error and
ignored-path oracle passed in attempt 6.

After the final journal wording update, the first execution-inventory check
reported drift. The canonical inventory generator was run again, and the
subsequent `--check` passed with 2,638 path records; no extra tracked path was
generated.

## POAM-016 and remaining limitations

POAM-016 stays open and parent-owned; see the current
[POAM-016 disposition](../../../poam/rrflow-1.0-alpha/poam-016.md).
This new initial source-read choice makes the controlled journey available to
newly installed estates with explicit operator acceptance. It does not alter
the immutable bootstrap principal or resolve the known fail-closed behavior
after unsupported post-install grant mutation; that restart/reopen limitation
remains open under POAM-016. No arbitrary existing-estate migration or grant
modification is supported.

The dependent UI source-read choice, source-bound worker integration and its
separate ownership remain outside this worktree. Candidate CI, protected
landing, and final reconciliation are parent-owned and remain pending. No
candidate or merge-group CI was run here.

## Change checklist and next action

- No roadmap completion or POAM status change.
- No external research was needed; the existing public lifecycle and owner
  references were the source of truth.
- No runtime trace or resource claim was needed; host-captured test output and hashes
  were retained for the focused process and package checks.
- Reread the changed files and complete diff; `git diff --check` passes.
- Next action: parent reviews this candidate, integrates dependency changes
  through the normal PR process, and owns candidate CI/landing and desktop
  work.

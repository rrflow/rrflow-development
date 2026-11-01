# User-selected nested repository boundaries

**Status:** active contract and diagnostic evidence; implementation remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/nested-repository-boundary-decision`
**Owner:** OBJ-07 / OBJ-14, D-01 / D-03 / H-06, POAM-006
**Pull request:** [232](https://github.com/rrflow/rrflow-development/pull/232)

## Baseline and bounded outcome

Development main `5b4039fe930332895c1b24138891d39f600030c7`, tree
`3c8e935c29e879bccef6162e257e3ebdfaffc049`. PR232's brief was published
at empty anchor `40b6d809bd32b324fc5518ad9749c1c7a7d24b00` before repository
edits. The push destination was public `development`,
`https://github.com/rrflow/rrflow-development.git`, ref
`refs/heads/codex/nested-project-boundaries`, exact revision `40b6d809...`.
No official promotion was attempted.

The owner requests that setup detect nested repositories and offer exclusion,
separate projects/instances (recommended), or both repositories in one
instance; the user decides. The topology owner now specifies one or more
explicitly selected source repositories in one project while keeping one
project/estate/engine authority. Gate D states engine-owned preview, exact
binding, filesystem admission and real-project proof. Connectome presents the
engine proposal. POAM-006 records the observed gap. This package changes no
runtime or product status and does not implement these choices.

## Real source and research

A source-built Windows executable from the baseline passed
`cargo build --locked -p rrflow-cli --bin rrflow` (two build jobs, dev debug
information disabled; 9m41s). A separate copy with one file link has SHA-256
`bd5bbad1caed19969b5b89c2572d18021d4a0dc5d6a445d1d782ba3b6c57177b`.
Read-only `rrflow-baseline.exe --json install plan --mode existing --project
<root> --identity <generated>` was run for each real root. Connectome failed
`inventory Git input: external or non-directory Git metadata requires
admission`; its `.git` points to `../.git/modules/connectome`. The parent
walker recognizes the gitlink, records it as excluded and does not descend.
The root RRFlow checkout separately failed `private object requires the
current owner and a protected non-null DACL`; the exact rejected object is
unknown. The existing `.rrflow` directory has inherited ACL entries, but this
is not sufficient evidence to identify the failing object. No ACL or source
file was changed and no plan was applied. Initial sandboxed previews failed
access denied; the real-checkout errors are from approved unsandboxed retries.
The first direct Cargo executable also had two hardlinks and was rejected by
file admission; copying it independently exposed the Gitfile failure. The
local diagnostic receipt is `target/real-project-build/preview-evidence.md`.

The primary [Git submodule documentation](https://git-scm.com/docs/gitsubmodules)
describes a Gitfile pointing at superproject module metadata; the
[Git worktree documentation](https://git-scm.com/docs/git-worktree) describes
shared and per-worktree metadata; and [gitmodules](https://git-scm.com/docs/gitmodules)
explains parent relative paths and rejects a symlinked `.gitmodules`. These
are input semantics, not authorization to follow a path. Future implementation
must use explicitly admitted retained filesystem authorities, bounded inputs,
identity/race checks and exact-plan revalidation. No upstream code, Git
subprocess, runtime fetch or new external dependency was added.

## Review, checks and remaining work

Read in full: README; AGENTS; change-authoring procedure; instance topology;
Gate D; Connectome contract; POAM-006; ADR-0002; source inventory walker,
GitInputs, inventory proposal; the prior Git inventory and installed-trial
journals; and this record. Inspected installation plan types, the filesystem
anchor, Windows private-file admission, and the generated PR template. The
production implementation was inventoried, not rewritten. The reviewed source
paths are `crates/authority/rrd-engine/src/engine/installation/inventory.rs`,
`inventory/git_inputs.rs`, `inventory/proposal.rs`, and
`crates/persistence/rrd-lsm/src/filesystem.rs`.

Changed: `docs/architecture/instance-topology.md`,
`docs/roadmap/rrflow-1.0/gate-d.md`, `docs/reference/client/connectome.md`,
`docs/poam/rrflow-1.0-alpha/poam-006.md`, this record and generated navigation /
execution inventory. No files were moved or deleted. The implementation trace
maps the existing gitlink exclusion to the future D-01/D-03 preview and H-06
presentation; no existing source behavior is deleted by this package.

Trace/resource/debug decision: retain the exact baseline, binary digest and
read-only failure; no payload logging or content trace is added. Resource and
latency measurements do not apply to a documentation-only change and no
performance claim is made. Future implementation must bind source identities,
metadata admission and receipts while preserving bounded work and redaction.

Validation on this candidate worktree:

- `python scripts/knowledge/render_navigation.py`: passed; 18 indexes, 848
  nodes, 3,087 edges, one index updated. Its `--check` passed with no changes.
- `python scripts/ci/build_execution_inventory.py`: passed; 2,165 records;
  its `--check` passed. The generated inventory had no tracked difference.
- `python scripts/ci/check_documentation.py`: passed; 849 statuses and 847
  classified coordinates, with parent indexes and local links checked.
- `git diff --check`: passed.

The first baseline-tree lookup used an unquoted PowerShell Git revision
expression and failed. Quoting it produced the tree identity recorded above;
no repository content changed. The execution-inventory recheck later reported
drift after a journal edit; regenerating its projection corrected it. Candidate
CI and merge remain pending. No local engine or UI test is claimed for this
contract-only change; no real installation, indexing, restart, separate/shared
instance, graph/vector, hosted entitlement, idle/wake or cross-project link was
run. The source-level design remains unimplemented and POAM-006 remains open.
No roadmap gate or objective changes status.

Checklist: owner and brief bound before edit; observed failures and primary Git
references retained; source/acceptance boundaries and stop conditions stated;
new verified gap recorded in POAM-006; no implementation removed; generated
navigation, documentation checks, full-file reread, final diff review and CI
receipt required before this package may be ready.

# One-project proof and resource-managed instance direction

**Status:** active local documentation and bounded developer-session evidence; no gate closed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/one-project-and-resource-managed-instances-20261106`
**Owner:** H-06/H-07 direction and POAM-011; roadmap owns implementation and acceptance

## Brief and scope

Starting RRFlow main: `1fd7f163de3448c694e09c916b328540303eab1c`; its
Connectome gitlink is `c1cbff203ecee89013878609d11aed04e89d0e71`. OBJ-11,
H-06/H-07 and POAM-011 remain open. This documentation package records the
owner's sequence: demonstrate one project first; later admit and manage
multiple independently authenticated instances against configurable resources;
keep enterprise control-plane deployment optional and separately maintained.
It adds no engine or UI implementation and makes no completion claim. Stop if
the scope requires deciding a licence, changing release status, moving code or
creating another engine authority.

The change brief was assembled during review but was not written to the PR
description before the first documentation edit. That missed the repository
change-authoring procedure; the PR must disclose the deviation and cannot use
this journal to imply otherwise. The PR body is to be submitted with the first
coherent commit because GitHub needs a differing commit to open a PR.

## Inspected sources and findings

Inspected the owner instructions, README, roadmap delivery path and H-06/H-07
sections, alpha objectives, POAM-011, topology and process-adapter contracts,
Connectome client contract, knowledge connector status, change-authoring and CI
procedure. Inspected the Connectome pinned `lifecycle.rs` constants and its
conditional checks; the separate locally run palette worktree was at
`30c7803c219ec7c99e326455eed48ad5074d229a`. No whole-repository or whole-file
code review is claimed. The pinned source defines four owned and eight tracked
instances; limits prevent unbounded work but do not implement the requested
configurable, resource-aware admission. `MAX_KNOWN_PROJECTS = 64` is a separate
bounded status-read input and is not treated as an instance entitlement.

Primary research: [MariaDB BSL 1.1](https://mariadb.com/bsl11/) changes
each publicly distributed version to its specified change licence on its change
date or its fourth distribution anniversary, whichever is earlier; the owner's
six-year intent therefore is not a standard BSL 1.1 term. Exact licence and
self-host grant need a separate decision. [Qdrant hybrid queries](https://qdrant.tech/documentation/search/hybrid-queries/)
describe configurable retrieval/fusion and
[SurrealDB record links and relations](https://surrealdb.com/docs/learn/data-models/graph/record-links-vs-graph-relations)
describe different relationship representations. These are research inputs, not
claims of implemented connectors or performance, and no upstream source is
adapted. No new trace, runtime signal or resource policy is implemented here.

## Bounded live observation

On this Windows workstation, the configured SWAG project connection addressed
an unrelated already-running instance on port 8792; the expected instance
identity was rejected. Its intended developer daemon was stopped. The existing
SWAG local developer-stack configuration and connection file were changed to
loopback port 65515, retaining the intended project, instance and credential;
the original files are backed up under `target/swag-port-recovery-20261106/`.
The unrelated daemon was not stopped. The normal developer launcher started
the intended project daemon, PID 32612. The SWAG MCP status returned an
authenticated response for `instance-1acc93a00413ac451f247f75`; this proves a
status operation only, not a model or prompt-to-context execution. The binary
was the existing SWAG developer build, sourced from `5319a0c5...`, not the
starting RRFlow main candidate. The workspace HTTP endpoint returned 200.

The existing Connectome palette developer binary, exact source-to-binary
provenance not independently established, was opened against its local Vite
development frontend. A new profile bound the same endpoint and expected
instance, and the native process opened a session using that project's existing
operator credential file. The UI displayed `RRD CONNECTED`, six records, four
relationships, zero vector collections and live rrflowKV counters. At the
observed instant the storage view showed 1,389 memtable versions, 711 keys,
1.89 MiB WAL and zero segments. These are one running project's observations,
not a correctness, crash recovery, indexing, competitive or H-06 qualification.
The existing user's modified Connectome checkout was not edited. The local
model endpoint was unavailable; real LFG-to-SWAG causal routing and external
SurrealDB/Qdrant knowledge paths were not demonstrated.

## Paths, checks and remaining work

Changed README, the roadmap delivery path, its H-06/H-07 chapter, and POAM-011;
created this journal and regenerated its navigation index. No code, licence,
release status, process owned by another application or prior evidence was
removed or rewritten. The one-project indexing/prompt/restart driver and
multi-instance resource manager remain to be implemented and qualified at their
existing owners. Exact licence and authorized self-hosting scope remain open.

`python scripts/knowledge/render_navigation.py` passed and updated one index;
`python scripts/knowledge/render_navigation.py --check` passed (18 indexes,
967 nodes, 3437 edges). `git diff --check` passed. The full changed-document
diff and this record were reviewed; candidate CI, native platform matrices, full installed
journey, soak and competitive benchmarks were not run for this documentation
change. A candidate PR, CI result and merge are not yet established. This
journal does not change roadmap or POA&M status.

## Candidate reconciliation review (2026-11-09)

Reviewed PR #288 at `12d7decfff66752a89f50bc1ed3ab01c3af3bfa8` against public
development main `a9f1b0fb8d948e113dca347e5eeac10dd224330a`. The old candidate
run `42447832343` failed documentation policy, engine architecture, optional
engine feature, and verification jobs on the old base. The documented C-07
whitespace finding was checked against refreshed main and is absent there; the
new candidate also needs its own CI run. A current-main merge preview was
conflict-free. The package remains documentation-only and does not establish
the one-project runtime flow or close H-06/H-07.

On the refreshed merge tree, `python scripts/ci/check_documentation.py` passed
(1,034 statuses and 1,032 classified coordinates),
`python scripts/knowledge/render_navigation.py --check` passed (18 indexes,
1,033 nodes, 3,610 edges), and
`cargo test -p rrd-engine --test workspace_architecture --locked` passed
(33 tests, including the tracked/untracked trailing-whitespace guard). The
journal status was normalized to the repository's recognized `active` label;
the Gate-H index was regenerated. Candidate CI and merge-group CI have not
been run on the reconciled revision at the time of this record. No release,
objective, roadmap, or POA&M status changed.

`git diff --cached --check` while preparing the merge commit reported CRLF
line endings as trailing whitespace across files brought in from current main.
No mass line-ending rewrite was made. The review-relevant
`git diff --check a9f1b0fb8d948e113dca347e5eeac10dd224330a..HEAD` passed on the
actual PR delta; `git diff --check` for the two documentation corrections also
passed. The architecture whitespace test passed on the merged tree.

# RRFlow CI and runner operations

**Status:** active operational contract; current checks are not RRFlow alpha or release evidence
**Coordinate:** `rrflow://rrflow-instance/data/operations/ci`
**Owner:** candidate CI topology, workflow supply-chain policy, diagnostic workflow boundary, and private runner procedure; indexed by `docs/operations/README.md`

This record describes checked-in automation and the operator steps around it.
The [release roadmap](../roadmap/rrflow-1.0.md) alone owns gate status, the
[repository run checklist](../roadmap/rrflow-1.0-execution-map.md#repository-wide-run-checklist)
owns package-level widening order, and the [POA&M](../poam/rrflow-1.0-alpha.md)
owns deficiencies. A green workflow proves only the commands and assertions it
actually executes.

## Candidate chain

RRFlow has one candidate CI chain. The thin caller is
`.github/workflows/ci.yml`; `.github/workflows/ci-reusable.yml` owns every
check. A branch push does not start a second copy of the same work.

The caller accepts pull requests, merge-queue candidates, and explicit manual
dispatches. Its concurrency key cancels stale executions for the same pull
request or ref. The reusable workflow reduces every partition into one stable
`ci-gate` check for branch protection:

- supervised RRD daemon smoke on the standard Linux runner;
- repository policy, formatting, architecture, evaluation, dependency, and
  binary-budget gates on a heavy Linux runner;
- five default-feature engine suites on isolated heavy Linux runners:
  `kernel-storage`, `query-retrieval`, `engine-authority`, `service-protocol`,
  and `product-operations`;
- one current shared SDK harness, isolated optional-feature checks, and a real
  PostgreSQL/pgvector integration;
- daemon portability on GitHub-hosted Windows and macOS runners; and
- one always-evaluated gate which fails unless every partition succeeded.

The topology job starts one local RRD daemon and checks readiness and
capabilities. It does not start Connectome or prove installation, rrflowMX and
rrflowKV semantic equivalence, native graph/BM25/vector projections, streamed
Arrow/DataFusion execution, persistent reasoning/context, or restart recovery.
The shared SDK harness is also current characterization: the deficiencies and
real-process corpus required for H-04 remain open. Neither job may be cited as
complete-engine evidence.

The repository-policy partition also verifies the generated
`docs/roadmap/rrflow-1.0-file-plan.jsonl`. That inventory must match every
tracked or non-ignored path and every gated future path before code execution
begins; regenerate it with `python3 scripts/ci/build_execution_inventory.py`
after an accepted path or content change.

The same partition enforces the documentation bootstrap package. The
documentation policy builds and validates two in-memory packages through the
single exporter, while the workflow exports twice at the checked-out revision,
compares the exact JSONL bytes, and runs the focused drift corpus. Package
outputs remain temporary and are never editable documentation authority.

The five engine suites cover every workspace package exactly once, but package
names do not define product ownership. The partitioning avoids one
workspace-wide test/link target graph on the hosted fallback without turning
all 20 implementation packages into separate product checks.

## Repository workflow policy

`scripts/ci/check_workflow.py` discovers every checked-in `.yml` and `.yaml`
under `.github/workflows/`; a newly added workflow cannot escape its
supply-chain checks. It requires:

- every external action to use a full 40-character commit SHA;
- every workflow service image to use a SHA-256 digest;
- every checkout step to set `persist-credentials: false`;
- top-level read-only contents permission and no `pull_request_target` in any
  workflow; and
- complete candidate-suite/default-feature/optional-feature coverage, bounded
  candidate jobs, safe fork routing, and exact final-gate reduction.

Workflow ownership is directory-wide in `.github/CODEOWNERS`, so adding a new
workflow does not bypass the existing workflow review boundary. Repository or
organization settings should additionally require full-SHA actions; that
external GitHub setting cannot be proven by this checkout.

## Scheduled storage diagnostics

`.github/workflows/rrd-lsm-benchmark.yml` is separate from candidate CI. It
runs manually and weekly on `ubuntu-latest`, uses the same immutable checkout,
toolchain, and cache action revisions, and uploads raw semantic-storage and
AI-access results with a pinned artifact action. It is intentionally not a
required `ci-gate` dependency.

The [rrflowKV benchmark reference](../reference/storage/rrflowkv-benchmark-harness.md)
owns those executable shapes and their limitations. Hosted runner output lacks
the fixed hardware, complete provenance, full governed engine flow, and
quality/failure corpus required by J-04; scheduling and artifact upload cannot
turn it into performance or competitive evidence.

## Runner trust boundary

The repository variables below contain JSON `runs-on` values:

| Variable | Physical pool value | Safe default |
| --- | --- | --- |
| `CI_SELF_HOSTED_ENABLED` | `true` | disabled |
| `CI_LINUX_STANDARD_RUNNER` | `["rrflow-standard"]` | `["ubuntu-latest"]` |
| `CI_LINUX_HEAVY_RUNNER` | `["rrflow-heavy"]` | `["ubuntu-latest"]` |

A pull request whose head repository differs from the base repository is
forced onto GitHub-hosted Linux even when the variables exist. Missing or
disabled `CI_SELF_HOSTED_ENABLED` also forces hosted Linux. The workflow does
not use `pull_request_target`. Same-repository branches, merge groups, and
manual proofs may use the private physical pool.

This routing expression is defense in depth, not permission to connect a
self-hosted pool to a public repository: a fork can propose a change to the
workflow itself. The installer queries GitHub and fails closed unless the
repository visibility is `PRIVATE`. Do not set the physical routing variables
or register the scale sets while the repository is public.

GitHub recommends ephemeral runners for autoscaling and identifies Actions
Runner Controller (ARC) as its reference Kubernetes implementation. The
checked-in deployment is configured as follows:

- ARC chart version: `0.14.2`; controller, runner, and Docker images are
  digest pinned;
- `rrflow-heavy`: scale 0..1, 48 CPUs and 64 GiB total, isolated Docker daemon;
- `rrflow-standard`: scale 0..2, 16 CPUs and 20 GiB each; and
- maximum runner allocation: 80 CPUs and 104 GiB, leaving 8 CPUs and 24 GiB
  for the 88-CPU/128-GiB host, k3s, ARC, and filesystem cache.

Each job receives a fresh pod and an empty work volume. No runner worktree is
mounted from the host. The runner service account has no Kubernetes API
permissions, and only the heavy class has a privileged Docker sidecar.

References:

- [GitHub: self-hosted runners](https://docs.github.com/en/actions/reference/runners/self-hosted-runners)
- [GitHub: deploy ARC runner scale sets](https://docs.github.com/en/actions/how-tos/manage-runners/use-actions-runner-controller/deploy-runner-scale-sets)
- [GitHub: secure use of self-hosted runners](https://docs.github.com/en/actions/reference/security/secure-use)

## Physical-host bootstrap

Install a pinned single-node k3s `v1.36.3+k3s1` cluster on the physical host
and make its kubeconfig available to the administration machine. Do not share
this cluster with production workloads. Install Helm 4 and authenticate `gh`
as a repository administrator. The checked-in script does not install or
attest those prerequisites. Run it from the target checkout with an explicit
repository identity, or let `gh repo view` resolve the current checkout:

```bash
GITHUB_REPOSITORY=OWNER/REPOSITORY \
  KUBECONFIG=/path/to/physical-host.kubeconfig \
  scripts/ci/install-arc.sh
```

The installer validates an `OWNER/REPOSITORY` coordinate and denies
public-repository registration before creating separate controller and runner
namespaces. It streams the existing GitHub token directly into a Kubernetes
Secret, installs the version-selected OCI charts with digest-pinned images,
waits for the controller, installs both scale sets, and sets the repository
routing variables. It does not write the token to argv or a values file.

The chart fetch, GitHub API calls, and Kubernetes operations require network
and administrator access. This is CI-infrastructure bootstrap, not the RRFlow
offline distribution or per-project installation path, and no checked-in
record currently proves that the physical pool is installed.

Prove physical execution after installation:

```bash
gh workflow run ci.yml --repo OWNER/REPOSITORY \
  --ref <candidate-branch>
gh run watch --repo OWNER/REPOSITORY --exit-status
```

The standard and heavy jobs must report `rrflow-standard` and `rrflow-heavy`
runner identities. Windows, macOS, the final gate, and all fork pull requests
must remain GitHub-hosted.

## Repository enforcement

After the first physical proof succeeds, protect `main` with these invariants:

- pull requests and the stable `pipeline / ci-gate` check are required;
- merge queue or an up-to-date branch is required;
- force pushes and deletion are denied; and
- Actions must require immutable full-SHA references.

The repository CI-policy check rejects candidate trigger duplication, unsafe
fork routing, unpinned actions, service or runner images, persisted checkout
credentials, over-broad workflow permissions, `pull_request_target`, missing
candidate timeouts, incomplete suite or feature coverage, missing gate
reduction, or a runner layout which exceeds the documented host allocation.
The Rust workspace architecture test remains focused on engine ownership,
dependency direction, and storage-opening authority rather than GitHub Actions
implementation details.

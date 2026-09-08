# RRFlow

RRFlow is one reasoning-data engine for AI governance, reasoning, recall, and
bounded context assembly. RRD is its embedded and daemon runtime; rrflowDB,
rrflowKV, rrflowMX, rrflowQL, Arrow/DataFusion, graph, lexical, vector,
inference, and client capabilities remain parts of that one system.

This README is the bootstrap product entry point and knowledge map. It owns
RRFlow's identity, non-negotiable architecture invariants, and current status;
it delegates each detailed subject to exactly one linked memory record. The
RRFlow 1.0 release checklist is owned by
[`docs/roadmap/rrflow-1.0.md`](docs/roadmap/rrflow-1.0.md). Supporting records
may explain or prove an owner, but cannot override it.

The target release-train version is `1.0.0`. It is frozen while the alpha
baseline is established. Current maturity is **pre-alpha**. The version is a
contract line, not a claim that the complete system is stable, optimized, or
release-ready.

## Enter RRFlow

| Need | Single owning record |
|---|---|
| Understand the product, components, and boundaries | [RRFlow system overview](docs/architecture/system-overview.md) |
| Understand project, estate, RRD instance, environment, deployment, and physical placement | [RRFlow instance topology](docs/architecture/instance-topology.md) |
| Select and prove deployment form, storage profile, and endpoint presentation independently | [RRFlow deployment profiles](docs/reference/deployment/modes.md) |
| Deploy one installed RRD instance through Kubernetes without creating another authority | [RRFlow Kubernetes deployment adapter](docs/reference/deployment/kubernetes-operator.md) |
| Understand the unavailable distributed target and disposition of current cluster code | [RRFlow distributed cluster contract](docs/reference/distributed/cluster-contract.md) |
| Follow writes, persistence, reads, Arrow/DataFusion, and context end to end | [RRFlow engine data flow](docs/architecture/engine-data-flow.md) |
| Understand why all capabilities remain under one authority | [ADR-0001: single-engine authority](docs/decisions/0001-single-engine-authority.md) |
| See the measurable alpha result | [RRFlow 1.0 alpha objective](docs/objectives/rrflow-1.0-alpha.md) |
| Execute work in dependency order and inspect accepted evidence | [RRFlow 1.0 release roadmap](docs/roadmap/rrflow-1.0.md) |
| Inspect verified gaps and remediation ownership | [RRFlow 1.0 alpha POA&M](docs/poam/rrflow-1.0-alpha.md) |
| Map a gate to exact files, symbols, tests, and stop conditions | [RRFlow 1.0 code execution map](docs/roadmap/rrflow-1.0-execution-map.md) |
| Inspect frozen product-version and change-control rules | [RRFlow version policy](docs/reference/release/version-policy.md) |
| Understand documentation ownership and future rrflowDB migration | [RRFlow knowledge map](docs/README.md) |

## Non-negotiable architecture

This section is the invariant index. The linked owner holds each detailed
definition; this portal does not reproduce those bodies.

| Invariant | Owning section |
|---|---|
| Canonical names for RRFlow, RRD, `RrdEngine`, rrflowDB, rrflowKV, rrflowMX, rrflowQL, Arrow/DataFusion, vectors, inference, LFG, and Connectome | [Canonical component terminology](docs/architecture/system-overview.md#canonical-component-terminology) |
| One project ↔ one estate/rrflowDB ↔ one RRD instance; deployment form, storage profile, endpoint presentation, security, and physical placement remain distinct | [Locked alpha topology](docs/architecture/instance-topology.md#locked-alpha-topology) and [deployment profiles](docs/reference/deployment/modes.md#independent-profile-coordinates) |
| One semantic, security, transaction, mutation, and context authority | [ADR-0001 decision](docs/decisions/0001-single-engine-authority.md#decision) |
| No legacy product line: one current pre-release implementation with requirement-to-code/test traceability before direct convergence | [Pre-release convergence boundary](docs/architecture/system-overview.md#pre-release-convergence-boundary) and [implementation-requirements traceability](docs/roadmap/rrflow-1.0-execution-map.md#implementation-requirements-traceability) |
| Persistent rrflowDB versus volatile rrflowMX | [Persistence and memory boundary](docs/architecture/system-overview.md#persistence-and-memory-boundary) |
| Temporal graph, scalar, BM25, and vector data under one transaction model | [Native multi-model boundary](docs/architecture/system-overview.md#native-multi-model-boundary) |
| rrflowQL fast and analytical paths over stamped Arrow/DataFusion work | [Query and analytical boundary](docs/architecture/system-overview.md#query-and-analytical-boundary) |
| Authentication, authorization, read stamps, compute, mutation, and evidence | [Security boundary](docs/architecture/system-overview.md#security-boundary) |
| Repository-contained source and an offline-verifiable default distribution | [Source and distribution boundary](docs/architecture/system-overview.md#source-and-distribution-boundary) |
| Exact semantic write and durable rrflowKV commit sequence | [Write and commit flow](docs/architecture/engine-data-flow.md#write-and-commit-flow) |
| Hybrid immutable storage and measured conditional zero-copy | [rrflowKV physical target](docs/architecture/engine-data-flow.md#rrflowkv-physical-target) and [conditional zero-copy](docs/architecture/engine-data-flow.md#conditional-zero-copy) |
| One bounded, deterministic, provider-neutral context operation | [Context assembly contract](docs/architecture/engine-data-flow.md#context-assembly-contract) |
| Project discovery begins with one deterministic committed tree snapshot | [Project-tree inventory and incremental attunement](docs/architecture/engine-data-flow.md#project-tree-inventory-and-incremental-attunement) |
| External databases, meshes, providers, and clients remain explicit adapters | [External integrations](docs/architecture/system-overview.md#external-integrations) |

## Product operation map

| Operation or surface | Owning record or gate |
|---|---|
| New/existing-project installation, specialization, and attunement | [Provider-neutral agent bootstrap](docs/reference/agent-bootstrap.md#installation-and-attunement) and [roadmap Gate D](docs/roadmap/rrflow-1.0.md#gate-d--install-configure-and-attune-one-real-estate) |
| Deterministic project-tree snapshot and incremental refresh | [Project-tree inventory flow](docs/architecture/engine-data-flow.md#project-tree-inventory-and-incremental-attunement) and [roadmap D-03/D-04](docs/roadmap/rrflow-1.0.md#gate-d--install-configure-and-attune-one-real-estate) |
| Governed in-engine functions and proposed-transaction bindings | [RRFlow governed functions](docs/reference/automation/functions.md) and [roadmap A-07/C/I](docs/roadmap/rrflow-1.0.md#executable-dependency-spine) |
| Discovered project commands, installed capability bindings, and external activities | [Project command capability contract](docs/reference/automation/project-command-capabilities.md), [roadmap D-06](docs/roadmap/rrflow-1.0.md#gate-d--install-configure-and-attune-one-real-estate), and [roadmap I-03](docs/roadmap/rrflow-1.0.md#gate-i--add-explicit-automation-scaffolding-without-automatic-hooks) |
| Durable provider-neutral self and `rrflow://` record resolution | [Seat identity and memory warps](docs/reference/seat-identity.md) |
| Estate desired/observed state and fenced external effects | [RRFlow estate control](docs/reference/operations/estate-control.md) |
| Embedded, single-node, cluster, rrflowMX, rrflowKV, loopback, and network combinations | [RRFlow deployment profiles](docs/reference/deployment/modes.md) |
| Kubernetes projection, reconciliation, readiness, deletion, and qualification | [RRFlow Kubernetes deployment adapter](docs/reference/deployment/kubernetes-operator.md) |
| Distributed consistency, replication, recovery, and qualification | [RRFlow distributed cluster contract](docs/reference/distributed/cluster-contract.md) |
| Local RRD launch, authenticated readiness, and bounded shutdown | [RRFlow local process adapter](docs/reference/deployment/local-process-driver.md) |
| Rust and generated language clients over the one public operation authority | [RRFlow SDK reference](docs/reference/sdk/README.md) and [roadmap H-04](docs/roadmap/rrflow-1.0.md#gate-h--prove-context-flow-feedback-live-delivery-and-connectome) |
| Connectome connection, projections, interactions, and conformance | [RRFlow Connectome client contract](docs/reference/client/connectome.md), [client bootstrap boundary](docs/architecture/system-overview.md#client-bootstrap-boundary), and [roadmap H-06/H-07](docs/roadmap/rrflow-1.0.md#gate-h--prove-context-flow-feedback-live-delivery-and-connectome) |
| Engine events, triggers, routines, skills, and host-event adapters | [Automation, routine, and skill flow](docs/architecture/engine-data-flow.md#automation-routine-and-skill-flow), [generic routine packages](docs/reference/agent-bootstrap.md#generic-routine-package), and [roadmap Gate I](docs/roadmap/rrflow-1.0.md#gate-i--add-explicit-automation-scaffolding-without-automatic-hooks) |
| Current rrflowKV bytes and removable format readers | [rrflowKV current physical format](docs/reference/storage/rrflowkv-current-format.md) |
| Canonical target source tree and dependency direction | [Frozen target source tree](docs/roadmap/rrflow-1.0-execution-map.md#frozen-target-source-tree) |
| Repository verification sequence | [Repository-wide run checklist](docs/roadmap/rrflow-1.0-execution-map.md#repository-wide-run-checklist) and [CI execution contract](docs/operations/ci.md) |
| Repository work instructions | [`AGENTS.md`](AGENTS.md) |

## Current status

The persistent rrflowDB target is not complete or alpha-qualified. Existing
types, crates, tests, rrflowKV persistence, rrflowMX execution, rrflowQL, and
DataFusion integration are implementation inventory—not proof that the target
engine flow is finished. The exact present-versus-target boundary is maintained
in [Current implementation boundary](docs/architecture/engine-data-flow.md#current-implementation-boundary),
and all observed deficiencies are maintained in the
[POA&M](docs/poam/rrflow-1.0-alpha.md#open-deficiencies).

The canonical checklist and accepted evidence are in the
[release roadmap](docs/roadmap/rrflow-1.0.md#rrflow-10-execution-checklist).
The next executable package is
[A-06 / KB-05](docs/roadmap/rrflow-1.0.md#a-06-knowledge-bootstrap-sequence):
resolve the remaining flat supporting records one complete file at a time.
B-01 and B-02 remain completed contract work; later Gate B work remains paused
until A-06 and A-07 are complete.

## Knowledge warp points

The checkout is RRFlow's bootstrap memory until authorized import and durable
rrflowDB readback pass. Each row provides one stable future data coordinate and
one repository fallback; the linked record owns the content.

| Memory | Durable warp | Checkout fallback |
|---|---|---|
| Knowledge structure and ownership | [`rrflow://rrflow-instance/data/documentation-index/rrflow-knowledge-map`](rrflow://rrflow-instance/data/documentation-index/rrflow-knowledge-map) | [`docs/README.md`](docs/README.md) |
| Master system overview | [`rrflow://rrflow-instance/data/architecture/system-overview`](rrflow://rrflow-instance/data/architecture/system-overview) | [`docs/architecture/system-overview.md`](docs/architecture/system-overview.md) |
| Project, estate, instance, and deployment topology | [`rrflow://rrflow-instance/data/architecture/instance-topology`](rrflow://rrflow-instance/data/architecture/instance-topology) | [`docs/architecture/instance-topology.md`](docs/architecture/instance-topology.md) |
| Deployment profiles and cross-profile conformance | [`rrflow://rrflow-instance/data/reference/deployment/modes`](rrflow://rrflow-instance/data/reference/deployment/modes) | [`docs/reference/deployment/modes.md`](docs/reference/deployment/modes.md) |
| Kubernetes deployment adapter | [`rrflow://rrflow-instance/data/reference/deployment/kubernetes-operator`](rrflow://rrflow-instance/data/reference/deployment/kubernetes-operator) | [`docs/reference/deployment/kubernetes-operator.md`](docs/reference/deployment/kubernetes-operator.md) |
| Distributed cluster contract | [`rrflow://rrflow-instance/data/reference/distributed/cluster-contract`](rrflow://rrflow-instance/data/reference/distributed/cluster-contract) | [`docs/reference/distributed/cluster-contract.md`](docs/reference/distributed/cluster-contract.md) |
| Single-engine authority decision | [`rrflow://rrflow-instance/data/decision/0001-single-engine-authority`](rrflow://rrflow-instance/data/decision/0001-single-engine-authority) | [`docs/decisions/0001-single-engine-authority.md`](docs/decisions/0001-single-engine-authority.md) |
| Engine data flow | [`rrflow://rrflow-instance/data/architecture/engine-data-flow`](rrflow://rrflow-instance/data/architecture/engine-data-flow) | [`docs/architecture/engine-data-flow.md`](docs/architecture/engine-data-flow.md) |
| rrflowKV current physical format | [`rrflow://rrflow-instance/data/reference/storage/rrflowkv-current-format`](rrflow://rrflow-instance/data/reference/storage/rrflowkv-current-format) | [`docs/reference/storage/rrflowkv-current-format.md`](docs/reference/storage/rrflowkv-current-format.md) |
| Seat identity and memory warps | [`rrflow://rrflow-instance/data/reference/seat-identity`](rrflow://rrflow-instance/data/reference/seat-identity) | [`docs/reference/seat-identity.md`](docs/reference/seat-identity.md) |
| Provider-neutral agent bootstrap | [`rrflow://rrflow-instance/data/reference/agent-bootstrap`](rrflow://rrflow-instance/data/reference/agent-bootstrap) | [`docs/reference/agent-bootstrap.md`](docs/reference/agent-bootstrap.md) |
| Governed functions and transaction bindings | [`rrflow://rrflow-instance/data/reference/automation/functions`](rrflow://rrflow-instance/data/reference/automation/functions) | [`docs/reference/automation/functions.md`](docs/reference/automation/functions.md) |
| Project command capabilities and activities | [`rrflow://rrflow-instance/data/reference/automation/project-command-capabilities`](rrflow://rrflow-instance/data/reference/automation/project-command-capabilities) | [`docs/reference/automation/project-command-capabilities.md`](docs/reference/automation/project-command-capabilities.md) |
| SDK references | [`rrflow://rrflow-instance/data/reference-index/sdk`](rrflow://rrflow-instance/data/reference-index/sdk) | [`docs/reference/sdk/README.md`](docs/reference/sdk/README.md) |
| Connectome client contract | [`rrflow://rrflow-instance/data/reference/client/connectome`](rrflow://rrflow-instance/data/reference/client/connectome) | [`docs/reference/client/connectome.md`](docs/reference/client/connectome.md) |
| Product version policy | [`rrflow://rrflow-instance/data/reference/release/version-policy`](rrflow://rrflow-instance/data/reference/release/version-policy) | [`docs/reference/release/version-policy.md`](docs/reference/release/version-policy.md) |
| Alpha objectives | [`rrflow://rrflow-instance/data/objective/rrflow-1.0-alpha`](rrflow://rrflow-instance/data/objective/rrflow-1.0-alpha) | [`docs/objectives/rrflow-1.0-alpha.md`](docs/objectives/rrflow-1.0-alpha.md) |
| Release roadmap | [`rrflow://rrflow-instance/data/roadmap/rrflow-1.0`](rrflow://rrflow-instance/data/roadmap/rrflow-1.0) | [`docs/roadmap/rrflow-1.0.md`](docs/roadmap/rrflow-1.0.md) |
| Alpha POA&M | [`rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha`](rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha) | [`docs/poam/rrflow-1.0-alpha.md`](docs/poam/rrflow-1.0-alpha.md) |
| Code execution map | [`rrflow://rrflow-instance/data/execution-map/rrflow-1.0`](rrflow://rrflow-instance/data/execution-map/rrflow-1.0) | [`docs/roadmap/rrflow-1.0-execution-map.md`](docs/roadmap/rrflow-1.0-execution-map.md) |

The [knowledge map](docs/README.md#record-header-and-indexing-pattern) defines
how these reviewed Markdown owners become deterministic JSON/JSONL import
records and, after the persistence gates pass, rrflowDB-resolved memory without
creating a second editable source of truth.

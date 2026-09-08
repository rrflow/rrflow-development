# RRFlow instance topology

**Status:** active accepted logical and physical topology; implementation convergence is incomplete
**Coordinate:** `rrflow://rrflow-instance/data/architecture/instance-topology`
**Owner:** project, estate, RRD instance, environment, deployment, and physical-placement relationships
**Decision:** [`../decisions/0001-single-engine-authority.md`](../decisions/0001-single-engine-authority.md)

The repository root [README](../../README.md) owns product identity and current
status. The [system overview](system-overview.md) owns component meaning, the
[engine data-flow record](engine-data-flow.md) owns execution sequence, the
[estate-control reference](../reference/operations/estate-control.md) owns
desired/observed operational semantics, the
[deployment-profile reference](../reference/deployment/modes.md) owns valid
deployment-form, storage-profile, and endpoint-presentation combinations plus
their conformance boundaries, the
[local-process adapter](../reference/deployment/local-process-driver.md) owns
host-local launch/readiness/shutdown effects, and the
[roadmap](../roadmap/rrflow-1.0.md) owns implementation order and evidence. This
record owns only the topology that those components inhabit.

## Locked alpha topology

RRFlow 1.0 installs one independent AI estate for one project. One logical RRD
instance serves exactly that estate through one `RrdEngine` authority. Embedded,
single-node server, and clustered execution are deployment forms of that same
instance; rrflowMX and rrflowKV are storage profiles beneath the same semantics.

```text
project ──governed-by──> estate (rrflowDB)
                              ^
                              │ serves exactly one
                         RRD instance
                              │
                    one logical RrdEngine authority
                              │
              ┌───────────────┬───────────────┐
              │               │               │
       deployment form   storage profile  endpoint presentation
    embedded/server/cluster  MX or KV     in-process/HTTP/WebSocket
```

The relationships are deliberately not a single generic parent chain. A
project is the governed external work, an estate is its RRFlow semantic state,
an instance is the installed runtime identity, an environment is observed
project context, and a cluster is physical execution. Conflating them would
make paths, policy, storage, and multi-project aggregation ambiguous.

## Canonical identities and cardinality

| Identity or term | Meaning | Required relationship | Not allowed to mean |
|---|---|---|---|
| **project** | One external software/product repository or explicitly bounded source estate whose development RRFlow governs. | Exactly one project identity is bound by a first-alpha installation. | A filesystem path, Cargo workspace, tenant, database, or operator fleet. |
| **estate** | The complete project-scoped RRFlow semantic boundary: governed knowledge, temporal graph, memories, reasoning state, indexes, evidence, automation, and operations. It is the logical rrflowDB. | Exactly one estate governs exactly one project in the first alpha. | A fleet of projects, a deployment controller, cluster, storage directory, or JSON aggregate. |
| **RRD instance** | The stable identity of one installed embedded or daemon runtime serving one estate. | Exactly one instance serves one estate; all public faces resolve to its one logical `RrdEngine`. | A process ID, host, endpoint, project path, node, database namespace, or client session. |
| **environment** | An explicit named project context such as development, test, staging, production, device, or region, captured with source and observation evidence. | Zero or more environment descriptors belong to the project estate; every environment-scoped fact names one explicitly. | A required ancestor of the estate/instance, an inferred directory name, or a hidden selector for another database. |
| **workspace** | A derived build/source topology inside the project, including packages, modules, generated boundaries, and harnesses discovered by attunement. | A committed project-tree snapshot owns its observed root-relative membership. | A project, estate, deployment, authorization scope, storage profile, or tenancy boundary. |
| **organization/account/entitlement** | Optional external operator, billing, or policy-provider references. | They may be linked through an explicitly configured adapter and policy. | A mandatory local RRFlow parent hierarchy or an authority that can bypass `RrdEngine`. |
| **tenant** | An optional security or data-partition subject whose semantics must be defined by a later schema/policy contract. | If used, policy and every affected operation name the exact tenant scope. | An implicit container between the instance and estate or a synonym for project. |
| **namespace/database** | Reserved logical catalogue concepts only if a later accepted schema contract proves a need for them. | No first-alpha topology path requires either term; rrflowDB is the project estate. | Mandatory SurrealDB-style containers, deployment identities, or additional persistence authorities. |

Stable IDs are canonical values independent of display names, paths, endpoints,
processes, and provider identifiers. Human labels can change without changing
identity. A physical locator can be evidence in an installed binding, but it
cannot become the resource ID.

Multi-project views do not weaken these cardinalities. Connectome or another
operator can aggregate several independently authenticated RRD instances as a
client. No local estate becomes a fleet database, and no instance silently
opens another project's state.

## Logical estate model

The logical relationship graph is:

```text
project ──governed-by──> estate
estate  ──served-by────> RRD instance
estate  ──contains─────> canonical records, relations, schemas, memories,
                          vectors, reasoning trees, events, routines, skills,
                          checkpoints, evidence, audit, outbox, and indexes
project ──observed-as──> project-tree snapshots and environment descriptors
```

Tables, collections, records, relations, aliases, vector points, lexical
postings, and graph adjacency are semantic or derived data families inside the
estate. They do not introduce another database engine. Schema and policy decide
which families exist and how they are authorized; topology never chooses an
alternate graph, vector, search, or analytical authority.

Every canonical fact and every derived projection is estate-scoped. A derived
index additionally binds its source cursor and relevant schema, model, and
configuration revisions. Cross-estate reads are explicit client-side or
authorized federation operations; they are never a local prefix accident or a
claim of cross-instance ACID.

## Deployment and physical placement

Deployment form, storage profile, and endpoint presentation are orthogonal.
The [deployment-profile reference](../reference/deployment/modes.md) owns their
valid combinations, default alpha posture, runtime discovery, and layered
conformance:

| Dimension | Values | Invariant |
|---|---|---|
| Deployment form | embedded, single-node server, clustered server | The same instance, estate semantics, operation catalogue, security decisions, and result contracts remain visible. |
| Storage profile | rrflowMX, rrflowDB backed by rrflowKV | Non-durability-specific behavior is equivalent. Only rrflowKV can claim crash/reopen, backup, recovery, or replicated durability. |
| Endpoint presentation | in-process, loopback HTTP/WebSocket, or configured network HTTP/WebSocket | Reachability does not establish identity or authorization. An optional mesh adapter may resolve or carry a configured network endpoint but cannot create another presentation or authority. |

rrflowMX is not the hot tier of rrflowDB and does not automatically flush into
rrflowKV. rrflowKV's memtable and cache are the hot tiers of a persistent
rrflowDB. Moving accepted data from rrflowMX to rrflowKV is an explicit,
authorized semantic transaction, not a startup side effect.

The physical clustered relationship graph is:

```text
RRD instance ──deployed-as──> cluster
cluster      ──contains────> node
instance     ──partitions──> shard
shard        ──has-copy────> replica ──placed-on──> node
replica      ──materializes> rrflowKV segment/object artifacts
```

A cluster is one instance's consensus and placement domain. It does not own
several project estates. A node is a physical process/host member, a shard is a
routing and consensus partition of that instance's state, a replica is one
copy placed on a node, and a segment is an internal rrflowKV or index artifact.
Changing placement, leadership, replica health, or segment generation cannot
change project, estate, or instance identity.

The current cluster contracts contain useful placement epochs, quorum,
snapshot-vector, routing, artifact-transfer, and reshard safety checks. They are
implementation inventory until the cluster package proves that those checks
operate on this topology through the canonical engine and storage contracts.

## Installation and binding resolution

Only the previewed and explicitly applied `rrflow install` operation may create
or change an installed topology. Installation has two distinct layers:

1. `.rrflow/config.toml` is a minimal project-local locator generated from a
   bundle-resident versioned template. It provides only enough information to
   find the installed instance/profile and a credential reference. It contains
   no canonical mutable state, plaintext secret, inferred provider hook, or
   second project model.
2. A canonical installed-estate binding is committed through `RrdEngine`. It
   binds the stable instance, estate, and project identities to the selected
   deployment form, storage profile, endpoint presentations, admitted project
   root, installed
   configuration/template/attunement revisions, and security authority. Host
   paths and endpoints are locators/evidence, never identities or authorization.

The exact serialized fields remain owned by D-01. The semantic requirements do
not: a valid binding must make the following checks possible before ordinary
state is read or changed.

| Required check | Failure behavior |
|---|---|
| Locator format/profile is supported and its referenced installation exists. | Fail closed; startup must not create or repair it. |
| Instance, estate, and project identities match the canonical binding. | Fail closed without opening another scope. |
| The requested project root is the exact admitted root; nested, neighboring, symlink-escaped, mount-escaped, and foreign stores are rejected. | Fail before project content or rrflowDB state is mutated. |
| Configuration, template, attunement profile, and security revisions match their committed digests. | Require an explicit preview/apply reconfiguration operation. |
| Credential reference resolves and the authenticated principal is authorized for the requested operation/resource. | Deny without falling back to anonymous or provider identity. |
| Selected storage profile satisfies the requested durability capability. | Return an explicit unsupported-capability result; never silently switch profiles. |

Normal `rrd-server`, embedded, CLI, MCP, SDK, and Connectome startup resolves
this installed binding read-only and then opens the one engine. No client or
daemon startup path may call an `ensure`, `initialize`, `load-or-create`, or
directory-derived identity function to manufacture installation authority.

Project relocation, data-root relocation, profile changes, and endpoint changes
are explicit previewed operations with a new plan digest. They validate source
and target containment, preserve stable semantic identities where authorized,
commit the new binding and evidence atomically, and leave the old locator
unusable. RRFlow 1.0 has no successful earlier manifest reader, implicit
migration, or read-old/write-new branch.

## Routing and authorization

Ingress resolves in this order:

```text
configured endpoint
  -> authenticated RRD instance identity and capability handshake
    -> installed estate/project binding
      -> typed public operation and operation-specific resource coordinate
        -> policy decision at the operation's read/transaction stamp
          -> RrdEngine execution
```

Network locality, a mesh peer, a project directory, an organization account,
or possession of a data path never grants access. HTTP, WebSocket, native SDK,
MCP, CLI, and Connectome must present the same logical operation and receive
the same authorization result, stamp, digest, and evidence.

A generic resource path validator that merely rejects empty or repeated kinds
is insufficient. A-07 must freeze operation-specific path grammars and parent
relationships. The public contract, security policy, estate state, installed
binding, and cluster placement must use one spelling and identity family; a
historical vocabulary table cannot constrain active public types.

## External systems and adapters

PostgreSQL, Turso, SQLite, Dragonfly, object stores, generators, test harnesses,
CI systems, deployment controllers, model providers, Wardenclyffe/Zuul Zero or
other meshes, and Connectome are optional external capabilities or clients.
Attunement can discover a typed inactive candidate. Activation requires an
exact previewed adapter binding, configuration, policy, authorization, limits,
and credential reference.

An adapter may contribute source data, evidence, an explicitly authorized
effect, endpoint resolution, or UI. It cannot become rrflowDB persistence,
rrflowQL planning, index authority, project identity, installation authority,
or a second reasoning/event/routine lifecycle. Generated or observed output is
re-inventoried and committed through `RrdEngine` before it becomes estate state.

## Current implementation audit

The passing current tests characterize several safety properties; they do not
establish the accepted topology.

| Current construct | Useful behavior to preserve | Conflict to remove directly |
|---|---|---|
| `InstanceManifest`, `InstanceMode::Dedicated`, and `.rrflow/instance.toml` | Strict unknown-field/version rejection, create-new publication, canonical instance IDs, exact-root discovery, and rejection of nested/neighbor projects. | It is a second installer with frozen compatibility fields, lacks estate/project/environment semantics, and lets server/CLI/test startup create authority outside D-01. Replace it with the one locator plus installed binding; retain no reader or alias. |
| `ProjectAuthorityBinding` at `server/state/project-authority/{instance}` | Digest validation, exact canonical root/store checking, and refusal to rebind a foreign store. | It stores absolute paths and manifest shape in a JSON control value, omits accepted installation/security revisions, is rrflowKV-specific, and is committed through a private control transition with fabricated invocation coordinates. Replace it with a semantic installed binding committed through the normal transaction. |
| `RrdEngine::open_project_store`, `open_bound`, and `bind_project_authority` | One composition point opens storage and checks binding before use. | Opening is coupled to `.rrflow/rrd`, normal startup can create token material, and binding is not proven equivalent across MX/KV or authorized through the installed operation. |
| `rrd-server initialize` and CLI/MCP/test `ensure_dedicated*` callers | Provide repeatable current test setup. | They are independent authority-creation paths. Test fixtures must invoke the real install plan/apply boundary or a test-only pure fixture builder over the same contract. The release command is removed. |
| `PLATFORM_TERMS`, `ResourceKind`, and `ResourcePath` | Bounded typed resource components and duplicate-kind rejection. | The active table mirrors a superseded historical document; environment is omitted; organization/estate/project/instance roles conflict; and arbitrary kind order is accepted. Replace with operation-specific canonical topology/resource contracts. |
| `EstateAuthorityResourceKind` and `rrd-estate::AuthorityResourceKind` | Bounded IDs, desired/observed validation, parent existence, receipt lineage, and some strict relationships. | The organization/account/project/environment/instance/node/shard/job/health/secret hierarchy duplicates installation, security, estate, job, health, and cluster owners inside a monolithic estate document. Split every useful semantic into its one owner and remove the catalogue. |
| `rrd-cluster` identity and placement contracts | Multi-zone voter checks, placement epochs, read stamps, snapshot-vector consistency, explicit cross-shard denial, transfer digests, and reshard cutover checks. | Cluster IDs use a separate loose string family and free-form scope strings; the contract is not yet bound to installed instance/estate identity or one engine transaction. Converge it at the cluster/deployment gates without making cluster an estate owner. |

The historical platform-vocabulary note remains provenance only. An active
test that requires `PLATFORM_TERMS` to match its word order is itself a tracked
defect, not proof of canonical terminology.

## Direct-convergence sequence

1. **A-07 vocabulary and traceability:** freeze the identities, relationship
   graph, operation-specific resource paths, module/type names, and exact
   preserve/remove map across contract, engine, estate, server, CLI, SDK/MCP,
   and cluster callers.
2. **C-02/C-03 storage semantics:** represent the installed binding, estate,
   topology relations, audit, outbox, and runtime log through the same
   authorized semantic transaction on rrflowMX and rrflowKV. Remove private
   JSON control authority rather than wrapping it.
3. **D-01 install:** introduce the bundle-resident `.rrflow/config.toml`
   template and canonical installed-estate binding; make preview side-effect
   free and apply the sole creator. Remove `.rrflow/instance.toml`,
   `InstanceManifest`, `rrd-server initialize`, and every `ensure_dedicated*`
   success path in the same convergence package.
4. **D-02/D-03 attunement:** persist jobs and commit the bounded project tree
   using the installed root and stable project/estate identities. Environments,
   workspaces, and external systems enter only as evidenced project data or
   inactive capability candidates.
5. **E/F query and indexes:** make graph, scalar, BM25, vector, and analytical
   reads estate-scoped at one stamp; Arrow/DataFusion computes over selected
   data and never changes topology or binding.
6. **H/I clients and automation:** expose one topology through every surface;
   routines and adapters resolve the installed engine and cannot create
   instances or cross estates.
7. **J release proof:** reject every superseded manifest/path/shape, install an
   empty and existing project from the signed offline bundle, restart, and show
   identical authenticated identity plus durable query/reasoning evidence.

## Characterization evidence and missing proof

At the review baseline, these focused tests pass:

- `rrd-engine/tests/runtime_instance.rs`: seven tests for manifest shape,
  containment, foreign stores, binding persistence, and relocation failure;
- `rrd-contract/tests/platform_terminology.rs`: three tests freezing the
  current term table and its historical-file order;
- `rrd-estate/tests/authority_catalogue.rs`: two tests for the duplicate
  authority catalogue and selected rrflowKV reopen behavior; and
- `rrd-cluster/tests/contracts.rs`: ten tests for placement, stamps, routing,
  transfer, telemetry, and reshard contract validation.

They do not prove real installation, one identity vocabulary, rrflowMX/rrflowKV
semantic parity, atomic topology/index/audit commits, cross-surface resolution,
clustered engine operation, relocation/reconfiguration, or offline deployment.
Those remain open in the roadmap and POA&M.

## Acceptance

The topology is implemented only when a clean bundle performs a no-write
preview and one explicit install for both an empty and existing project; every
surface resolves the same stable project, estate, and instance through one
authenticated `RrdEngine`; rrflowMX and rrflowKV pass the complete
storage-profile semantic differential; rrflowKV survives reopen;
environment/workspace/external capability
records cannot alter authority; physical placement cannot alter logical
identity; unauthorized, nested, neighboring, escaped, foreign, moved, and
superseded bindings fail closed; and no `.rrflow/instance.toml`, initializer,
generic historical path hierarchy, private binding store, or alternate reader
remains.

# RRFlow deployment profiles

**Status:** active accepted deployment classification; implementation convergence is incomplete
**Coordinate:** `rrflow://rrflow-instance/data/reference/deployment/modes`
**Owner:** deployment-form, storage-profile, and endpoint-presentation combinations plus their conformance boundaries

RRFlow is one independently deployable engine. A deployment profile describes
how one installed RRD instance is composed and reached; it never selects a
different transaction coordinator, graph engine, vector database, query
planner, reasoning authority, or persistence product.

The [instance-topology architecture](../../architecture/instance-topology.md)
owns project, estate, instance, environment, and physical-placement identity.
The [engine data-flow architecture](../../architecture/engine-data-flow.md)
owns the common write, read, Arrow/DataFusion, reasoning, and recall paths. The
[local-process adapter](local-process-driver.md) owns host process effects, the
[server reference](../protocol/server.md) owns HTTP/WebSocket behavior, and the
[roadmap](../../roadmap/rrflow-1.0.md) owns implementation order and completion
evidence. This record owns only the profile classification and the proof needed
to claim semantic equivalence across its independent axes.

## Independent profile coordinates

The installed profile is the product of three orthogonal coordinates:

| Coordinate | Accepted values | What it selects | What it cannot select |
|---|---|---|---|
| Deployment form | `embedded`, `single_node_server`, `clustered_server` | Whether the same logical `RrdEngine` is called in-process, served by one RRD process, or served by a coordinated cluster. | Storage durability, endpoint security, a different operation catalogue, or another engine. |
| Storage profile | `rrflow_mx`, `rrflow_kv` | The volatile rrflowMX implementation or the durable rrflowKV implementation beneath the same storage and semantic transaction contracts. | Process placement, network reachability, query semantics, or an automatic volatile-to-durable migration. |
| Endpoint presentation | `in_process`, `loopback_http_websocket`, `network_http_websocket` | How an authenticated client invokes the installed instance. A server may expose more than one configured presentation. | Instance identity, authorization, durability, or mesh membership. |

Security posture is bound separately. Cleartext is restricted to an explicitly
configured loopback endpoint. A configured network endpoint requires the
accepted authenticated encrypted transport. TLS, mutual TLS, certificate
rotation, credential providers, and mesh carriage are security or adapter
capabilities—not deployment modes.

Artifact packaging is also separate. The current `rrflow-edge` mmap artifact,
browser bundles, mobile packages, SDKs, Connectome, and model packages are
clients or derived artifacts. None is a fourth deployment form and none owns
canonical estate state.

## Valid combinations and alpha posture

| Deployment form | Storage profile | Endpoint presentation | Target posture |
|---|---|---|---|
| `embedded` | `rrflow_mx` | `in_process` | Valid volatile composition. It must pass all semantics that do not require durability. |
| `embedded` | `rrflow_kv` | `in_process` | Valid durable composition. The embedding process exclusively owns the rrflowKV writer. |
| `single_node_server` | `rrflow_mx` | loopback or configured network HTTP/WebSocket | Valid volatile server composition. Durability-only operations fail before preparing authoritative state. |
| `single_node_server` | `rrflow_kv` | loopback or configured network HTTP/WebSocket | First-alpha persistent service target and Connectome bootstrap target. The server exclusively owns the rrflowKV writer. |
| `clustered_server` | `rrflow_kv` | configured network HTTP/WebSocket | Planned and unavailable until the distributed contract, consensus, independent-host, failure, and placement gates pass. |
| `clustered_server` | `rrflow_mx` | any | Invalid: process-local volatile state cannot be the canonical replicated cluster substrate. |

The default first-alpha installation is one authenticated single-node RRD
server over rrflowKV with an explicit loopback HTTP/WebSocket endpoint. A
network or mesh-resolved endpoint is an explicit configuration addition. The
default is a release target, not a shortcut: embedded and rrflowMX compositions
must still use the same semantic contracts, and unsupported combinations must
fail explicitly rather than silently switching profiles.

rrflowMX is not a write buffer, cache, or automatic hot tier for rrflowDB.
rrflowKV's WAL, memtable, immutable pages, and bounded cache form the
hot-to-durable path of rrflowDB. Moving accepted semantic state between
rrflowMX and rrflowKV requires an explicit authorized operation with ordinary
validation,
stamps, evidence, and commit semantics.

## Installation and runtime discovery

Gate D-01 selects the complete profile during preview/apply and commits it in
the installed project-estate-instance binding. Startup resolves that binding
read-only. The composition root supplies the resolved descriptor to
`RrdEngine` and outward transports; no component may reconstruct it from
incidental facts.

In particular:

- a persistent root does not prove the deployment form;
- an absent root does not prove that the caller is embedded;
- TLS enabled or disabled does not distinguish a local from a remote engine;
- the client's network location cannot rename the server's profile;
- a loopback address is not installation or authentication authority; and
- compiled enum variants cannot advertise unavailable cluster or artifact
  behavior.

The target public capability descriptor reports these facts independently:

```text
installed instance/project/estate identity
deployment form
storage profile
active endpoint presentations
security/configuration/capability revisions and digests
available operation and resource-limit capabilities
```

The exact Rust type split and wire encoding are frozen during A-07 after the
complete naming and generated-surface inventory. The semantic rule is already
fixed: the existing scalar `deployment_mode` field must not survive as an
ambiguous authority, and unavailable combinations must not be emitted as
active capability values.

## One engine flow in every profile

Every valid combination enters the same authority and produces the same
non-durability-specific result:

```text
configured endpoint or in-process call
  -> authenticate instance and principal
  -> resolve installed project/estate binding
  -> RrdEngine authorize + capture stamp
  -> engine-selected native fast path or stamped rrflowQL plan
     -> rrflowMX or rrflowKV transaction/access primitives
     -> graph/scalar/BM25/vector operators when selected
     -> bounded Arrow/DataFusion analytical execution when selected
  -> same result, denial, stamp, digest, evidence, and commit impact
```

DataFusion remains compute-only in every form. It consumes stamped bounded
Arrow batches from the selected storage profile and returns results or a
proposal to `RrdEngine`; it never writes rrflowKV, mutates rrflowMX, commits an
index, or changes deployment state directly.

## Conformance is layered, not a label

A single two-document lookup cannot qualify these profiles. The completed
conformance suite is separated by responsibility so failures identify the
broken boundary.

### Storage-profile semantic differential

Run the same engine-owned corpus unchanged on rrflowMX and rrflowKV. Compare
exact results, denials, read/commit stamps, semantic digests, commit impacts,
and bounded resource counters for:

- authenticated allow and deny decisions, row/field filtering, and audit;
- transaction read-your-writes, rollback, conflicts, idempotent replay, and
  the declared snapshot-isolation behavior;
- current and bitemporal records, both graph adjacency directions, and bounded
  traversal;
- scalar and unique indexes, incremental BM25, exact vectors, filtered HNSW or
  quantized candidates, exact reranking, and deterministic RRF;
- direct fast reads and stamped rrflowQL execution over streamed Arrow batches
  through DataFusion with the same logical result;
- persisted reasoning-tree, context, evidence, feedback, event, routine, and
  attunement operations where those gates apply; and
- commit-impact changefeed/subscription behavior.

Durability assertions are not forced onto rrflowMX. rrflowKV alone must pass
WAL and manifest fault injection, crash/reopen, writer exclusion, backup,
recovery, compaction, mapped-buffer lifetime, ENOSPC, and acknowledged-commit
tests.

### Deployment-form differential

Execute the same public semantic operations through an embedded composition
and a real standalone single-node process. Compare instance/estate identity,
authorization, result, stamp, digest, cancellation, limits, and traces. The
standalone process must be installed through the accepted D-01 boundary and
must prove authenticated readiness and bounded shutdown; a raw root, marker
file, or in-test server object does not qualify it.

Clustered-server conformance is a separate expansion of this corpus. It must
add independent-host quorum, replication, leader/follower failure, partition,
placement, snapshot-vector, reshard, and recovery evidence without weakening
the single-node semantics.

### Endpoint-presentation differential

Run equivalent operations over in-process, loopback HTTP/WebSocket, and
configured authenticated network HTTP/WebSocket presentations. Compare the
same typed operation, authorization decision, result/denial, stamp, digest,
deadline/cancellation behavior, backpressure, and correlated causal trace.
TLS and mesh tests qualify their security and carriage properties; they do not
create a `remote` database kind.

### Derived-artifact proof

Test `rrflow-edge` and other offline artifacts against their own immutable
source-cursor, checksum, bounded-read, and exact-result contracts. This proof
must not be counted as rrflowMX/rrflowKV, transaction, graph, DataFusion,
reasoning, or deployment-form conformance unless the artifact actually enters
those ordinary engine paths.

## Current implementation audit

The current code passes useful characterization tests, but its classification
is structurally wrong and its corpus is far below the preceding acceptance
boundary.

| Current implementation | Characterized behavior worth retaining | Required direct convergence |
|---|---|---|
| `rrd_contract::DeploymentMode::{RrflowMx, Embedded, LocalDaemon, Edge, Remote, Distributed}` and `ServiceCapabilities::deployment_mode` | Closed decoding, deterministic capability serialization, and generated-client projection. | Split the storage, deployment-form, and endpoint facts; remove artifact/client-relative/speculative values from the active profile field; regenerate every SDK from the one contract with no alias. |
| `RrdEngine::deployment_mode()` | Deterministically reports `RrflowMx` without a persistent root and `Embedded` with one. | Remove inference from storage-root presence. The installed composition root supplies the explicit profile descriptor. |
| `rrd-server::http::capabilities` | Reports a deterministic current capability response. | Remove `Remote` versus `LocalDaemon` inference from the TLS flag. Project the installed deployment form, selected storage profile, active endpoints, and security capability independently. |
| `rrd-deployment-conformance-v1.json` and contract validation | Strictly validates a versioned fixture with unique non-empty document IDs, bounds, and expected output membership. | Replace the overclaimed shared-deployment meaning with the layered semantic, form, transport, durability, and artifact corpora above. |
| engine deployment tests | Exercise one schema plus two records on rrflowMX and rrflowKV, run one exact-ID rrflowQL query through the current DataFusion executor, reject a durability operation on rrflowMX, deny a second rrflowKV writer, and reopen at cursor three. | Retain this as a seed characterization only. It does not prove streaming/pushdown, native graph/index/vector access, complete semantic parity, failure atomicity, reasoning, or resource bounds. |
| Rust client loopback and mutual-TLS tests | Exercise real sockets and verify selected TLS identity behavior. | Stop deriving a database mode from TLS. Run the complete cross-surface corpus against an installed real process. |
| standalone server-process test | Starts a real child, observes readiness, exercises the seed corpus, proves writer exclusion, shuts down, and reopens. | Replace raw-root initialization, marker authority, and private process state through D-01 and the accepted local-process adapter before counting it as deployment proof. |
| `rrflow-edge` offline test | Builds, mmap-opens, and queries a deterministic feature-hash artifact from the two fixture documents. | Keep it as derived-artifact evidence. It parses the fixture as untyped JSON and does not enter `RrdEngine`, rrflowQL, transactions, graph/index semantics, or the shared storage profile. |

The current control journal and global runtime cursor are also not two accepted
ordering authorities. Gate C converges authoritative semantic state, index and
projection deltas, audit, outbox, and cursor into one atomic engine-owned
commit. Logs and traces observe that commit and never manufacture it.

## Direct-convergence sequence

1. **A-07:** freeze the three coordinates and their exact public/internal
   vocabulary; split the contract module mechanically; inventory and remove
   every ambiguous current mode and generated projection without a shim.
2. **B-04/H-04:** expose the structured installed descriptor and run one
   operation catalogue across HTTP, WebSocket, SDK, MCP, CLI, and Connectome.
3. **C-02 through C-04:** prove the full semantic differential on rrflowMX and
   rrflowKV, then add rrflowKV-only durability evidence.
4. **D-01:** make preview/apply choose and persist the profile; make all startup
   paths resolve it without inference or implicit initialization.
5. **E/F/G/H/I:** fill the shared corpus with native graph, lexical, vector,
   streamed Arrow/DataFusion, reasoning, context, feedback, delivery, and
   automation behavior as each owning gate lands.
6. **H-07:** add configured mesh endpoint resolution as an optional transport
   adapter, never as identity, authentication, storage, or deployment mode.
7. **J:** reject every superseded value/shape, qualify the complete matrix on
   clean installed artifacts, and publish raw comparative deployment evidence.

## Acceptance

This profile contract is implemented only when installation binds one explicit
valid combination; capability discovery reports its independent coordinates;
no caller infers a mode from root/TLS/network location; rrflowMX and rrflowKV
pass the complete non-durability semantic differential; rrflowKV passes the
durability matrix; embedded and installed single-node forms plus every active
endpoint pass cross-surface equivalence; cluster values remain unavailable
until independent-host proof; derived edge artifacts remain outside deployment
semantics; and no current `DeploymentMode` alias, speculative value, or
two-document fixture is accepted as substitute evidence.

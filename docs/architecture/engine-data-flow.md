# RRFlow engine data flow

**Status:** active accepted target architecture; implementation gaps remain open
**Coordinate:** `rrflow://rrflow-instance/data/architecture/engine-data-flow`
**Owner:** detailed transactional, storage, query, context, and automation flow

The repository root [README](../../README.md) owns RRFlow's identity and
non-negotiable invariants. The [system overview](system-overview.md) owns the
master component and security-boundary map, the
[alpha objective](../objectives/rrflow-1.0-alpha.md) defines the result, the
[roadmap](../roadmap/rrflow-1.0.md) orders delivery, and the
[POA&M](../poam/rrflow-1.0-alpha.md) tracks observed gaps. This record owns the
detailed end-to-end flow only.

## System boundary

RRFlow is one hybrid transactional and analytical reasoning-data engine. The
names describe cooperating layers, not independent products or stores:

| Layer | Responsibility |
|---|---|
| rrflowDB | One persistent AI estate containing governed knowledge, temporal graph state, reasoning state, evidence, and indexes. |
| `RrdEngine` | Sole semantic transaction coordinator: authenticates, authorizes, binds read stamps, validates mutations and model proposals, and invokes the selected storage profile. |
| rrflowKV | Durable physical profile: WAL, sequence allocation, MVCC snapshots, mutable memtable, immutable segments, manifests, flush, compaction, and recovery. |
| rrflowMX | Non-durable physical profile implementing the same semantic storage port for volatile and conformance execution. |
| rrflowQL | Query language and planning boundary selecting native fast operators or the analytical path. |
| Arrow substrate | Shared columnar buffer model used by eligible immutable segment pages and streamed analytical batches. |
| DataFusion | Bounded vectorized execution over stamped Arrow batches; it does not authorize or commit RRFlow state. |
| Native indexes | Scalar/unique, graph adjacency, BM25, exact-vector, HNSW, and related access paths bound to canonical source coordinates. |
| LFG or provider model | Replaceable constrained adapter that proposes recipes, branches, or bounded query intent. |
| HTTP, WebSocket, SDK, MCP, CLI, Connectome | Transport and client surfaces invoking public `RrdEngine` capabilities without recreating engine logic. |

## Write and commit flow

```text
HTTP / WebSocket / SDK / MCP / embedded caller
                         |
                  typed public request
                         |
                         v
                     RrdEngine
       parse -> authenticate -> authorize -> bind schema/policy
                         |
              optional analytical transform
                         |
            rrflowQL/DataFusion returns batches
              or model returns a proposal
                         |
                         v
                     RrdEngine
       validate -> lower semantic mutation -> authorize effects
                         |
                         v
                      rrflowKV
       WAL frame -> sequence/MVCC memtable -> commit receipt
                         |
              flush and compaction builders
                         |
                         v
       immutable segment + index roots + manifest publication
                         |
                         v
             changefeed / live delta / trace
```

`RrdEngine` owns the semantic transaction and authorization decision. rrflowKV
owns WAL concurrency, sequence allocation, physical snapshot isolation, flush,
compaction, and durable publication. `RrdEngine` must not hold a global WAL
lock or expose physical keys to callers.

rrflowQL parses and binds expressions. DataFusion may scan, filter, join,
aggregate, rank, or produce transformed Arrow batches. A DataFusion `DataSink`
or another compute adapter returns a proposed result to `RrdEngine`; it does
not write rrflowKV files, publish manifests, or bypass semantic validation.

One accepted semantic transaction must publish its canonical record changes,
temporal versions, both graph adjacency directions, synchronous index changes,
runtime-log entry, and durable asynchronous index deltas as one rrflowKV write
batch. Index builders may run later, but their source cursor and freshness
state are part of the committed transaction.

### Current runtime-log and graph oracle

The current semantic storage port exposes a bounded, authenticated runtime-log
page containing `requested_after`, `through_cursor`, `head_cursor`, validation
evidence, and matching changes. A consumer always resumes at
`through_cursor`, even when its scope filter matched no changes, because the
cursor records the examined global prefix rather than the number of returned
rows. This behavior is tested on rrflowMX and rrflowKV, including rrflowKV
flush, close, reopen, and continued hash chaining.

`RuntimeGraphSnapshot::from_changes` is the current temporal graph oracle. It
folds transaction-visible record, relation, event, and retirement mutations at
one scope, valid-time instant, and known cursor. Each event has a stable
cursor-derived node identity; a subject-bearing event also produces a
deterministic `emitted` relation from its subject to that event. The snapshot
supports outgoing and incoming relation views, bounded relation-filtered
breadth-first traversal, and an exact structural differential between two
known cursors. rrflowQL additionally exposes deterministic directed traversal
with cycle suppression and the first shortest path to each reached node.

These are semantic reference results, not the target physical graph engine.
The current implementations can reconstruct a snapshot from retained runtime
changes and traverse materialized vectors. Gates C-03 and C-04 must replace
normal whole-log reconstruction with atomically maintained version and
adjacency keys; Gate E-01 must prove bounded native traversal against this
oracle. A structural differential is neither a committed mutation nor a live
query result until its owning `RrdEngine` operation validates and commits the
corresponding proposal.

## rrflowKV physical target

rrflowKV combines write-optimized state with scan-optimized immutable storage
inside one durability authority:

```text
recent writes                         published immutable state
-------------                         -------------------------
checksummed WAL                       manifest generation
mutable MVCC memtable       flush     ordered key/version spine
point/range/CAS access       ----->    Arrow-compatible column pages
                                      adjacency/posting/index roots
                                      encoding and checksum metadata
```

The ordered key/version spine preserves exact point, prefix, range, temporal,
and conflict semantics. Arrow-compatible pages allow selected immutable
columns to enter `RecordBatch` streams without reconstructing generic Rust row
objects. The same manifest pins both forms so they cannot represent different
commits.

Column pages may contain canonical record values. Scalar, lexical, graph, and
approximate-vector indexes remain derived acceleration state and cannot
outrank their source keys, schema, catalogue, or read cursor. Exact vector
values remain canonical; HNSW and quantized structures remain candidate
indexes followed by exact reranking when the query contract requires it.

The mutable memtable does not need to mimic a columnar file. It is optimized
for high-frequency writes, current state, CAS, and bounded range reads. A
stamped analytical scan merges the visible memtable delta with immutable
segment pages before returning a batch.

The checkout has not reached that layout. Its
[current physical-format reference](../reference/storage/rrflowkv-current-format.md)
records the implemented v3 LZ4 row-block segments, frozen bytes, recovery
rules, and removable pre-1.0 readers without promoting them into target
architecture.

## Conditional zero-copy

Memory mapping alone is not a universal zero-copy guarantee. A segment page
may be borrowed directly by Arrow only when its physical type, alignment,
endianness, compression state, validity representation, deletion state, and
requested projection permit it. The snapshot must retain the segment mapping
for the complete lifetime of every emitted batch.

Dictionary materialization, decompression, deletion filtering, type coercion,
memtable merging, computed expressions, and incompatible alignment may require
pool-owned buffers. Every physical plan therefore reports:

- mapped and physically read bytes;
- borrowed Arrow buffer bytes;
- decompressed and decoded bytes;
- copied and newly allocated bytes;
- rows/pages eliminated by key, projection, predicate, and limit pushdown; and
- peak memory, spill bytes, elapsed time, and output bytes.

The release claim is measured conditional zero-copy for eligible pages, never
"zero parsing, zero allocation" for every query. Apache Arrow likewise notes
that IPC reads can allocate when compression or other conditions require it.

## Read and query flow

```text
authorized intent + anchors + explicit budgets
                         |
                  capture one ReadStamp
 runtime cursor + manifest + valid time + schema/catalogue/policy
                         |
                    rrflowQL planner
              +----------+-----------+
              |                      |
              v                      v
       native fast path       analytical path
   point/range/CAS/tree       native candidate operators
   adjacency prefix scans     graph + BM25 + HNSW/exact
              |                      |
              |          rrflowKV stamped page/memtable scan
              |                      |
              |             Arrow RecordBatch stream
              |                      |
              |           DataFusion bounded execution
              +-----------+----------+
                          |
                deterministic exact rerank/RRF
                          |
             ContextPacket + evidence + same stamp
```

Fast state-machine navigation and bounded graph pointer work do not invoke
DataFusion. Analytical scans use a custom provider whose planning methods
declare supported pushdown without performing storage I/O; its execution stream
performs bounded reads when polled. Pipeline-breaking operators must reserve
memory or spill within the request budget.

Graph traversal uses versioned outgoing/incoming adjacency prefix ranges.
BM25 uses incrementally maintained dictionary, document statistics, postings,
positions, and tombstones. Vector search uses payload/scalar filters to narrow
eligible candidates, HNSW or exact candidate generation according to the cost
plan, and exact reranking. Reciprocal-rank fusion is a pure query-time operator;
verified outcomes may update a versioned policy only for later read stamps.

The local routing model does not read raw Arrow or storage buffers as model
input. `RrdEngine` constructs a bounded, typed route packet or an explicit
Arrow-to-tensor feature bridge. The model returns a grammar-constrained
proposal; deterministic predicates, authorization, CAS, and persistence remain
engine operations.

## Context assembly contract

The provider-neutral operation is `context-assemble`, served at
`POST /v1/context/assemble`. `rrd-contract` owns `AssembleContext`, the plan
and evidence types, and `ContextPacket`; `RrdEngine::assemble_context` owns the
implementation. The Rust client, RRD HTTP handler, CLI, MCP adapter, and any
embedded caller must invoke that operation or the same engine method. A host
adapter may translate its provider's request lifecycle at the edge, but it
cannot add a provider-specific context endpoint, retrieval pipeline, state
store, or lifecycle authority.

For one request, `RrdEngine` authenticates and authorizes the operation and
captures one runtime manifest, commit cursor, optional schema revision,
catalogue revision, and valid-time coordinate. The caller supplies intent, not
physical topology:

- one scope and query;
- one explicit nonzero valid-time coordinate;
- optional canonical record anchors; and
- explicit graph-depth, returned-item, output-byte, and scanned-change
  budgets.

The caller cannot select record fields, collection IDs, vector names,
embedding providers, indexes, or graph relations. The engine discovers visible
text, claims, compatible vector sources, and graph roots from the captured
snapshot and catalogue. Automatic semantic retrieval is eligible only for an
installed deterministic local text-embedding backend whose execution denies
network access.

The closed request contract enforces these hard ceilings:

| Resource | Maximum |
|---|---:|
| Query bytes | 64 KiB |
| Seed records | 256 |
| Returned items | 512 |
| Encoded item bytes | 768 KiB |
| Scanned runtime changes | 1,000,000 |
| Graph depth | 32 |

Within those ceilings, lexical hits are capped to `max_items`; compatible
vector sources and the exact-scored hits from each source are each capped to
`max_items`; graph traversal is capped to `max_items * 16` edge steps; and the
final fused result is capped by both `max_items` and `max_output_bytes`. Any
retrieval or output cap that hides remaining work sets `truncated=true`; a
truncated packet is never represented as complete.

Every returned item carries one or more evidence entries with the source kind,
source identifier, rank, source score, reciprocal-rank contribution, physical
plan digest, and evidence digest. The packet carries the read stamp, query
digest, exact encoded-item byte count, truncation state, and packet digest. Its
plan records all five canonical stages—seed, lexical, semantic, graph, and
fusion—in order, including whether each was selected or skipped, the selected
access path, exactness, reason, decision digest, compiled security-policy
revision, and authorization digest. The plan and packet must validate against
the same read stamp.

With the same validated request, authorization state, persisted read
coordinate, catalogue, and installed deterministic model set, assembly order,
fusion, evidence, and packet content are deterministic. Current physical paths
are snapshot BM25, exact snapshot-vector scoring, bidirectional snapshot graph
BFS, and reciprocal-rank fusion. Roadmap Gates E, F, and H replace their
whole-snapshot costs with native incremental access paths without changing the
public operation or its single-stamp semantics.

## Project-tree inventory and incremental attunement

Attunement begins with a deterministic project-tree snapshot. Parsing,
embeddings, indexes, graph relations, skills, routines, and model context are
not allowed to guess what a project contains or race an uncommitted filesystem
walk. The snapshot is the durable input boundary between an external project
workspace and RRFlow's governed knowledge.

```text
rrflow install preview
        -> resolve project root + inventory policy + budgets
        -> produce exact plan and plan digest; no writes

explicit apply(plan digest)
        -> RrdEngine creates estate + durable attunement job
        -> authorize one bounded inventory read plan
        -> enumerate metadata under the project root
        -> stream eligible file bytes for digest/classification only
        -> pure rrd-attunement inventory proposal
        -> RrdEngine validates and atomically commits:
             source-tree snapshot + containment edges + change set
             + inventory checkpoint + runtime-log entry + audit
        -> commit receipt permits the parse phase to begin
```

The engine-side attunement executor coordinates filesystem effects,
authorization, leases, cancellation, budgets, and commits. The planned
`rrd-attunement` crate is pure compute: it accepts bounded enumerated entries
and content chunks, sorts and normalizes them, and returns a deterministic
proposal. It cannot open paths, contact a provider, read secrets, access a
store, emit lifecycle callbacks, or commit state. No client, watcher, hook,
parser, or model owns project discovery.

### Canonical inventory records

| Record | Required semantics |
|---|---|
| `SourceTreeSnapshot` | Estate and project identity; logical root coordinate; snapshot identity and root digest; prior snapshot/cursor when incremental; inventory-policy, walker, and hasher revisions; entry/error totals; bounded-work accounting; creation commit. Host-absolute paths are evidence-local metadata, not portable record identities. |
| `SourceTreeEntry` | Stable identity from the estate plus normalized root-relative path; parent identity; entry kind (`directory`, `file`, `symlink`, `mount`, or `error`); size, executable/mode facts allowed by policy; content digest only for eligible files; classification and detector evidence; observed metadata revision. |
| `SourceTreeChangeSet` | Deterministically ordered additions, modifications, removals, and rename candidates between two committed snapshots, with old/new entry identities and the evidence used for each classification. |
| `InventoryPolicy` | Versioned root, traversal, ignore, content, secret, generated/vendor/cache, symlink, mount, size, file-count, byte, concurrency, timeout, and cancellation rules plus one canonical digest. |
| `InventoryError` | Root-relative coordinate, operation, stable error class, retryability, and redacted evidence for unreadable, vanished, replaced, or changed-during-read entries. Errors are data; they are never silently omitted. |

Entries are ordered by normalized path bytes before hashing or proposal
encoding. Directory digests are Merkle-style digests over the directory's own
normalized facts plus ordered child identities and digests; the project root
digest therefore changes when an included child, classification, or applicable
policy changes. Concurrency may improve enumeration and hashing, but it cannot
alter the committed order or digest.

The inventory commit creates only observed filesystem records and typed
`contains` relations. Package, module, import, export, symbol, call,
dependency, schema, and semantic relations require later parse, normalize, and
entity-link evidence. Filename conventions are useful classifier evidence,
not permission to invent those graph edges.

### Traversal, ignore, and safety rules

Inventory uses two bounded passes: metadata enumeration first, then streaming
content fingerprint/classification for eligible files only. It follows these
rules in order:

1. Enforce RRFlow safety exclusions for VCS internals, the project's rrflowDB
   state, build outputs, dependency trees, caches, temporary files, generated
   bulk data, and secret candidates before opening file content.
2. When the root is a Git worktree, retain tracked files and apply Git's exact
   ignore precedence to eligible untracked files. A matching ignore pattern
   cannot hide a tracked source. A non-Git project remains fully supported.
3. Do not blanket-ignore hidden paths: `.github`, `.cargo`, and similar
   project-owned configuration may be material. Classification comes from the
   versioned detector registry and persisted evidence, not scattered language
   checks in the walker.
4. Do not follow symbolic links or cross the authorized root/mount boundary by
   default. Record the leaf and any escape, loop, or mount decision. A more
   permissive policy requires an explicit operator grant and a new policy
   digest.
5. Exclude candidate secret paths before content reads. For ordinary eligible
   content, persist detector rule identities and redacted findings, never
   secret bytes. Environment credentials and external data-source credentials
   are not inventory inputs.
6. Bound entries, depth, individual file bytes, total bytes, open file
   descriptors, concurrency, elapsed time, and cancellation. A truncated or
   failed snapshot is explicit and cannot masquerade as complete input to a
   later phase.
7. Revalidate identity/metadata around each streamed read. Deletion,
   replacement, or mutation during the read becomes an explicit error or a
   bounded retry; mixed bytes never receive a clean content digest.

The initial safety profile excludes at least `.git`, RRFlow estate files,
`target`, `node_modules`, common generated/build/vendor/cache trees, sockets,
devices, and files over the configured ceiling. Exact defaults belong to the
versioned policy template and its tests, not prose-only special cases.

### Incremental refresh and downstream phases

An incremental run compares the previous committed entries and policy to the
new metadata pass. It reads content only when metadata, policy, detector, or
hash revision requires it. Unchanged entries preserve their identity and
digest. Rename is a deterministic change classification supported by matching
content and entry evidence; correctness never depends on an operating-system
rename notification.

Filesystem notifications are untrusted scheduling hints. Gate D supports the
explicit inventory operation without a watcher. After Gate I defines canonical
engine events, an explicitly installed watcher may submit a bounded external
event, but still cannot update a snapshot, call a parser, or mutate an index.
Overflow, dropped-event, or rescan signals force a fresh authoritative
inventory diff. An unchanged run still commits an explicit no-work checkpoint
with zero content reads, so resume and verification never infer completion
from silence.

Each later phase consumes an exact committed snapshot/change-set digest:

- parse incrementally reuses prior syntax trees only when source digest,
  grammar revision, and parser revision match;
- normalize and entity-link preserve unaffected canonical identities while
  recording unresolved or ambiguous evidence;
- lexical, embedding, vector, and graph phases propose atomic canonical facts
  and derived-index deltas through `RrdEngine`; and
- ground and verify prove source coordinates, projection cursors, package
  digest, restart readback, and every skipped/failed phase.

Every project-development operation has the same precondition. Before an AI or
human can plan a source mutation, `RrdEngine` resolves the latest complete
authorized project-tree snapshot at the operation's `ReadStamp`. A missing,
failed, truncated, policy-stale, or explicitly out-of-date snapshot returns a
typed `inventory-required` precondition or proposes the inventory operation; it
does not fall back to an ad hoc `find`, editor file list, model guess, or
provider cache. The context planner queries bounded tree records and
`contains` relations plus later grounded semantic edges. It never passes the
entire project tree just because the tree exists.

A source-mutation proposal binds its input snapshot and affected entry
digests. The authorized filesystem activity verifies those preconditions,
applies only the previewed changes, and then requests a new authoritative
inventory pass. Downstream reasoning waits for the resulting committed change
set. This makes “inspect the file tree first” an engine-enforced development
rule while still detecting a file changed outside RRFlow between planning and
apply.

The first acceptance corpus runs against this repository and covers stable
digests under different traversal schedules; add/edit/remove/rename; Git ignore
precedence including tracked ignored files; non-Git and hidden source;
symlink escape/loop and mount boundaries; unreadable, vanished, and racing
files; secret candidates; generated/vendor/cache exclusions; cancellation and
restart; resource ceilings; close/reopen; and an unchanged rerun with zero
content reads. It must prove parse cannot start before the snapshot and
inventory checkpoint share a successful commit receipt. Gate I's first
project-development routine must additionally prove that no current complete
snapshot returns `inventory-required`, a changed-since-plan file fails its
precondition, and no model or adapter can authorize a path outside the
committed tree/root policy.

## Automation, routine, and skill flow

RRFlow automation is durable engine state, not a host callback framework and
not another runtime. The following responsibilities are disjoint even though
they share one rrflowDB transaction, graph, query, and evidence model:

| Element | Owns | Cannot own |
|---|---|---|
| Engine event | One immutable, uniquely identified occurrence with producer, action, target, scope, payload schema, provenance, causation, correlation, idempotency, and commit coordinates. | Execution, authorization, scheduling, or UI styling. |
| Trigger | A versioned deterministic predicate over committed engine events and a bounded operation or routine-start proposal. | Arbitrary code, direct mutation, transport calls, or retry loops. |
| Routine definition | An immutable, digest-bound directed graph of semantic operation steps, conditions, budgets, retry rules, compensation, verification, and terminal states. | Mutable run state, physical access-path selection, or provider-specific commands. |
| Routine run | Durable execution state pinned to one routine revision, activation event, specialization, policy, and input digest. | Redefining its routine during replay or inferring success from a trace/client. |
| Skill package | Immutable instructions, schemas, examples, evaluation cases, and resource descriptors resolved at a read stamp. | Permissions, mutation, process execution, scheduling, or database/index selection. |
| Function | A bounded deterministic transform or proposed-transaction validator executed by an engine-owned sandbox. | Durable orchestration, post-commit event matching, network access, or lifecycle state. |
| Host-event adapter | Explicit translation of one authenticated external occurrence into the public engine-event submission contract. | Automatic installation, canonical state, routine scheduling, or storage access. |
| MCP adapter | Projection of public RRD operations/resources/tasks and, when configured, transport for one external capability invocation. | A second context assembler, workflow engine, database trigger system, or presentation authority. |

### One canonical event path

Internal and external occurrences become the same canonical engine event:

```text
accepted semantic mutation                 authenticated external occurrence
          |                                               |
          | same transaction                              | submit operation
          v                                               v
       RrdEngine validates identity, schema, scope, policy, and idempotency
                                  |
                                  v
       semantic commit: data/index deltas + engine event + outbox + audit
                                  |
                         rrflowKV commit receipt
                                  |
                                  v
                    committed-event consumption only
```

An internally caused event is included in the same semantic write batch as the
change it describes. An external host, process, telemetry source, or MCP client
submits a bounded event through an authenticated public operation; triggers do
not see it until that submission commits. Producer identity plus event identity
is unique, and a repeated identity with different canonical bytes is an
idempotency conflict. Payloads are schema-bound and size-bound, with large
content stored by verified object reference.

The kernel event value and public event envelope are two representations of
this one semantic object, not separate event systems. I-01 must directly
converge the current `RuntimeEvent` value into the canonical engine-event
vocabulary and define its exact lowering into `RuntimeMutation::Event`; no
forwarding event type or parallel event log survives.

The existing synchronous `FunctionTrigger` behavior occurs before a proposed
transaction commits. It is therefore a transaction function binding. It may
validate the proposed transaction or derive an event inside that same commit,
but it is not a post-commit event trigger. Pre-release convergence renames that
contract directly so the word `trigger` has only one automation meaning.

### Durable trigger and routine execution

```text
committed event at cursor N
        -> trigger worker leases a bounded cursor range
        -> pure predicate evaluation against pinned trigger/specialization
        -> activation decision committed with reason and evidence
        -> unique activation key(event identity, trigger digest, routine digest)
        -> RrdEngine commits Pending RoutineRun or a denial
        -> worker leases one ready step
        -> RrdEngine authorizes the step's semantic operation
        -> operation returns a typed result or mutation proposal
        -> RrdEngine commits result + effects + checkpoint + audit + events
        -> next ready step, waiting state, compensation, or terminal outcome
```

Trigger cursors, activation decisions, routine definitions, routine runs, step
attempts, leases, cancellations, checkpoints, and terminal outcomes are
ordinary governed rrflowDB records. Their causal relationships are typed
temporal graph edges. Only the commit receipt advances them. Workers are
replaceable pollers over leased state and can crash after any external effect;
therefore every step carries an idempotency key and persists the accepted
result digest before dependent steps become ready.

Routine graph traversal is deterministic. Wall-clock reads, model calls,
process execution, network access, and external MCP calls are explicit
activities outside replay. Their inputs, selected capability identity,
timeouts, attempts, outputs, and evidence are committed before replay uses the
result. A definition revision never changes an existing run. Cancellation is a
persisted request evaluated at step boundaries; timeout and compensation are
explicit transitions, not background guesses.

A routine step names a versioned public RRFlow operation or an attuned
capability reference. It cannot name a Rust function, executable path, shell
fragment, provider, MCP endpoint, storage profile, KV prefix, table provider,
graph implementation, BM25 implementation, HNSW generation, TurboQuant
generation, or DataFusion node. The installed specialization resolves generic
capabilities to exact authorized bindings. This is how a generic
`verification` step can become Cargo, pnpm, pytest, or another direct argv only
after project inventory and operator policy prove that binding.

### Skill resolution

A skill package is content, not an agent or plugin runtime. Its immutable
manifest binds:

- canonical identity, revision, media type, byte size, and content digest;
- compatible engine/operation schemas and required estate capabilities;
- instruction, input/output-schema, example, evaluation, and resource
  descriptors, each pinned by digest;
- context-token/byte limits, data classification, provenance, dependency
  digests, and retirement state; and
- requested capabilities and secret references, which remain inert until
  separately allowed by estate policy.

Attunement may install generic skill packages and record eligibility, but
activation is a separate persisted policy decision. A routine step resolves an
exact skill revision through `RrdEngine` at its captured read stamp. The
resolver returns a bounded context projection plus evidence; instructions
cannot grant themselves tools, increase budgets, select indexes, or commit
state. An update creates another immutable revision, while existing routine
runs retain the digest they started with.

### Context, graph, indexes, TurboQuant, and DataFusion

Knowledge graph edges, reasoning-tree edges, and routine-control edges are
different typed relation families in the same temporal graph and transaction
model. Routine traversal follows only routine-control relations. Context
assembly may traverse authorized knowledge/reasoning relations and returns the
contributing edge identities as evidence.

Every routine or interactive request expresses semantic intent, anchors,
freshness, quality, and resource budgets. `RrdEngine` captures one `ReadStamp`
and delegates physical planning to rrflowQL:

1. Exact point, range, state-machine, and bounded adjacency work uses the
   native fast path.
2. BM25, payload/scalar filtering, graph adjacency, exact vectors, HNSW, and
   TurboQuant are eligible native access operators selected from the stamped
   catalogue.
3. TurboQuant is a derived candidate projection bound to collection, vector,
   model, metric, dimensions, configuration digest, generation, and source
   cursor. It may reduce candidate work only when active, fresh, and admitted
   by measured quality policy; exact canonical vectors perform required final
   scoring.
4. Broad scans, joins, aggregations, telemetry analysis, and columnar
   transformations stream rrflowKV pages and memtable overlays as bounded Arrow
   batches through DataFusion.
5. Deterministic fusion produces one `ContextPacket`; no access path can return
   data outside the captured stamp or omit its work and contribution evidence.

Indexes are not refreshed by a provider hook. Schema/catalogue rules lower an
accepted semantic mutation into atomic current, temporal, outgoing/incoming
adjacency, scalar/unique, BM25, exact-vector, runtime-log, and projection-delta
changes. Rebuildable HNSW/TurboQuant workers consume committed deltas, build an
immutable generation at a source cursor, verify it against the exact oracle,
and propose activation through `RrdEngine`. A missing, stale, corrupt, or
quality-ineligible projection causes an exact fallback or an explicit budget
denial, never silent stale recall.

The eventual rrflowKV binary key codec uses typed, length-delimited ordered
tuples rather than ambiguous human-delimited strings. C-01 freezes the exact
bytes. Its semantic families must preserve bounded prefix/range access for
current and temporal records, both adjacency directions, scalar/unique values,
BM25 terms/postings, canonical vectors, projection generations/deltas, engine
events, trigger cursors, routine state/checkpoints, skill manifests/bindings,
outbox delivery, and audit evidence. Physical key families accelerate one
logical rrflowDB estate; they do not become public APIs or independent stores.

### MCP and presentation boundary

RRFlow can participate in MCP in two directions without moving authority:

- inbound, the RRFlow MCP server exposes the same context, event, routine,
  skill, status, cancellation, and subscription operations as HTTP/WebSocket
  and SDK clients; and
- outbound, an explicitly configured capability adapter may invoke an external
  MCP tool for one leased routine step after pinning server identity, tool and
  input-schema digests, permissions, secrets references, timeout, idempotency,
  and result limits.

An MCP task identifier may project a durable routine-run identifier, but MCP
task state is never the source of truth. Disconnect, reconnect, cancellation,
or another provider must resolve the same persisted routine state.

Connectome may render low-latency tooltips, panels, graphs, or modals from a
typed, schema-validated presentation proposal referencing persisted data and
evidence. RRFlow never hides control tags in model prose, parses UI behavior
with a streaming regex, or represents a deterministic rule/database result as
unexplained model reasoning. The client owns accessible styling; `RrdEngine`
owns authorization and semantic payload validation; every displayed claim can
identify whether it came from canonical data, deterministic computation, a
model proposal, or an operator action.

### First complete automation proof

The first vertical slice is one generic `error-resolution` routine:

```text
typed diagnostic event
 -> policy/capability trigger
 -> durable error-resolution run
 -> pinned language/project skill resolution
 -> context intent anchored to diagnostic + file + symbol
 -> stamped graph/BM25/vector/TurboQuant/DataFusion eligibility decisions
 -> bounded model or deterministic proposal
 -> separately authorized mutation proposal
 -> attuned owning verification capability
 -> committed verification/result/checkpoint/evidence
 -> restart and cross-surface readback without duplicated effects
```

The acceptance corpus must exercise match, non-match, denial, stale index,
exact fallback, analytical escalation, malformed model/tool output, crash after
external effect, retry, cancellation, compensation, restart, and uninstall.
HTTP, WebSocket, SDK, MCP, and Connectome must observe the same run identity,
state, stamp, result digest, and evidence. Component tests or a successful
compile cannot substitute for this proof.

## Current implementation boundary

| Concern | Present checkout | Required target |
|---|---|---|
| rrflowKV writes and hot reads | Checksummed WAL frames, mutable MVCC version chains, snapshots, authenticated block filters, a byte-bounded decoded-block cache, physical counters, and an AI-hotset benchmark. | Preserve the useful WAL, snapshot, filter, cache, and measurement behavior while C-02/C-04/C-06/C-07 replace the storage contract and immutable format; F-05 decides the final cache from measurements. |
| rrflowKV immutable storage | LZ4-compressed row-record blocks with block indexes, manifests, compaction, recovery, and pre-1.0 readers. | Ordered key/version spine plus Arrow-compatible column pages; remove every earlier-format reader before alpha exit. |
| Transactions, identity, and audit | Authenticated `ReadStamp`, `DataTransaction`, session/policy checks, commit receipts, audit envelopes, and native/volatile implementations exist at different levels of integration. | C-02/C-03 and H-04/H-05 must prove one snapshot/conflict/authorization/audit contract across rrflowMX, rrflowKV, embedded, and transport paths. |
| Arrow conversion | Materialized `QueryRow` values are converted into newly allocated typed Arrow arrays. | Stream eligible segment buffers and bounded decoded/memtable overlays through a stamped provider. |
| DataFusion | Real bounded execution over the materialized Arrow snapshot. | Push projection/predicate/limit into rrflowKV and compose native graph/BM25/vector operators at one stamp. |
| Project discovery and attunement | B-01 freezes the provider-neutral eleven-phase plan/job/checkpoint contract, but no engine-persisted executor, project-tree snapshot, inventory phase implementation, or parse phase exists. | D-01 through D-05 must prove preview/apply, a deterministic committed tree snapshot and change set, pure phase proposals, restartable checkpoints, and downstream work pinned to exact source/policy/tool revisions. |
| Embedding and vectors | Deterministic local embedding, model/provenance binding, exact search, filtered planning, compact dense artifacts, HNSW, quantization, and accelerator differential checks exist. | D-05/E-04/E-05/F-03 must bind canonical vectors and every derived artifact to one committed source cursor, maintain atomic deltas, and preserve exact fallback/reranking. |
| Graph and BM25 | Semantic graph and lexical foundations exist, but current context execution still reconstructs broad snapshots. | Transactionally maintained adjacency and BM25 access paths with exact fallback, reopen, corruption, and bounded-work proof. |
| Reasoning and context | Generic reasoning-tree, router, context-plan, evidence, and bounded context contracts exist; current assembly uses snapshot BM25, exact vectors, graph BFS, and RRF. | G/H must persist CAS tree execution and dynamically select native or analytical paths without adding another model, planner, or context authority. |
| Automation, skills, and MCP | A deterministic JavaScript/WebAssembly function catalogue and synchronous proposed-transaction bindings exist; MCP exposes one context tool; no canonical engine event, post-commit trigger, durable routine, skill package, or external MCP capability runner exists. The package-command note now describes only the target capability boundary and claims no evidence. | I must directly converge event terminology, retain the bounded function sandbox under its narrower name, persist triggers/routines/skills through `RrdEngine`, project them across public surfaces, and remove all host-lifecycle residue. |
| Edge and outward delivery | The separate `rrflow-edge` adapter has deterministic offline/provenance evidence; HTTP, SDK, MCP, CLI, and subscriptions have partial real-process coverage. | H/J must route every surface through the same public operations and prove correlated identity, authorization, stamp, result, restart, and distribution behavior. |

These differences are tracked by POAM-002 through POAM-010 and their owning
roadmap gates. POAM-014 tracks implementation traceability across storage,
security, attunement, inference, reasoning, automation, and outward boundaries.
No present type, file format, merge relationship, or passing compile closes
them.

## External systems

PostgreSQL, Turso, SQLite, Dragonfly, object stores, mesh networks, model
providers, and application databases are optional governed integrations. An
operator explicitly configures an adapter; attunement may discover that an
integration is possible but cannot activate it. External systems do not become
RRFlow's canonical persistence, transaction coordinator, query planner, or
context authority.

## Source constraints

The architecture uses these upstream behaviors without inheriting another
system's authority:

- [Apache Arrow columnar format](https://arrow.apache.org/docs/format/Columnar.html)
  defines relocatable columnar buffers and the mutation tradeoff.
- [Apache Arrow IPC](https://arrow.apache.org/docs/cpp/api/ipc.html) documents
  both zero-copy-capable reads and cases requiring allocation.
- [DataFusion custom providers](https://datafusion.apache.org/library-user-guide/custom-table-providers.html)
  separate planning-time pushdown declarations from execution-time streams.
- [Git ignore semantics](https://git-scm.com/docs/gitignore) and
  [Git's content-addressed object model](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects.html)
  inform tracked-file reconciliation, deterministic path precedence, and the
  Merkle-style snapshot; RRFlow retains its own policy and digest envelope.
- [Tree-sitter incremental parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)
  informs reuse after a committed change set; it does not replace inventory.
- [Lance read and write behavior](https://lancedb.github.io/lance/introduction/read_and_write.html)
  demonstrates immutable columnar fragments, deletion metadata, and compaction
  tradeoffs; rrflowKV is not a Lance clone.
- [Qdrant storage](https://qdrant.tech/documentation/manage-data/storage/) and
  [indexing](https://qdrant.tech/documentation/manage-data/indexing/) inform
  WAL/segment recovery, filter-aware candidates, and persisted index behavior.
- [CloudEvents](https://github.com/cloudevents/spec/blob/main/cloudevents/spec.md)
  informs transport-neutral occurrence identity and producer-scoped duplicate
  handling; RRFlow retains its own closed, estate-bound event contract.
- [Temporal workflow determinism](https://github.com/temporalio/documentation/blob/main/docs/encyclopedia/workflow/workflow-definition.mdx)
  informs deterministic replay and the separation of external activities from
  durable control flow; RRFlow does not add Temporal as another runtime.
- [Model Context Protocol architecture](https://modelcontextprotocol.io/specification/2025-06-18/architecture)
  and [task projection](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/tasks)
  inform adapter isolation, capability negotiation, and long-running operation
  projection; MCP never becomes RRFlow's routine authority.
- [OCI content descriptors](https://github.com/opencontainers/image-spec/blob/main/descriptor.md)
  inform media-type, size, and digest binding for portable skill resources;
  RRFlow's signed offline distribution remains the deployment authority.

## Qualification

The architecture is implemented only when the owning roadmap gates prove:

1. exact transaction, snapshot, key ordering, and rrflowMX/rrflowKV
   conformance;
2. crash recovery and compaction while read stamps and mapped buffers remain
   live;
3. atomic record, graph, scalar, BM25, vector, runtime-log, and index-delta
   maintenance;
4. streaming and pushdown against data larger than the allowed query memory;
5. exact-oracle and recall-quality results after update, delete, rebuild, and
   reopen; and
6. fixed-hardware latency, memory, copy/decode, disk, and failure evidence.

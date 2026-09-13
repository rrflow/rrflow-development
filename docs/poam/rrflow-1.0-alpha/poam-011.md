# POAM-011 — public surfaces, SDKs, and Connectome

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-011`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Public transports, SDKs, MCP, mesh resolution, and Connectome have not passed one
cross-surface real-process corpus. A-07.1b split the Rust client's former implementation
monolith into direct responsibility modules and replaced its public bearer-bearing lease
field with a credential-private, redacted session handle. B-04 now removes the dedicated
subscription socket and supplies one closed, bounded `/v1/ws` protocol plus a Rust
client/server implementation: validated limits configure carriage before upgrade/allocation;
exact connection, sequence, request, cancellation, subscription, generation, cursor, ACK,
heartbeat, error, and backpressure coordinates are enforced; two durable streams multiplex
and replay independently; and malicious connection/sequence/shape/binary/size/response
faults fail closed. Generic operation execution and cancellation remain truthfully disabled
until H-04, and no generated SDK has WebSocket carriage. The Rust client also still
implements only 28 of 33 catalogued HTTP operations; the bearer remains an ordinary
internally copied string-like identifier; and complete HTTP validation, secret lifecycle,
retry certainty, actual operation cancellation, resolver rotation, trace propagation,
installed-profile conformance, and fail-closed harness behavior remain absent. A-07.1c split
the TypeScript client's existing HTTP implementation into direct responsibility modules and
directly renamed its conformance entry, while leaving `subscription.ts` absent until real
WebSocket behavior exists. It can still address all 33 generated HTTP descriptors only
through one generic call; it has no WebSocket or remote-HTTPS profile, applies no runtime
request/success-payload validation, ignores response media and exact success status, exposes
bearer/API-key strings in plain serializable objects, retries unpinned non-mutating POST
observations and broad thrown failures immediately, and lacks W3C propagation. A direct
probe accepted `201 text/plain` with an invalid query payload after replay and serialized
the bearer. Its private raw-TypeScript package emits no JavaScript/declarations and has no
Node-minimum, browser, package-consumer, or offline-install proof. A-07.1d split the Python
client's existing synchronous behavior into direct responsibility modules, removed the
catch-all model module, and added the packaged typing marker without inventing async or
socket behavior. The synchronous client still has the same generic 33-descriptor shape and
no async, WebSocket, or remote-HTTPS profile; request and success payloads are unvalidated,
exact status/media/GET correlation/digest bindings are absent, public mutable dictionaries
expose session credentials, unpinned non-mutating POSTs and timeouts enter an immediate
retry loop, error details are discarded, and there is no W3C propagation. Its direct probe
reproduced the same replay plus `201 text/plain` invalid-payload acceptance and serialized
bearer. Its wheel now carries `py.typed`, but complete release metadata,
supported-interpreter/platform testing, clean consumer proof, reproducibility, and signed
offline dependency closure remain absent. A-07.1e split the Go client's current HTTP
behavior into idiomatic direct responsibility files, removed the catch-all `models.go`, and
added package documentation without inventing socket, generated-model, or orchestration
behavior. Its context-aware generic 33-descriptor HTTP call still has no WebSocket or
authenticated remote profile; it accepts and returns untyped maps, ignores exact success
status and media, leaves GET correlation and causal bindings unchecked, exports API-key and
bearer fields, replays every non-mutating POST after broad transport failures and
per-attempt timeouts, uses the process-global default transport with ambient proxy/resource
policy, has no server cancellation or W3C propagation, and has no
supported-toolchain/platform/module-consumer qualification. Its direct probe replayed an
invalid unpinned query, accepted `201 text/plain` with an invalid payload, and exposed the
bearer through JSON plus formatting. A-07.1f split the Java client's current HTTP behavior
behind its unchanged public facade into package-private responsibility classes and added
package documentation without inventing async, socket, generated-model, or orchestration
behavior. Its synchronous generic 33-descriptor HTTP call still uses untyped payload/result
values and has no async, WebSocket, or authenticated remote profile; it ignores exact
success status and media, leaves GET correlation and causal bindings incomplete, stores API
keys and bearer tokens in public records, immediately replays every non-mutating POST after
broad I/O failures and request timeouts, inherits JDK proxy/executor/pool defaults without a
connect timeout or exposed close, and has no server cancellation or W3C propagation. Its
direct probe replayed an invalid unpinned query, accepted `201 text/plain` plus invalid
payload, and exposed the bearer through record formatting and Jackson serialization.
Consecutive clean JARs differed, an empty-cache offline Maven test failed before
compilation, and the artifact has only a filename-derived automatic module with no clean
consumer, supported-JDK/platform, or signed dependency closure. A-07.1g pinned the .NET
workspace to SDK 10.0.111, centralized common build/dependency inputs, moved the root README
and sole generated projection to direct paths, removed `Models.cs`, and split the existing
HTTP responsibilities while preserving all 140 exported signatures. The asynchronous client
still exposes one generic 33-descriptor HTTP call with arbitrary object/`JsonElement` shapes
and no WebSocket or authenticated remote profile; it ignores exact success status and media,
leaves GET correlation and causal bindings incomplete, stores API keys and bearer tokens in
public records, immediately replays non-mutating POSTs after broad transport/timeout
cancellation, inherits `HttpClientHandler` proxy/cookie/pool/header and independent
client-timeout defaults, and has no server cancellation or explicit W3C binding. Its direct
probe replayed an invalid unpinned query, accepted `201 text/plain` plus invalid payload,
and exposed the bearer through record formatting and `System.Text.Json`. Its manifest-absent
test reports a false pass; consecutive DLL/PDB builds matched but `.nupkg` bytes differed,
signature verification failed, an empty-feed solution restore failed, package metadata is
placeholder/incomplete, and the pin still lacks offline toolchain qualification,
external/trimmed/AOT consumers, and a supported platform matrix. The shared live harness
bypasses installation through direct rrflowKV/security/estate fixture setup, and its domain
labels overstate CRUD, backup/restore, vector, live-feed, error, retry, cancellation, and
version coverage. Generated-surface parity proves only method/path/auth/mutation metadata.
The separate Connectome repository at `38f68ce7adda9d03501f3591165f0e14996899ec` passes
TypeScript/Biome/identity checks, a production build, four focused native RRD tests, and two
Chromium smoke tests, but it handwrites a partial RRD client, has no network TLS or
WebSocket path, duplicates stale deployment and attunement/automation contracts, defines a
retired parallel diagnostics protocol and an unaccepted `rrflow-control` protocol, hardcodes
cloud topology/reconnect behavior, retains SurrealDB client/UI/CBOR/QL/Wasm authority, and
tests only a mocked three-GET browser handshake rather than an installed engine.

## Impact

User-visible behavior can disagree with the engine, credentials can reach diagnostics or
browser code, a client can consume unintended resources or replay work with the wrong
semantics, and green generation/type/mock/live-fixture output can falsely qualify an
incomplete or unusable surface.

## Owning gates

A-07, B-04, B-05, D-01, H-04 through H-07, J-01 through J-05

## Closure evidence

Every supported SDK binds every catalogue descriptor exactly once; complete generated
runtime request/response/status/media/identity validation, redacted opaque sessions,
semantic retry/uncertainty, correlated server cancellation, bounded multiplexed frames, W3C
propagation, authenticated endpoint rotation, and adversarial resource/error tests pass.
TypeScript additionally ships deterministic ESM JavaScript/declarations in the signed
offline distribution and passes minimum/current Node plus real-browser consumer, CORS,
credential, HTTP, and WebSocket matrices. Python additionally ships reproducible sdist/wheel
artifacts with `py.typed`, complete metadata, tested dependency constraints, and a signed
offline closure; one shared semantic layer drives native sync/async clients and passes
minimum/current CPython, type-consumer, HTTP, WebSocket, cancellation, TLS, and platform
matrices. Go additionally ships a documented deterministic nested-module artifact and
dependency closure; uses an explicit bounded concurrent transport, opaque credentials,
idiomatic typed errors, and caller plus correlated server cancellation; and passes
minimum/current Go, race, external-consumer, HTTP, WebSocket, TLS, OS, and architecture
matrices with toolchain/network behavior explicit. Java additionally ships byte-reproducible
main/source/Javadoc JARs, a stable module identity, exact POM/plugin/runtime closure, and a
file-backed signed offline repository; uses an explicit bounded closeable HTTP/WebSocket
carriage, opaque credentials, typed async plus blocking APIs, interruption and caller/server
cancellation; and passes minimum/current JDK and Maven, classpath/module-path,
external-consumer, HTTP, WebSocket, TLS, concurrency, OS, and architecture matrices. .NET
additionally ships byte-reproducible compiled, symbol/source, and NuGet artifacts with
complete metadata, package validation, an exact SDK/dependency/feed closure, and verified
signatures; uses explicit bounded `SocketsHttpHandler`/`ClientWebSocket` carriages,
source-generated validated models, opaque credentials, `Task`/`IAsyncEnumerable` APIs, and
correlated caller/server cancellation; and passes the pinned supported SDK/runtime,
external/trimmed/Native-AOT consumer, HTTP, WebSocket, TLS, concurrency, OS, and
architecture matrices. A D-01-installed rrflowMX/rrflowKV real-process corpus structurally
proves each scenario and compares result, denial, stamp, digest, receipt, trace, restart,
and resource evidence across HTTP, WebSocket, Rust, every generated SDK, CLI, MCP, GraphQL,
and Connectome; missing harness configuration fails or is explicitly skipped and never
reports conformance.

## Convergence update

POAM-011 convergence update (2026-09-08): B-05 now provides bounded,
catalogue-derived GraphQL query parsing and lowering into the same
`Query`/`Parameters`/`BoundQuery` path as rrflowQL. It adds no public route,
resolver, storage call, or second executor. POAM-011 remains `Sequenced`
because H-04 still owns authenticated outward GraphQL carriage and the full
cross-surface real-process corpus.

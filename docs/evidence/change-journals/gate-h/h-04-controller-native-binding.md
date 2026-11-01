# H-04: generated native controller proposal binding

**Status:** active draft implementation evidence; live installed client proof and protected landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-controller-native-binding`
**Owner:** H-04 / POAM-011; G-01 proposal semantics
**Pull request:** [222](https://github.com/rrflow/rrflow-development/pull/222), dependent on [206](https://github.com/rrflow/rrflow-development/pull/206)

## Bound change and before/after oracle

Driver: SWAG controller session. From clean main `6fe8b7590d913beda9b009551e5d2d2644e406d9`, the isolated branch fast-forwarded its declared dependency to `f278fa6f4233f32b069eff4e5fa00bf50ba12aac` (tree `9c27d87922d47afb2d1532bd3b9f4c0dbfea488f`). The PR brief preceded implementation. Refinements bound generator rejection checks and a native example/live proof before those edits.

Before: the public catalogue advertises endpoint proposals, but the Rust client used by Connectome has no callable binding. After: `call_operation::<operations::TaskEndpointPropose>` uses generated request/response types, path and mutation metadata with explicit contract-owned request/output identity validation. The existing general session helper can retry transport failures; this new operation always sends once, checks exact HTTP success status, JSON media type and response correlation (including errors), and retains the existing bounded decoder. A lost response requires engine attempt readback, not automatic redispatch.

The existing contract exporter now has explicit `--catalogue` mode. The surface generator projects all 51 fixed-path session POST markers and fails on stale output. Metadata is not callable coverage: only the proposal marker has a semantic validator; a compile-fail example proves another marker cannot be invoked. Existing legacy methods retain their behavior and remain part of the open H-04 migration. No second path/type registry or handwritten provider policy was added. The existing full engine proposal validator reuses the new request-identity check; packet/configuration validation remains at its existing owner. Wire schemas and OpenAPI digest are unchanged.

## Traceability and reviewed boundaries

Reviewed AGENTS, README owner links, H-04/Gate G/POAM-011, Rust SDK reference, existing client operation/transport/session code and task-effort/real-server fixtures, contract proposal/export/catalogue, surface generator, installed receipt and proof driver, and installation/configuration denial boundaries. Complete authored files and their diffs are reviewed before publication. Git inventories the incoming dependency; no unrelated source is deleted, moved or consolidated.

Created the Rust metadata projection, sealed semantic call module, native example, focused client tests and projection generator/tests. Changed contract export/proposal validation, client module/transport, surface checking and its CI test registration, optional native mode in the existing installed proof driver, this journal and SDK/POAM references. Generated navigation/inventory remain replaceable projections. No new normal client dependency, renderer credential, alternate lifecycle, training record, ranking or worker authority.

The example follows existing native installed-locator/credential bootstrap, bounds input reading, accepts explicit project/address/request/timeout, emits typed output separately from raw HTTP receipts, and records session-close failures. Proof-driver native mode excludes fault, admission-only and recovery combinations. Its later live receipt must distinguish the original engine binary identity from the new client identity. No captured response can stand in for that live call.

No external algorithm or library adaptation was selected. Existing primary source code and actual retained receipts determine this boundary. Trace/resource choice: bounded request/response and explicit correlation; no new credential or automatic prompt logging, performance claim or GPU training. Stop on semantic-validation loss, implicit retry, duplicate catalogue authority, engine-admission changes or unrelated transport regression.

## Actual checks and failures

- Initial `cargo check --locked -p rrd-client`: passed, compiling all generated type associations.
- Initial focused suite: 2 passed, 1 failed. A diagnostic rerun retained the failure: the uninstalled engine reports that endpoint configuration requires an installed project identity, before it can report absent configuration. The test now retains this exact denial and adds a separate actual installation-API fixture; no engine guard was weakened.
- Corrected focused suite: 4 passed. It validates all five request identities against the retained actual proposal; real engine permission, installation and absent-configuration denials; archived-response transport, foreign identity, wrong/duplicate media, wrong success/redirect status, dropped response with retries configured as three, invalid request/missing idempotency and expired deadline before any send.
- `cargo test --locked -p rrd-client -p rrd-contract`: 170 passed, zero ignored, including the compile-fail validator gate. Captured-response probes are explicitly not new provider results or routing evaluation.
- Strict all-target Clippy for client/contract passed before the example was added. The new example's first build failed on usize/u64 comparison; the bounded read now uses checked conversion. Its build and strict example Clippy passed afterward.
- The format check first caught the newly added unformatted example; after formatting, `cargo fmt --all -- --check` passed.
- A documentation update failed under the Windows default text encoding. The SDK reference was rebuilt from original Git UTF-8 bytes and the documentation gate then caught the same heading encoding damage and duplicate addition in POAM-011. That record was also rebuilt from original Git UTF-8 bytes with one addition; no captured receipt bytes changed.
- Projection tests: four passed against the actual export with metadata corruption probes. Generation is deterministic; final repeated generation/parity and repository policy checks are recorded in local logs before publication.

The private endpoint returned HTTP 200 to health and model discovery during preparation. That is access evidence only. No new live model call has completed at this record's creation; its later immutable receipt must report success or failure independently. Existing raw/typed receipt replay is transport evidence only. No full candidate, protected merge, UI journey, worker execution, source-edit workflow or model qualification is claimed. The draft depends on PR206 and the unresolved shared native CI failure. Roadmap/POAM acceptance remains unchanged. Logs are retained under the operator's October 30 `controller-native-client` evidence directory.

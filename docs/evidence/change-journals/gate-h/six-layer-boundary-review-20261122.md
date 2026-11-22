# Six-layer boundary review before further engine implementation

**Status:** active review evidence; architectural mandate is not satisfied
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/six-layer-boundary-review-20261122`
**Owner:** review disposition of the [temporal relation package](temporal-referential-actions-20261122.md); [Gate F](../../../roadmap/rrflow-1.0/gate-f.md) owns cold-query convergence

## Change brief and evidence boundary

The owner supplied an explicit six-layer mandate on 2026-11-22 and required
review before further implementation. Inspected root revision:
`38d1b3b1ceaa09e0c8a1266c8591490cc03d5257`, on the existing
`feat/temporal-referential-actions` branch with draft
[PR496](https://github.com/rrflow/rrflow-development/pull/496).
Its remote previously held `ea2713a7e247dd6ca697e2eb3970686fc97e7771`.
The paired Connectome branch remains draft
[PR119](https://github.com/rrflow/connectome-development/pull/119), at
`dd54eb1d90b4ec1036ebb33910b206e741e6b545` when inspected. Neither draft is
merged or six-layer acceptance evidence. The parent gitlink is not advanced by
this review.

This change records the actual implementation boundaries and the first
replacement boundary. It changes this evidence record and generated navigation
only. No runtime code, feature, schema, storage format or retention policy was
changed after the mandate. Existing implementation is preserved for review.
The PR description received this review brief before the evidence file edit.
No roadmap item or POA&M is closed.

The review is targeted, not exhaustive. The complete implementation files read
are listed below with Git object identities. Additional partial reads of the
transaction orchestration and repository commit path are not whole-codebase
coverage. Git inventory, previous test counts and earlier source summaries are
not substituted for the listed reads.

## Findings against the required ownership

| Layer | Concrete implementation finding | Disposition |
|---|---|---|
| QL | `pipeline.rs` binds catalogue, plan and execution to the same supplied `ReadStamp`, using `StorageEngine`. However `cold_history.rs` imports `ParquetRecordBatchReaderBuilder`, reads Parquet metadata, encodes artifacts and decodes change JSON. `lib.rs` publicly exports that codec. | Preserve stamped query validation; move physical cold encoding and decoding out of QL. The current QL boundary fails the mandate. |
| TX | `transaction_admission.rs` admits one active effect owner through a mutex/condition-variable queue. `SemanticCommitPlan` requires the global runtime cursor to equal `expected_cursor`; every semantic commit writes that cursor and catalogue watermark. | Snapshot checks exist, but this is not independent concurrent semantic commit execution. Keep outcome reconciliation and atomic effects during separation; removing the gate alone would be unsafe. |
| MX | `engine.rs` implements `RrflowMxStore` with `Mutex<BTreeMap<...>>`, owned byte vectors and cloned reads. `StorageProfile` chooses MX or KV. | It is a volatile complete-storage profile, not a lock-free Arrow focus tier composed with durable storage. Replacing its name would not implement the requested behavior. |
| DB | The storage trait exposes physical transactions and shared semantic repositories. Cold publication/readback bypass that trait with `as_rrflow_kv()`, `anchored_path()` and filesystem operations in engine orchestration. | A DB-owned placement/scan/publication boundary must replace those concrete calls. The current trait is not the requested unified tier router. |
| LSM | `database/transactions.rs` pins physical transaction snapshots, checks written-key conflicts, appends WAL before publishing the memtable and fences uncertain post-WAL failures for reopen. Projected readers retain owned segment/memtable views. Foreground writes can flush and compact under pressure. | These are reusable guarantees. They do not establish lock-free writers, WiscKey separation, focus-driven Arrow demotion or sub-millisecond cold reads. Those capabilities were not proved by this review. |
| CLD | Production cold history is derived Parquet with receipt-chain and source-digest validation. `ColumnarFormat` instead exchanges owned `ColumnarTable`/`Vec<u8>` values and supports only UTF-8/u64 columns. Optional Vortex/Parquet comparison implementations are not used by production. | No production `ColdStorageBackend` exists at the reviewed boundary. Current publication explicitly never reclaims/replaces source history. It cannot be called an authoritative cold tier. |

Specific copy and test behavior matters. The Vortex comparison calls
`artifact.to_vec()`; Parquet calls `Bytes::copy_from_slice`. The shared
conversion allocates Arrow arrays from owned values and copies strings back
out. This is not a zero-copy batch handoff. The comparison integration file
uses `cfg(all(feature = "parquet-reference", feature = "vortex-reference"))`:
enabling just one reference feature excludes the entire test file. The
comparison example times in-memory encoding/decoding, not disk I/O, point-read
latency, focus rotation, publication or recovery.

The owning F-06 text currently mandates Parquet publication and the columnar
port comments exclude both references from product builds. Those statements
conflict with the owner's new Vortex-primary/Parquet-reference production
backend requirement and must be corrected in the implementation package's
owning records. Neither existing prose nor prior benchmark results override
the new requirement.

## First replacement package: DB-owned cold storage

Proposed implementation ownership is QL, DB and CLD, with TX integration only
through existing stamped publication/commit authority. MX and LSM are protected
dependencies of this package. This is a proposal, not files already authored
or a claim of zero boundary violations in the current tree.

Use one context-aligned cold module beneath the current semantic storage
boundary, rather than creating six empty crates:

```text
crates/persistence/rrd-store/src/cold.rs
crates/persistence/rrd-store/src/cold/
  backend.rs          ColdStorageBackend and format-neutral scan/write contracts
  manifest.rs         immutable generation identity, coverage and ownership
  publication.rs      durable prepare/publish/reconcile through the storage port
  read.rs             stamped, bounded, projected batch reads
  history.rs          canonical history batch schema and semantic validation
  backends/
    vortex.rs         Vortex codec, layout, read/write and format-specific metrics
    parquet.rs        Parquet codec, layout, read/write and format-specific metrics
```

The DB facade owns backend selection, placement and generation publication;
only backend modules know the file format. A backend returns batches and
evidence for a DB-authorized snapshot; it never creates a transaction watermark
or changes valid-time visibility. QL consumes DB scan traits and TX visibility,
not a concrete cold backend. Engine orchestration keeps authentication and
request admission, not paths or compression options. Feature selection must
reject an unavailable configured backend, never fall through to another one.

| Existing behavior and source | Canonical destination | Required preservation oracle |
|---|---|---|
| QL cold page scope/cursor/digest checks, budgets, deadlines and history columns | DB semantic history validation plus CLD backend encoding | Same stamped changes and failures through each backend; no JSON row round-trip on the Arrow scan boundary |
| Engine `cold_history/files.rs` staging, fsync, rename and verification | Storage-owned publication and backend I/O | Interruption at stage-write, sync, rename and receipt commit never publishes a partial generation |
| Engine `state.rs` pending/head/receipt chain and `maintain.rs` replay | DB-owned generation manifest coordinated through TX | Exact recovery, stale-source rejection, corruption refusal and bounded work remain observable |
| Existing comparison codec behavior and tests | Two feature-selected `ColdStorageBackend` implementations and common conformance fixtures | Each feature independently runs nonzero identical cases; combined build can compare answers |
| Current source history retention | DB manifest and TX snapshot/pin lifetime contract | No reclamation until cold lookup, retained snapshots, restart and incomplete-publication cases pass |

Do not implement a second publication path while leaving the engine's Parquet
path active. Replacement includes its production consumer and tests. Query
export is a separate public capability: relocating its codec must preserve its
consumer rather than deleting export to satisfy an import check.

The next tier packages must separate immutable, snapshot-bound working context
from transaction-private scratch and durable commit publication. The owner was
asked whether MX context is disposable while durable effects require WAL
publication before visibility. That clarification remains pending at this
record. No focus eviction, new memory authority or cross-tier visibility policy
is implemented on an assumed answer. A focus signal cannot itself authorize
mutation, weaken isolation, or reclaim history pinned by a reader.

## Acceptance commands and actual results

Environment: Windows, this checkout's target directory
`D:/.usr/Temp/rrflow-root-target`, `CARGO_BUILD_JOBS=4`.
The following commands actually ran on the inspected source. Each exited 1
before compilation because the named feature does not exist:

```powershell
cargo test --locked -p rrd-store --no-default-features --features cld-vortex --test columnar_reference --no-run
cargo test --locked -p rrd-store --no-default-features --features cld-parquet --test columnar_reference --no-run
```

No test executed in either probe. No six-layer benchmark ran. This is a failed
feature-admission check, not a measured backend comparison. The existing
reference comparison can be invoked with its actual names, but was not rerun
in this review because it cannot answer the production tier question:

```powershell
cargo test --locked -p rrd-store --features parquet-reference,vortex-reference --test columnar_reference
cargo run --locked --release -p rrd-store --features parquet-reference,vortex-reference --example columnar_comparison -- --rows 100000 --trials 7 --output D:/.usr/Temp/rrflow-bitemporal-validation/columnar-review.json
```

Documentation verification passed: `python scripts/knowledge/render_navigation.py`
regenerated the one nearest index; `python scripts/ci/check_documentation.py`
validated ownership, classifications, indexes and links; `git diff --check`
reported no whitespace errors. The startup session check failed on the existing
unadvanced Connectome gitlink and the unpushed root follow-up commit; it was
not reported as a clean session. GitHub returned no open root issues when
checked. No new candidate CI was started for the held draft.

The proposed production package must author these common targets before claiming
the following acceptance commands pass. They are planned, not runnable targets
in the inspected revision:

```powershell
cargo test --locked -p rrd-store --no-default-features --features cld-vortex --test cold_storage
cargo test --locked -p rrd-store --no-default-features --features cld-parquet --test cold_storage
cargo bench --locked -p rrd-store --no-default-features --features cld-vortex --bench cold_storage
cargo bench --locked -p rrd-store --no-default-features --features cld-parquet --bench cold_storage
```

Use one deterministic corpus and retained query answers across both arms:
corrections, retirements, overlapping valid intervals, old knowledge cursors,
concurrent pinned readers, selective projections, failed publication, corrupt
artifacts and restart. Benchmark point/projection/full scans, encode/decode,
copied/borrowed/decoded bytes and warm/cold latency separately. A benchmark must
validate answers before reporting times. Neither a successful build nor fewer
copies establishes a latency target.

## Research and remaining limits

The [Arrow format](https://arrow.apache.org/docs/format/Columnar.html) defines
shared-memory representations; it does not supply transactional mutation
coordination. [Vortex arrays](https://docs.vortex.dev/concepts/arrays) distinguish
compressed execution from canonical Arrow-compatible representations.
Consequently the proposed contract must identify which buffers are shared,
which require decoding/allocation and who pins their lifetime. A CPU Arrow
batch does not establish a GPU transfer guarantee. No dependency upgrade,
upstream source adaptation, GPU execution or latency qualification occurred.

The source review and two failed feature probes are the new evidence. Earlier
relation, HTTP and installed-binary results remain scoped to that earlier
package; they do not prove this architecture. Desktop readback of the latest
relation panel, final candidate CI and queue integration remain unfinished.
No files, branches, stashes, archives, databases or user processes were deleted.
The next implementation boundary is production DB/CLD publication and stamped
readback through one backend port, after the grounded package brief and the
needed visibility decisions are settled.

## Reviewed source objects

The table below is generated from Git for the complete files actually examined
in this review. Paths not listed remain unreviewed for this coverage claim.

| Complete file | Git blob at inspected revision |
|---|---|
| [crates/persistence/rrd-store/Cargo.toml](../../../../crates/persistence/rrd-store/Cargo.toml) | `4ca888256f0346449b0716f8f2cc506afe145710` |
| [crates/persistence/rrd-store/src/lib.rs](../../../../crates/persistence/rrd-store/src/lib.rs) | `5431d1b33f64e34426406439bca09260d095035c` |
| [crates/persistence/rrd-store/src/engine.rs](../../../../crates/persistence/rrd-store/src/engine.rs) | `a05820095f8845d7e5018f4ca60c1b8648159475` |
| [crates/persistence/rrd-store/src/transaction.rs](../../../../crates/persistence/rrd-store/src/transaction.rs) | `0daa7b16252e066c322a2ef6cf1fa5cf63451a29` |
| [crates/persistence/rrd-store/src/columnar.rs](../../../../crates/persistence/rrd-store/src/columnar.rs) | `5caba76e5685f344db636d60f894b8dfd4215da2` |
| [crates/persistence/rrd-store/src/columnar_reference.rs](../../../../crates/persistence/rrd-store/src/columnar_reference.rs) | `05379d91470837095e883ab5be22663b93244530` |
| [crates/persistence/rrd-store/src/columnar_reference/vortex.rs](../../../../crates/persistence/rrd-store/src/columnar_reference/vortex.rs) | `20799f0b425844abe38d2cdeb2d7632bbe356778` |
| [crates/persistence/rrd-store/src/columnar_reference/parquet.rs](../../../../crates/persistence/rrd-store/src/columnar_reference/parquet.rs) | `44b4b6368ea539c32e78745dfaaca58a3917ee19` |
| [crates/persistence/rrd-store/src/access/semantic_commit/plan.rs](../../../../crates/persistence/rrd-store/src/access/semantic_commit/plan.rs) | `fe23b861c6dac25ff715812950a7287621bfc85f` |
| [crates/persistence/rrd-store/tests/columnar_reference.rs](../../../../crates/persistence/rrd-store/tests/columnar_reference.rs) | `d7c6ce4b87d87d01991bc3239070d61e40c7bbe3` |
| [crates/persistence/rrd-store/examples/columnar_comparison.rs](../../../../crates/persistence/rrd-store/examples/columnar_comparison.rs) | `2768646f8e889e1da08bafb677c87f97e96c4985` |
| [crates/compute/rrd-query/Cargo.toml](../../../../crates/compute/rrd-query/Cargo.toml) | `0e9e54c39c76f44cd38376f7f9843e921223b99e` |
| [crates/compute/rrd-query/src/lib.rs](../../../../crates/compute/rrd-query/src/lib.rs) | `586e90bb431b88f91b1c9d005a0f28d19ff26eee` |
| [crates/compute/rrd-query/src/pipeline.rs](../../../../crates/compute/rrd-query/src/pipeline.rs) | `2ba3afe5e8a0750477224588eb9c838a3495b306` |
| [crates/compute/rrd-query/src/cold_history.rs](../../../../crates/compute/rrd-query/src/cold_history.rs) | `622c9726da49079254ccb49dafb55386b47e561c` |
| [crates/authority/rrd-engine/src/engine/transaction_admission.rs](../../../../crates/authority/rrd-engine/src/engine/transaction_admission.rs) | `7750d8a02033d202af8b92ca8d86d926caca2682` |
| [crates/authority/rrd-engine/src/engine/cold_history.rs](../../../../crates/authority/rrd-engine/src/engine/cold_history.rs) | `dc676cef8f77fa80f547e4dd466e37cff3a7d1b3` |
| [crates/authority/rrd-engine/src/engine/cold_history/state.rs](../../../../crates/authority/rrd-engine/src/engine/cold_history/state.rs) | `c837450a853caa9b8c9bfdce982f1717f0ecb279` |
| [crates/authority/rrd-engine/src/engine/cold_history/files.rs](../../../../crates/authority/rrd-engine/src/engine/cold_history/files.rs) | `841660adc72621386799ed83e0c2148b7cb42cb6` |
| [crates/authority/rrd-engine/src/engine/cold_history/maintain.rs](../../../../crates/authority/rrd-engine/src/engine/cold_history/maintain.rs) | `fee840afd1391c5277326f7c2d553e24f1a0bce0` |
| [crates/persistence/rrd-lsm/Cargo.toml](../../../../crates/persistence/rrd-lsm/Cargo.toml) | `b67bf599f33463adf846d41c87a8e9d3a18df8f7` |
| [crates/persistence/rrd-lsm/src/database/transactions.rs](../../../../crates/persistence/rrd-lsm/src/database/transactions.rs) | `0424df210803d96830f288e28c4df5068d255ada` |

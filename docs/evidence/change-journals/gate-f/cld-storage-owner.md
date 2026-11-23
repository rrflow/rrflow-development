# Cold storage ownership

**Status:** active candidate evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-f/cld-storage-owner`
**Owner:** F-06 and [POAM-090](../../../poam/rrflow-1.0-alpha/poam-090.md)

Start at `b10cd07b0372f1601de44f20ef816d94f02c3d0d`. The production
maintenance and authenticated generation reader call codecs exported by
rrd-query. Engine code constructs physical filenames. The separate columnar
adapters live in rrd-store.

Move the existing codecs, adapters and their characterization workloads to
`crates/persistence/rrflow-cld`. Give QL and CLD a shared format-independent
Arrow representation. Bind engine publication, verification and restart through
the CLD interface while retaining the same source stamp, receipt chain, digest,
deadline, admission, retry and corruption checks. Preserve explicit query
export. Keep source transactions, authorization and placement in their owners.

Check both format features independently, compare the same projected source
values, and exercise the existing installed lifecycle readback. Do not infer
zero-copy, throughput superiority or equality of backend guarantees from a
successful build. Keep F-06 open for behavior lacking installed evidence.

## Source and replacement

The former query codec in `rrd-query/src/cold_history.rs` is now split between
the CLD stamped-row validator and its Parquet reader/writer. Bounded query
interchange moved from `rrd-query/src/parquet.rs` to CLD. The Arrow row, field
types and stamped batch conversion have one owner in `rrflow-arrow`, imported
by both crates. Neither CLD nor its Arrow dependency imports the planner.
The former store columnar table, projected format adapters and comparison
workload are in CLD. Existing error identities, corruption checks, empty pages
and read limits moved with their implementations.

The engine still owns source reads, authorization, durable receipt chaining
and the transaction gate. It now asks a `ColdStorageBackend` to encode and
decode. A pending generation carries its format and authenticated stamp.
Reopen selects the codec from the receipt. The installed policy selects the
writer and the engine admits its availability during configuration. The debug
event records source manifest, format, bytes read and decoded, and decode time.

## Execution evidence

On Windows, `cargo test -p rrflow-cli --features cld-vortex --test installed_lifecycle -- cold_history_publishes_and_reads_after_process_restart` ran against actual child
processes. Initial verification ran a Parquet case and a Vortex case, both
passed. The final selector exercises both settings in the owning installed
lifecycle case. The driver commits a document and correction, authenticates
to the serving process, reads generations, terminates it, starts the same
installation again and checks the original changes and receipts.

The first Vortex stamped-generation read returned an integrity error because
the native reader did not request the metadata segment. The corrected open
explicitly includes metadata. The first independent Parquet test build also
exposed a missing trait import in the moved footer-failure characterization.
Both failures were retained here instead of weakening their checks.

Independent backend acceptance commands, using the same 5,000-row corpus:

```sh
cargo test --locked -p rrflow-cld --no-default-features --features cld-parquet
cargo test --locked -p rrflow-cld --no-default-features --features cld-vortex
cargo bench --locked -p rrflow-cld --profile dev --no-default-features --features cld-parquet --bench columnar -- --rows 5000 --trials 3
cargo bench --locked -p rrflow-cld --profile dev --no-default-features --features cld-vortex --bench columnar -- --rows 5000 --trials 3
```

Docker ran the repository's selected-test resolver and the 12 inventory checks
with the checkout mounted read-only. The timed workload measures in-memory
column encoding and projection. It does not measure filesystem point reads,
mixed-tier QL planning, focus rotation, GPU handoff or a production crossover.
F-06 and POAM-090 remain open.

Both feature-isolated CLD suites passed. On x86-64 Windows, the three-trial
development-profile medians for the 5,000-row columnar workload were:

| Format | Encoded bytes | Encode ms | Full decode ms | Two-column decode ms |
| --- | ---: | ---: | ---: | ---: |
| Parquet | 134396 | 42.392 | 16.418 | 4.247 |
| Vortex | 199812 | 492.934 | 16.528 | 5.046 |

The combined run also verified that each codec rejects the other codec's
artifact. The 14 engine cold-history checks passed, including corruption,
restricted reads, changed configuration, writer contention and termination at
publication boundaries. The first GitHub candidate rejected the journal status
and the old invariant requiring Vortex to belong to rrd-store. The revised
invariant binds both physical libraries exclusively to CLD and excludes QL
and all physical hot storage from CLD's production dependencies.

A separate Python SDK run then installed two fresh projects with the real
executable (SHA-256
`62cacbd21e0d22811829504f49bb68a33d1701721733d1d1cf2f0e46fbbed2b1`).
Each committed the original document at cursor 4 and its correction at cursor
5 with different valid-from times. Authenticated reads after process restart
matched every original source manifest, artifact digest and mutation. Parquet
published two generations and Vortex three, reflecting maintenance timing. The
comparison used each project's own receipts. The local proof directory is
`rrflow-cld-readback-f8952c9d47ee4cdbb7728025527d7bdb` under the temporary
directory. Its readback.json contains the results without session credentials.

The workspace architecture suite passed all 34 checks with the corrected
ownership rules. Strict Clippy passed for both the Arrow and CLD crates with
all features and targets. The generated SDK and signal surface check reported
88 HTTP operations and verified seven projections. No alpha gate was closed.

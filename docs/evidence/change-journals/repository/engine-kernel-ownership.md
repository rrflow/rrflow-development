# Engine ownership of the temporal kernel

**Status:** installed development binary exercised on Windows
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/engine-kernel-ownership`
**Owner:** single-engine prerequisite for the [kernel contract](../../../roadmap/kernel-contract.md)

Baseline commit `5fe635b20605606e052cdccd2160a03f9e64143e`, repository
tree `1a5b8961b82d501627b7f639ab714f357385a7b7`, original kernel tree
`f77f46260bb69d71372b7d7e5d3b42e2bc321d53`. The change brief and the
candidate and merge records belong to [PR 500](https://github.com/rrflow/rrflow-development/pull/500).

## Ownership and source review

The original package at `crates/kernel/rrd-core` supplied shared values and
invariants to the planner, storage, and engine. It had no server entry point.
All 61 tracked files moved under `crates/authority/rrd-engine/kernel`.
`rrd_engine::kernel` exports the same Rust types used by storage and query.
They can depend on the nested library without depending on their caller,
`RrdEngine`. The engine instance implementation moved from `engine/core.rs`
to `engine/instance.rs`. Package membership, downstream dependency edges,
compiled consumers, generators and current source references follow the move.

The production review covered the kernel manifest and these source files,
listed by their original paths relative to `src`. Standalone test modules,
fixture contents and inline test bodies were outside the production review.

| Directory | Rust modules read |
|---|---|
| Root | lib, authenticated_log, claim, data, digest, error, ident, reasoning_tree, reference, result_identity, runtime, schema, telemetry, temporal, trace |
| authenticated_log | accumulator, generation |
| runtime | audit, data_snapshot, encoding, graph, ids, model, projection, referential, stamps, transaction, validate |
| runtime/referential | timeline |
| schema | organisation, registry, types, validation |
| trace | convergence, event, ids, validation, vocabulary |

The engine review followed the package manifest, public composition root,
instance construction, transaction and trace callers. Two rules had duplicate
implementations. Trace events and standalone trace installation now use
`RuntimeTraceEvent::schema_update` for schema registration, revision
increments and overflow rejection. The engine retains its cursor comparison
and retry policy. Schema encoding now calls the existing
`runtime_logical_model_tag` used by retirement encoding. Both inputs previously
mapped the same model variants to the same bytes. Transport retains its
independent wire types and the frozen reasoning-tree wire fixture.

Removed the Rust test that repeated every package path listed by Cargo and
enumerated top-level directories. Removed the shared assertion that equated
package names with directory names. The Cargo manifests own membership.
Architecture checks still inspect actual dependency edges and outward imports.
This work involves no external algorithm adaptation or change in tier policy.
Existing durable trace events and public commit receipts supply the observations.

## Installed engine evidence

The first executable build reported three references to the former instance
module in tail repair. After updating the callers,
`cargo build --offline --locked -p rrflow-cli --bin rrflow` succeeded.
`cargo tree --offline --locked -p rrflow-kernel --edges normal --depth 1`
reported exactly `serde` and `sha2`. The installed binary SHA-256 was
`d94e9a0913576072d0de1b0c156009a38f82f9027492728af2ea9a181a58c2e6`.

The installed flow used this checkout's `scripts/dev/bitemporal_proof.py`
and Python SDK. It planned and applied installation in a fresh project,
started a loopback server, made transactions, killed the process, and reopened
the same project. To repeat that part with an unused output directory:

```powershell
python scripts/dev/bitemporal_proof.py --engine target/debug/rrflow.exe --output "$env:TEMP/rrflow-kernel-proof-$([guid]::NewGuid())"
```

| Actual operation | Before and after process termination |
|---|---|
| Read document alpha at valid time 100, known cursor 6 | Original |
| Read the same document at valid time 100, known cursor 7 | Corrected |
| Read the same document at valid time 99, known cursor 7 | Zero rows |
| Read beta and its edge at known cursor 10 | Both present at valid time 149 and absent at 150 after cascade |

On the same installation the retained eight-client driver opened eight
sessions and eight data transactions at cursor 10. Each previewed a different
document, then all clients submitted commits from a barrier. Each received an
acknowledgment, with eight distinct cursors from 11 through 18. After another
forced termination the public query returned worker-0 through worker-7 at
cursor 18. The driver checked the exact returned titles against the committed
set. The host retained the full responses in `evidence.json` and
`multiwriter.json` under `rrflow-kernel-500-09e4d3520f5e48abba529c11e5278936`
in the temporary directory.

Local test suites and benchmarks were skipped per the owner's direction.
These runs exercise single-node behavior on this Windows executable. They
establish neither distributed transaction guarantees nor cold-tier or UI
coverage. No objective, roadmap or POA&M status changed.

# Lineage: Vyrm to RRFlow, rrflowLSM, rrflowQL and rrflowDB

**Status:** historical record; claims only what is stated per row
**Coordinate:** `rrflow://rrflow-instance/data/architecture/rrflowlsm-lineage`
**Superseded by:** [rrflowKV current format](../reference/storage/rrflowkv-current-format.md)
**Owner:** the mapping from the earlier product name to current names, and what was read to confirm it

The owner's statements, 2026-11-15 and 2026-11-17: Vyrm is the earlier name of
this engine, and its storage layer `vyrm-kv` is what is now **rrflowLSM**.
Vyrm was renamed during pre-release convergence; no compatibility name is kept
in code. This record exists so the earlier name resolves to the current one in
one place. The name appears in this repository only in the records the
architecture test lists for it (`LINEAGE_RECORDS` in
`crates/authority/rrd-engine/tests/workspace_architecture/retired_and_inputs.rs`);
anywhere else it is a violation.

The original migration control document, with its full target map and rules,
is kept in Git history at `3fc2842d5^:docs/rrflow-vyrm-migration-ledger.md`.
Its rationale stands; its `rrflow-*` crate targets were not used. The crates
that exist are the `rrd-*` ones below.

## Names

| Earlier | Now | Read on 2026-11-17 |
|---|---|---|
| Vyrm (the engine) | RRFlow (Reason Ready Flow) with RRD (Reason Ready Daemon) | the old ledger's product map, against the current workspace |
| `vyrm-kv` | **rrflowLSM**, crate `persistence/rrd-lsm` | **verified**: format, magics, versions (below) |
| VyrmQL, `vyrm-ql` | rrflowQL, crate `compute/rrd-query` | **verified** by capability: tokenizer, planner modes, filter algebra |
| `vyrm-vector` | crate `compute/rrd-vector` | **verified** by capability: HNSW, TurboQuant, sparse, MaxSim, payload indexes |
| VyrmMX (`vyrm-mx`) | folded into the one query contract | decision 2026-11-14; the test oracle remains |
| `vyrm-core` | `kernel/rrflow-kernel` | mapped by role, not read |
| `vyrm-embed` | `compute/rrd-inference` | mapped by role, not read |
| `vyrm-edge` | `adapters/rrflow-edge` | mapped by role, not read |
| `vyrm-cluster` | `operations/rrd-cluster` | mapped by role, not read |
| `vyrm-cli` | `adapters/rrflow-cli` | mapped by role, not read |
| `vyrm-eval` | `evaluation/*` | mapped by role, not read |
| `vyrm-operator`, `vyrm-graph`, `vyrm-node`, `vyrmd` | counterpart not identified | **open**: read before claiming |

## What was read for rrflowLSM

The earlier format contract named every on-disk magic with the old prefix. The
code now uses the new prefix on the same structures; the segment and footer
versions have advanced (03 to 06):

| Structure | Earlier | Now |
|---|---|---|
| Write-ahead log file | `VYRWAL01` | `RRDWAL01` |
| Record | `VYR1` | `RRD1` |
| Batch payload | `VYRBAT02` | `RRDBAT02` |
| Segment (current) | `VYRSEG03` | `RRDSEG06`; the reader also accepts `RRDSEG01` to `RRDSEG05` |
| Segment index footer | `VYRIX003` | `RRDIX006` |

The recorded vector-search evidence `docs/evidence/m5-vector-local-10000x128.json`
differs from its earlier version in one value only: its `schema` string,
`vyrm.vector-evidence.v1` then, `rrflow.vector-evidence.v1` now. Every measured
value is identical.

## Not carried

- The earlier native-versus-Fjall harness was removed with its dependency; its
  old store could not be ported, because semantics and Fjall were interleaved.
  The comparison is restored on the current boundary as the optional
  `fjall-baseline` feature of `rrd-store`; see
  [fjall-baseline-20261117](../evidence/change-journals/repository/fjall-baseline-20261117.md).
- Vyrm-era documents with no counterpart by name (33 were found absent) were
  not each read. The database pillar is not fully verified.

## Open

1. The earlier native application identity `VYRSK002` (the keyspace encoding
   of `rrd-store`) has no confirmed counterpart; `RRDSNP01` is a different
   structure (the snapshot bundle file) and is not its successor.
2. Read the six crates marked "mapped by role" and the four with no identified
   counterpart.

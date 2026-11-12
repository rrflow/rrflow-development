# POAM-054 — a default install accepts a vector corpus its search then refuses

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-054`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

Found by the [comparison lab](../../../eval/workbench/lab/README.md) on `clyffy-01`, on an
engine built from `3a72a6b4adc43c842f67a59f4fc91d725eac7580`. An installed engine
on default settings accepted all 200,000 points of 384 dimensions, then refused
every vector search and every TurboQuant build:

```text
StorageAdmission("projected-read output_buffer_bytes limit 134217728 would be exceeded by 134221074")
```

The same defaults serve 20,000 points and refuse 100,000. The exact size where
refusal starts has not been measured. An exact search or a build reads every stored
vector version into one output buffer: `scan_source` in
`rrd-store/src/access/versioned_read.rs` scans the whole range with no row limit,
and `projected_output_bytes` caps that buffer.

Writes give no warning as a corpus nears the limit, and the refusal arrives as a
429 on the read. `projected_output_bytes` is sealed at installation. Editing an
installed `.rrflow/config.toml` makes `serve` fail with `ProjectBindingMismatch`,
so the value can be chosen only through `install plan --configuration`.

A higher value does not make every size searchable. With the limit raised to 16 GiB,
and with [PR #259](https://github.com/rrflow/rrflow-development/pull/259)'s fix for
the second ceiling it then met, the same 200,000 points are still refused:

```text
ConfigurationLimit("runtime versioned read exceeded key budget 1000000: observed 1000001")
```

`MAX_VECTOR_STORAGE_KEYS` in `rrd-contract` is a public contract ceiling on vector
searches and on index and quantization builds, and the lab already requests that
maximum, so no setting lifts it. The budget counts scanned rows and the accumulator
proof nodes read to authenticate them, so a read reaches it well before the corpus
holds a million vectors. A read of 100,000 points stays inside it. The size where
refusal starts has not been measured.

## Impact

A project that grows past the limit keeps writing vectors it can no longer search,
and has no supported way to raise the limit on its existing installation. Choosing
a higher value at install time does not remove the cost. With the limit raised to
16 GiB the engine reached 6.3 GiB of resident memory at this size before a second
ceiling refused the read, and a project that grows past the key budget cannot search
its vectors or build an index over them at any setting.

## Owning gates

E-04 and OBJ-13 (vector search and TurboQuant builds that serve what the engine
accepts), and D-06 (governed configuration changes after installation).

Roadmap owners: [Gate E](../../roadmap/rrflow-1.0/gate-e.md) and
[Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

A test through an installed engine on default settings shows that a corpus at a
stated size is searchable and that a TurboQuant build over it completes, without
one read buffering the whole corpus. Any size the defaults cannot serve is either
refused at write with the limit it would break, or served after a governed
configuration change whose test shows existing state surviving it. No corpus the
engine accepts is left beyond every setting by the key budget.

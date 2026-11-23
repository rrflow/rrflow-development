# Cold storage ownership

**Status:** candidate evidence
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

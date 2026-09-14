# KB-05 journal: local-estate-authorization-v1

**Status:** active immutable change-journal evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/kb-05/local-estate-authorization-v1`
**Owner:** the unchanged historical package receipt identified by its legacy heading and payload digest
**Legacy source:** `docs/roadmap/rrflow-1.0-execution-map.md@b7b061900b67d535ffec2710e6e4c30815d0f216#L1140`
**Legacy payload SHA-256:** `2616e74daef9aacd58ab05f7a807e742850f2346eef33461f4d119922857613d`

This record refracts one completed package receipt out of the former
monolithic execution map. Its fenced payload is preserved byte for byte.
It reports historical evidence and cannot change roadmap completion or
POA&M lifecycle status. Follow the [canonical roadmap](../../../roadmap/rrflow-1.0.md)
for acceptance and the [change-journal index](../) for discovery.

## Journal payload

```text
gate/package: A-06 / KB-05 / local-estate-authorization-v1
revision: parent a8dad0c; result is the commit containing this entry
baseline files/digests: docs/local-estate-authorization-v1.md=cc0e13b0701d790eacff05b3e5de57cceb02238a9133905b0bb712b85e1c57ff; rrd-estate/local_authorization.rs=02054dacb1b4a6aa06e68509859f0a2a75df4453f797df9ffe62233915700f42; rrd-estate/lib.rs=37d585386aff6152f6021d6e978a2288aa733d476e69719f1de85225736cfeab; rrd-estate/backup_job.rs=35659494cfeb35328c0bc419e400fdbea06528f0f5b048e21c1197caa56a9816; rrd-estate/recovery.rs=6d22f4dedbf9a15965357aab533968007a1b1392988d12ba12157b224604a822; engine/estate_control.rs=3c3eb4684b50144958240ac252c9f9a3bfdc4c58ab819cc1528f1646d4547d61; rrd-estate-admin.rs=10971201428c8f2b9d36cc4f7f2bb18a14661c00d14a31dacb62c2da499a88ba; rrd-recovery-controller.rs=f19ddee8f5c9c67bb156038ac66647208e0e2c463d463252ff1f9330ccd6b2e1; estate_admin.rs=876fecbddf01a8e4c2fa0eaf44714d7d9caeb1307ecefc5c97cd9ec59cb5a93a; engine_authority.rs=01b5f3c109db3079bccc89cce37ac6a21486f6fd3444bce3826f0d5476eb9e65
files read in full: root README; flat local-estate authorization record; current security index and authority; research index; rrd-estate manifest, public root/aggregate repository, local authorization, backup-job, and recovery implementations plus complete local-authorization, backup-job, and recovery tests; rrd-engine manifest/public root/composition root/core/estate-read/estate-control implementations and complete engine-authority test; rrflow-cli manifest, complete estate-admin and recovery-controller adapters, and complete estate-admin process test; prior complete system-overview, ADR, objective, roadmap, POA&M, canonical invocation, public contract/test, and workspace-architecture reviews reused only after unchanged hashes and relevant spans were revalidated
files changed/created/deleted/moved: create docs/reference/security/local-estate-authorization.md; update the security and research indexes, security authority, POA&M, this resolved-review/queue/journal, and generated file inventory; delete docs/local-estate-authorization-v1.md; no Rust, public contract, fixture, engine, estate, CLI, storage, SDK, or runtime file changed
contract or behavior changed: none; the canonical reference makes locality an adapter property, retains the seven real effect distinctions, and requires their direct absorption into one installed, stamped, canonical security/invocation/transaction path
smallest test command and result: cargo test -p rrd-estate --test local_authorization --locked — 1 passed before editing and 1 passed after editing
owning package command and result: cargo test -p rrd-estate --all-targets --locked — 22 passed before editing and 22 passed after editing; cargo clippy -p rrd-estate --all-targets --locked -- -D warnings — passed
cross-boundary command and result: cargo test -p rrflow-cli --bin rrd-estate-admin --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrflow-cli --test estate_admin --locked — 1 passed before editing and 1 passed after editing; cargo test -p rrd-engine --test engine_authority --locked — 3 passed before editing and 3 passed after editing; cargo test -p rrd-engine --test workspace_architecture --locked — 16 passed; deterministic inventory reported 753 records; documentation policy reported 87 statuses and 64 classified coordinates; generated-surface parity reported 33 HTTP operations and OpenAPI e0b107bc875dc5318d90b518993023730c83c475d323e69ea54a747050e86715; workflow policy, frozen 1.0.0 version policy, Ruff, Cargo formatting, and diff checks passed
failure/crash/differential evidence: the passing engine test proves canonical security can omit `EstateAdmin` while the separate local policy still authorizes estate creation; POAM-017 records that dual authority. The owning suite also requires successful older estate documents and backup jobs with missing fields; POAM-018 records that pre-release compatibility path. Existing tests cover selected reopen/replay and fenced restore behavior, but this package creates no new crash, cross-surface, MX/KV authorization differential, or resource evidence
not run and reason: full rrflow-cli/engine/workspace suites, server/client/SDK conformance, every permission denial, credential/file race and platform ACL matrices, install resolution, DataFusion, reasoning, and release qualification do not prove a documentation-only KB-05 classification and remain owned by their named gates
remaining known errors: 21 KB-05 records remain; A-06/A-07 are incomplete; POAM-016 through POAM-018 remain; local estate mutation is not one canonical authorized operation and current estate decoding still accepts older successful shapes
roadmap checkbox changed: no
```

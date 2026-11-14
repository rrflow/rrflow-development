# D-03 non-following filesystem leaf classification

**Status:** active partial implementation evidence; Inventory execution remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-nonfollowing-leaf-classification`
**Owner:** OBJ-14 / D-03 / POAM-006; PR224 driver: active SWAG implementation session

## Bound change and advancement

Starting revision `806813a9a1c1d309a655e8d0f4352aa864346f0a`, tree
`859436d837fa2e8ea47647ac0fb49ec8e1f081b6`, on
[PR224](https://github.com/rrflow/rrflow-development/pull/224). The exact native
source/export/test brief was published before edits. PR223 landing and PR221
selected-plan integration remain prerequisites of the complete package.

The existing filesystem authority rejected links, mounts and special objects,
preventing the engine walker from recording their non-followed leaf kinds.
`AnchoredPath::source_leaf` now performs no-follow metadata inspection relative
to a retained, admitted parent. Ordinary leaves retain strong native observations;
links (including Windows junctions), mount crossings and special objects return
explicit classifications without target paths or content. Parent alias and
hardlink denial remains in force. Classification does not authorize traversal.
The existing open/read/write/source-observation behavior is unchanged.

Changed paths: `rrd-lsm/src/filesystem/source.rs`, its export in `filesystem.rs`,
and the existing `tests/filesystem.rs`, plus this journal and generated index.
No implementation is moved or deleted; no dependency, configuration authority,
public operation, runtime callback or model behavior is introduced.

## Review, decisions and validation

Reviewed the changed source and complete diff; retained-root, parent-chain,
metadata/enumeration and native mount/object helpers; source observation capture;
affected actual-file, junction and native mount tests; and D-03 traversal owner
requirements. Primary implementation reference: the already pinned
`cap-primitives 4.0.3` no-follow `stat` and metadata code in the local Cargo source
cache. No upstream code is copied and no ambient path or link-target read is added.
No benchmark/new runtime tracing is required for this classification boundary.

The first test compilation failed with the absent `SourceLeaf`/`source_leaf`
interface. After implementation on Windows:

- `cargo test -p rrd-lsm --test filesystem --locked`: all 11 passed, including
  actual junction classification and continued denial of reads/metadata below
  that junction; ordinary file/directory observations and hardlink/escape denial.
- `cargo clippy -p rrd-lsm --test filesystem --locked -- -D warnings`: passed.
- Format, navigation regeneration, documentation policy (817 statuses / 815
  coordinates), execution inventory generation/check (2081 records), and staged
  diff checks passed.

Tests use actual authored filesystem fixtures, not model evaluation or training
records. Unix dangling links, loops and sockets have an explicit no-follow test.
Existing Linux private bind-mount and macOS mounted-volume tests gain a leaf-kind
assertion before their existing denial checks. These platform-specific cases were
not executed on this Windows host; their native CI evidence remains required.
The source is not a complete snapshot or proof of race-free traversal.

Retained local evidence under `20261030/engine-source-inventory/`:

- `leaf-first-oracle.log`: `824c5b10e98014fc2caea6a716559f9281fbc9be32c1813ff56eda9314609b0f`.
- `leaf-filesystem.log`: `d9b797503555a1f6a8e603298887904ae54c1b0df582a2b6de6458523870be48`.
- `leaf-clippy.log`: `a9b0128e72e757af0a7c7d44bc8e4058f2d9f41b254be5f7b2daab33169244f8`.
- `leaf-results.json`: `88eabdaf43cf7255de5270d291a314a9f5e8673d7c6d20b3f629ab9d0fc24ec3`.

- `leaf-repository-results.json`: `0291c02b8709743f97dd74f2605fc36b9acfe944b5e7326587fbdf5a3c021ae7`.

## Remaining scope and checklist

The brief, owner boundary, first compile failure, actual-file oracles, source
review and resource/research decisions are recorded. Stop on any classification
that relaxes read/traversal authority, leaks target content, or bypasses admitted
parents. No objective/roadmap/POAM status changes and no candidate/merge acceptance
for PR224. Native cross-platform qualification and the actual bounded engine walker,
validated prior snapshots, canonical source publication and installed edit/restart
proof remain open. No new binary, provider request, training or workflow completion.

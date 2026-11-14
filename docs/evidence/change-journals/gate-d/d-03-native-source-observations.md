# D-03 native source observation prerequisite

**Status:** active partial implementation evidence; engine inventory remains incomplete
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-native-source-observations`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), D-03 / [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md)

## Package and current scope

[PR224](https://github.com/rrflow/rrflow-development/pull/224) owns the full
engine-inventory integration, not just this helper. Driver: SWAG active
implementation session. Branch from public main
`a76faf7046b48819a06c31219e9db973b7f6a06b`; ordinary merge
`da5662e97111b82b3669b2556cd3d0ad6545a27f` incorporates PR223's pure proposal
prerequisite at `a13ce981db6b25c358ccce6cc8afd83fba09d025`. The complete PR brief
was published before these edits. Both dependencies and final candidate CI
remain required; PR221 overlaps installation/plan admission and must be reconciled.

This record freezes the first implementation increment only. No inventory worker,
Git policy, installed snapshot, runtime checkpoint or user task execution is added
by it. Later integration receives its own evidence record; this receipt cannot
be upgraded into that proof.

## Change brief and reviewed boundary

Existing anchored paths retain no-follow and mount authority. Their metadata API
returns standard metadata; the private immutable-file identity uses object ID,
length and modification time for storage-object reopen. That private storage
identity is unchanged. Source reuse additionally needs mutable-source observation
and explicit timestamp-race policy.

Created `rrd-lsm/src/filesystem/source.rs`; changed `filesystem.rs` only to declare
and export its source types; added four tests to `rrd-lsm/tests/filesystem.rs`.
The new source observation captures native object/volume identity, size, kind,
modification and change timestamps, attributes/mode, link count and applicable
Unix ownership. Windows ownership is explicitly absent rather than fabricated.
Versioned kernel SHA-256 produces a revision digest. Absolute paths and file
contents are not part of it. Two consecutive observations must match; supplied
read handles must match the currently admitted path. Symlink, mount, hard-link,
special-file and lexical escape denials use the existing anchored authority.

The timestamp-age helper rejects zero margins and recent/future observations.
The engine must supply a justified filesystem resolution margin, a usable clock
boundary and matching prior/current observations before cache reuse. Unknown or
ambiguous metadata requires content reads. These primitives do not claim an atomic
filesystem snapshot or detect a privileged attacker rewriting all metadata.

Reviewed owner sections: README/AGENTS chain, complete architecture inventory
section, Gate D D-02/D-03, POAM-006, change-authoring procedure and PR221's statement
that step descriptions are not execution configuration. Existing implementation
sections reviewed: anchored open/mount/metadata/file validation, native immutable
identity, installation profile and phase construction, attunement control and
`RuntimeRepository::commit_with_control_transitions`. Complete newly authored
source and changed test functions were reviewed; the whole large filesystem/native
and test files are not claimed as full semantic reads. No code was moved/deleted.
No dependency, installed profile, default source policy or public operation changed.

## Research and resource decision

Primary references:

- [Git timestamp races](https://git-scm.com/docs/racy-git): metadata equality can
  be ambiguous; recent/coarse observations need content revalidation.
- [Windows basic information](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_basic_info),
  [native file identity](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info)
  and [handle information API](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getfileinformationbyhandleex)
  informed bounded native structure reads. The initially attempted `fileapi`
  documentation URL returned 404; the linked `winbase` page is the correct owner.
- [Rust Unix metadata](https://doc.rust-lang.org/stable/std/os/unix/fs/trait.MetadataExt.html)
  owns device/inode, mode/ownership and nanosecond mtime/ctime fields.
- [Git ignore rules](https://git-scm.com/docs/gitignore) and gitoxide pure
  buffer/index/config APIs were investigated for the next increment. Cargo
  metadata confirmed available `gix-index 0.58.0`, `gix-ignore 0.24.0` and
  `gix-config 0.62.2`; these were downloaded as development research only.
  No such dependency is yet selected, linked or added to Cargo.lock.

No upstream implementation is copied. Native calls use fixed-size structures;
metadata observation reads no content and does not move the supplied cursor.
There is no benchmark, scale or zero-content-read engine claim. The timestamp
edit test uses one 30ms interval to characterize this filesystem, with no
retry-until-green loop. Coarse filesystems must use conservative engine fallback.
The tests use actual temporary files and retained handles, not modeled filesystem
responses or model-training data. No credentials, provider or project file was used.

## Commands and actual results

Windows, with `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2` and an explicitly
selected existing development Cargo target cache. This does not create a runtime
or build-source dependency on a sibling checkout.

- `cargo test -p rrd-lsm --test filesystem --locked source_observation -- --nocapture`:
  four passed, six filtered; build 12.43 seconds, tests 0.04 seconds.
- `cargo test -p rrd-lsm --test filesystem --locked`: ten passed, zero failed or
  ignored; includes existing junction/substitution, retained-root and publication
  ownership cases. Tests 0.53 seconds.
- `cargo clippy -p rrd-lsm --lib --test filesystem --locked -- -D warnings`:
  passed, 7.08 seconds.
- `cargo fmt -p rrd-lsm`: applied standard Rust formatting.

All first focused source tests passed. No test failure was suppressed. Native
Linux/macOS, mount-image qualification, full storage suite, installed integration,
Git/secret policy, cancellation, zero-content-read full traversal, atomic
publication and real-repository restart/replan are not run or complete here.
Candidate PR223 is separate evidence for the proposal component and cannot
validate these unmerged changes. PR224 stays draft while its full outcome is open.

Raw local evidence under `20261030/engine-source-inventory/`:

- `native-source.log`: `32cb41866322fcd526fa8162b48a9ba41c0fe95a61cf823c2f9a1b6d089cf914`.
- `source-clippy.log`: `e75df16a38cccf4e4d341c969d7d5074e35eb06adb0c68638d52802fa96a8934`.
- `native-results.json`: `a2daeb35bc5bf903058bd1e077dd7897a6765f088218579175531d5e24f139b4`.

## Checklist and next action

The brief binds full engine integration and exact owner boundaries; this partial
receipt records only the native prerequisite. Research and resource decisions,
reviewed sections, implementation paths, actual commands and remaining errors are
explicit. Regenerate the nearest journal index before commit and retain final
format/diff results. Final `cargo fmt -p rrd-lsm -- --check` and
`git diff --cached --check` passed; documentation policy accepted ownership,
813 statuses, 811 classified coordinates, parent indexes and local links. The
complete new source and authored test diff were reread before publication.
No roadmap/POA&M completion status changes.

Next implement the admitted versioned inventory policy, bounded metadata/content
walk and Git decision inputs, then compose source records/containment/change sets
with the attunement checkpoint using the existing atomic runtime/control commit.
Do not activate Inventory on helper success. Full source identity/replan semantics,
source safety and installed proof must remain the same PR's acceptance boundary.

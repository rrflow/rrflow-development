# D-03 bounded Git inventory inputs

**Status:** active partial implementation evidence; full engine inventory remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-git-inventory-inputs`
**Owner:** OBJ-14 / D-03 / POAM-006; PR224 driver: active SWAG implementation session

## Bound change and advancement

Starting revision `ef029fde4c98311b637e548f0304a5a261fa3dd1`, tree
`81e3455815301d709c05683b79bf5a68b47dd429`, on
[PR224](https://github.com/rrflow/rrflow-development/pull/224). The published brief
was extended before these edits with exact parser/test/dependency boundaries.
PR223 remains a landing prerequisite; PR221 overlaps later installation changes.

Previously the pure proposal crate could not interpret admitted Git metadata.
`rrd-attunement/src/git.rs` now parses supplied index bytes, verifies nonzero
checksums, retains tracked-path/kind facts with explicit ASCII case handling,
and evaluates explicitly supplied global/repository/nested ignore rules. It
preserves parent exclusions and tracked-descendant discovery. The engine must
apply safety exclusions before using these facts; this module does not itself
admit reads or combine tracking with safety policy. Missing inputs are not
implicitly discovered. Split/sparse indexes fail explicitly until their extra
inputs have an authorized implementation. No partial path set is returned.

Index metadata/object IDs never establish working-file content freshness.
Index and canonical ignore inputs receive kernel SHA-256 identities. Diagnostics
return redacted classes rather than upstream path/pattern errors. Input bytes,
entries, source/pattern counts, path/depth and decoded path bytes are bounded.
A length-only Git-format preflight bounds v4 expanded paths before the upstream
decoder can grow its backing buffer beyond its initial estimate. Decoder threads
are limited to one. Allocation guards are not aggregate resident-memory or
elapsed-time measurements; engine admission must account for collection overhead,
combined buffers, duration, leases and cancellation.

Changed paths: crate manifest/lib, new Git parser/tests/fixture generator and
fixture directory, Cargo.lock, the existing workspace dependency-boundary test,
this journal and generated Gate D index. No first-party implementation is moved
or removed. No installation/profile/public contract or provider code changes.

## Research, review and adaptation

Primary references: [Git index format](https://git-scm.com/docs/gitformat-index),
[Git ignore rules](https://git-scm.com/docs/gitignore), and pinned gitoxide source
for `gix-index 0.58.0` decode/options/entries/verification and `gix-ignore 0.24.1`
buffer parsing/matching. Direct dependencies are pinned to those versions and
`filetime 0.2.29`, needed for the parser's internal timestamp type. Its epoch
bookkeeping value is neither an observed timestamp nor a cache-validity claim.
Cargo.lock adds 34 third-party packages, removes none and changes no existing
package version. Existing jiff dependency edges gain timezone platform inputs
and defmt version disambiguation. Gitoxide filesystem/discovery APIs are not
called; there is no runtime Git command, environment discovery or network fetch.
The length preflight is derived from Git's wire format, not copied source.

Reviewed owner sections: README identity/authority, AGENTS, complete change-authoring
procedure, D-03 project-tree requirements and the existing PR brief. Reviewed
installation profile identity/limits to retain its distinct precondition role;
no change to that role is claimed. Complete new parser, focused test and fixture
generator source, manifests, full authored diff and dependency changes were
reviewed. The large existing architecture test was reviewed at changed dependency
and target-tracking boundaries; no full-file semantic review is claimed.

Fixtures are authored parser correctness inputs generated with the recorded Git
version, not real task data, routing evaluation or training records. The committed
manifest binds six actual Git index files and 46 `git check-ignore` observations
across case modes. An additional sparse marker is a deliberately constructed
format-rejection case. Generator scratch repositories were retained locally;
ordinary Rust tests read committed fixtures and do not launch Git.

## Validation and surfaced failures

Windows with `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2`, and an existing
explicit development target cache. No sibling source dependency is introduced.

- Initial check failed because `FileTime` has no `Default`; corrected with the
  explicit pinned type. The failure log retains PowerShell stderr presentation.
- Initial focused suite: 11 passed, one failed. The unsafe-path mutation also
  broke index ordering, so it failed earlier as `InvalidIndex`. The fixture was
  corrected to preserve ordering and exercise `InvalidPath` directly.
- Initial documentation check rejected the journal coordinate because it omitted
  the canonical `/data/` path; corrected before publication.
- Initial architecture suite: 31 passed, one failed because the new Cargo test
  target was still untracked. Staging the actual sources fixes clean-checkout
  inclusion; the guard was not relaxed.
- `cargo test -p rrd-attunement --locked`: 33 passed (14 Git, 19 inventory),
  zero failed/ignored. Includes actual index versions 2/3/4, SHA-256, checksum
  corruption, truncated input, split/sparse rejection, compressed path bounds,
  ignore precedence/negation/case and all 46 recorded Git decisions.
- `cargo clippy -p rrd-attunement --all-targets --locked -- -D warnings`: passed.
- `cargo test -p rrd-engine --no-default-features --features edge --test workspace_architecture --locked`:
  all 32 passed. This checks workspace metadata/source boundaries, not engine execution.
- Strict Clippy for that same architecture target passed.

Raw retained local evidence under `20261030/engine-source-inventory/`:

- `git-initial-check.log`: `281784ea2f287a0120d2beddd3414c2346466c61e5bf865068a9a6ba340c3e99`.
- `git-first-tests.log`: `0368314ae988a10fa43bdf8800eeadee5787ff88e09a8935ddfc0a3c69248104`.
- `git-architecture.log`: `3044cdea6e597092e16ce8a35beaed9386fa67625bfa0859807cb3ca08bae125`.
- `git-final-tests.log`: `187cb0a59ef27a286406d5dfd5d26340992d1bd4f36e6df1ceb6a597450d4c9e`.
- `git-final-clippy.log`: `e8d2998b3f7f2ddf0db3509ee8698cf2cf151a0eb4c0f1cd0e387a17252d7459`.
- `git-final-architecture.log`: `9891d434c979ffbedb02dd13868b3845657bc5650c80585ad7c6c53d29cde5b1`.
- `git-architecture-clippy.log`: `0de9f0f1f122101501ce2d729bc1789573632f86c32b457525443333ac5235c2`.
- `git-final-validation.json`: `4fafcd034d315e2552ddfaee98b25c460df2c2905b54a6b406e05df3b962f995`.

## Remaining scope and checklist

The brief, boundary mapping, upstream research, resource review, first failures,
focused/package checks and diff review are recorded. Navigation regeneration, documentation policy (814 statuses / 812 coordinates),
Rust formatting and staged/unstaged diff checks passed before publication. All
seven fixture hashes were independently verified against their manifest. No roadmap or POAM
completion status changes. PR224 stays draft; this increment has no candidate
or protected merge acceptance and does not inherit PR223's ongoing CI.

Next bind an admitted versioned policy and bounded native reads to these pure
inputs, then atomically publish source records/containment/change sets with the
attunement checkpoint using `commit_with_control_transitions`. Git metadata
root/config/global-ignore admission, supported split/sparse behavior, cancellation,
real repository traversal, non-Git/safety/symlink cases, installed restart/edit
replanning and Linux/macOS verification remain open. The installed Inventory
phase is still unavailable. No worker effect, model call, training record,
release artifact or complete-workflow evidence was produced.

# D-03 Git-aware installation source traversal

**Status:** active implementation evidence; durable Inventory remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-git-aware-source-traversal`
**Owner:** OBJ-14 / D-03 / POAM-006; PR224 driver: active SWAG implementation session

## Bound change and demonstrated advancement

[PR224](https://github.com/rrflow/rrflow-development/pull/224) baseline
`d1b8a26a73cfb8f01791737913b4098988c1acdd`, tree
`c4cc385adc54705cd5b22cdc26ade218b2ebd4a0`. The PR brief declared the canonical
walker increment before edits and subsequently named its installed-test and
operational-reference projections. Staged implementation/reference tree before
this journal/index: `8676708f87678e492e8f9c37de7047d4f35e1daf`.

Previously installation and replanning used safety exclusions but no Git tracking
or ignore inputs. That same active traversal now applies bounded admitted Git
configuration/index/info-exclude/worktree-ignore data, retains tracked ignored
source, avoids opening excluded secret/ignored files, and records non-followed
native leaves and tracked submodule roots in its precondition. Root, directories,
files and present/absent Git inputs are revalidated before returning the digest.
The precondition grammar advances to v3 and binds ordered Git-input identities.
Installation profile/configuration ownership is unchanged.

The installed-engine oracle preserves the non-Git case and adds a Git project:
ignored edits permit start and two selected Connect steps; a tracked-file edit
returns `AttunementSourceChanged`; successor acceptance preserves the selected
steps and original receipts; reopening replays identical predecessor/successor
receipts. No Inventory checkpoint or source-mutation worker is implied.

## Scope, ownership and review

Changed `installation/inventory.rs` and added private `inventory/git_inputs.rs`.
Added pure Git configuration parsing at `rrd-attunement/src/git/configuration.rs`,
its export/error variants, manifest and parser tests. Updated the exact dependency
assertion, installed attunement tests, SDK identity projection, installed-lifecycle
reference, this journal and its generated index. No source was moved/deleted and
no parallel traversal/runtime authority was added. Existing bounded-reader,
admission-count, ordering, exclusion, hidden-source, metadata/depth and expiry
oracles remain. The new fixed hash was independently reconstructed with Python.

Reviewed the complete active inventory/policy implementations, pure Git parser,
new configuration/Git-input modules, native observation/leaf APIs, affected
installation/attunement/CLI/SDK boundaries, exact dependency checks, generated
identity changes and package diff. The broader attunement test module was inspected
at its changed setup and selected-plan functions; this is not a whole-engine review.

## Research and resource decisions

Primary references: [Git ignore semantics](https://git-scm.com/docs/gitignore),
[repository layout](https://git-scm.com/docs/gitrepository-layout), and pinned
Gitoxide `gix-config` 0.62.2 source (`from_bytes_no_includes`, metadata `api`,
section/value accessors). Only buffer interpretation is adapted. Runtime code
does not invoke repository discovery, ambient configuration, include expansion,
object loading or Git subprocesses. Four locked packages are added (`gix-config`,
`gix-config-value`, `gix-ref`, `gix-sec`); existing package versions are unchanged.
Existing dependency features also add `walkdir`/`bstr` edges.

External/non-directory Git metadata, includes/external excludes/worktree
indirection, unsupported extensions, sparse/split inputs and unverified index
checksums fail explicitly. Git input bytes have an aggregate ceiling; observations
and absent inputs count against source/metadata bounds. Buffers are read only to
admitted lengths with native observations and elapsed checks. Source metadata
enumeration precedes source content streaming. Logical metadata accounting includes
Git observations; it is not an RSS bound. Existing command logs suffice; no runtime
trace subsystem or performance claim is added. Fixtures are correctness inputs,
never learning records or independently reviewed outcomes.

## Retained failures and corrections

- First parser oracle: compilation failed because `RepositoryConfig` did not yet
  exist. Initial adaptation exposed three pinned-API method/type mismatches;
  these were corrected against the crate source.
- An initial malformed-config assertion used unsectioned text accepted by the
  upstream parser. The oracle now uses an unterminated section; admission cases
  also cover uppercase keys, implicit values and external-input directives.
- The old v2 digest failed after the deliberate v3 grammar change. Independent
  Python reconstruction produced
  `31a1b80977ce83f586733848513b6d9079fecf3337034f4a4e6ad05208c9af9a`.
- The installed test initially used the wrong fixture include depth; corrected
  from the compiler diagnostic.
- First full engine run: 268 passed, one stale SDK identity assertion failed,
  four existing ignored. The actual planner regenerated nine identity/scope
  fields; the complete rerun passed.

## Observed validation

Windows, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2`, explicit existing
Cargo target cache, no sibling source dependency:

- `cargo test -p rrd-attunement --locked`: 15 Git/config and 19 inventory tests passed.
- `cargo test -p rrd-engine --lib --locked installation::inventory::tests`: 14
  passed, including actual locked ignored/secret files and an outside directory
  junction whose target content does not affect the precondition.
- Installed Git selected-plan/source-edit/reopen oracle: passed.
- `cargo test -p rrd-engine --lib --locked`: 269 passed, zero failed, four existing ignored.
- Strict Clippy for all targets of `rrd-attunement` and `rrd-engine`: passed.
- `cargo test -p rrflow-cli --test installed_lifecycle --locked`: seven passed.
- Edge `workspace_architecture`: 32 passed. Rust formatting passed.
- SDK installation identities regenerated through the actual read-only planner.

Real-repository evidence uses an isolated public clone of baseline `d1b8a26a`,
internal `.git` and an uninitialized submodule leaf. The rebuilt primary was copied
to a single-link executable. `rrflow --json install identity` and four
`install plan --mode existing --project <clone>` calls succeeded: unchanged source
has the same digest; an excluded `target/` file preserves it; a README edit changes
it. Original README bytes were restored and no estate was created. Individual
elapsed observations: 23.469 seconds initially, 7.515 unchanged, 10.938 with the
excluded addition, 11.109 with the source edit. This is read-only preflight, not
a benchmark, zero-read reuse, installed Inventory or clean-machine qualification.

Both installed Python SDK tests passed against the copied primary (11.07 seconds
of test execution). SDK identity verification, format, generated surfaces,
documentation (821 statuses / 819 coordinates), navigation, execution inventory
(2,087 records) and staged whitespace checks all passed.

## Remaining scope and checklist

Declared scope, replacement map, first failure, research/resource decisions,
affected-boundary review, installed oracle and failures are recorded. No objective,
roadmap or POAM status changes. PR224 remains draft and unmerged; PR223's protected
pass does not qualify this tree. Linux/macOS and mount campaigns for this increment
have not run here.

The precondition still rereads eligible content. It is not an atomic filesystem
snapshot, authenticated persisted cache, error-entry publication, canonical
snapshot/change set or Inventory checkpoint. Unknown/racy observations must not be
reused. Cancellation, complete descriptor/output accounting and post-read race
semantics remain full-executor work. Linked Git worktrees, ambient/global config
and non-ordinary Git metadata are explicitly unsupported by this admission policy.
No provider call, worker effect, training admission, distribution, container or
public-readiness claim is made.

Next: carry admitted traversal into the pure proposal builder; reconstruct prior
snapshots from validated durable records; atomically publish source records,
containment/change set and the resulting runtime checkpoint; prove unchanged
zero-content reads and installed Inventory edit/replan/restart.

## Local evidence identities

Retained under `20261030/engine-source-inventory/`:


- `walker-results.json`: `5c7068ba377aaa77a03b65884f22050a083c6e6c08f3f634404586216741fb8e`.
- `walker-final-results.json`: `7a81a9bee7d739f1a7d9a912f4c5193a3f894c99f4ab8fbf159374cb23edc978`.
- `walker-independent-digest.json`: `897fe8521c57b2983d9e7bcbfe54d34bd1852f4db5c2c8deea330a96b79721d4`.
- `walker-source-envelope.json`: `743430faa3aa6e01002df90b0816d1811dea2548c921e2b6e1a54afcdb7b06d0`.
- `walker-repository-corpus.json`: `b31c383c78d399977da0ff9c10182ab673ed6b79e1c786483c03b0a721ee8507`.
- `walker-repository-results.json`: `1d6b482cfcc3a9df841acccc47999a0437c208f868ebcf4188af497f8334ed3d`.
- `walker-python-sdk-result.json`: `f7784d44f8c8f44b73256458797097e47e6f2194459691882341caaa80e8b2fb`.
- `walker-repository-check-results.json`: `1a722f07ad92588c38eef7135f8d5aa03a6f3a8109b5107017065229db833e90`.

Actual copied primary SHA-256: `f17fead19f8344d05949cbd72121c163e10626887a52f57c9a49ed095d62bede`.

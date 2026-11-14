# D-03 installation inventory policy

**Status:** active partial implementation evidence; committed Inventory remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-installation-inventory-policy`
**Owner:** OBJ-14 / D-03 / POAM-006; PR224 driver: active SWAG implementation session

## Bound change and advancement

Starting revision `22bbd5ff17a6e092391a1d0d0268d2610bd79289`, tree
`39964f486cee6891b858af1178ae956b260e93d0`, on
[PR224](https://github.com/rrflow/rrflow-development/pull/224). Its brief admitted
this policy increment before implementation. PR223 remains a landing dependency;
PR221 overlaps installation plan templates and requires integration before landing.

The installation precondition previously had only file-count and content-byte
limits. It now compiles typed policy from the existing versioned installation
profile, applies positive safety exclusions before content reads, bounds visited
entries, path/depth, retained logical metadata, eligible content and elapsed work,
and binds the policy SHA-256 into the version-two precondition digest. Native
identity observations around each opened file strengthen replacement detection.
The precondition remains distinct from a complete committed project snapshot.

The version-three default/clyffy profiles carry policy values as reviewable data.
No provider/model/task choice is embedded in code. Exclusions include generated
outputs, key/environment files (including `server.env`) and `.swag` private local
state. Hidden project configuration such as `.github` remains eligible. Oversized
files are excluded during enumeration; an already admitted file growing beyond
its ceiling fails rather than silently disappearing. Excluded entries still
consume enumeration budget. Policy changes alter the precondition even when
eligible content is unchanged. Safety rules cannot negate an earlier exclusion.

The policy's Git, symlink/mount and conservative reuse settings specify the
forthcoming Inventory implementation. Installation still reads eligible content
and rejects symlink/mount traversal through its anchored filesystem authority.
It does not yet consume repository index/ignore inputs, emit exclusion records,
reuse cached content, or activate the Inventory phase. Output, handle, concurrency,
attempt and Git-input fields do not establish implemented executor guarantees.
Metadata accounting is logical retained-path accounting, not measured RSS.
Elapsed checks occur between bounded operations; a blocking native read is not
interruptible and no hard wall-clock latency guarantee is claimed.

`EstateConfiguration`, public operations and authorization remain unchanged.
POAM-006's broader configuration admission is not bypassed by a new settings API.
Old binaries, estates and receipts keep their original identities. This package
neither migrates nor reinstalls them.

## Traceability, review and decisions

Canonical paths: `rrd-engine/src/engine/installation/policy.rs` (new), the existing
installation loader/inventory, engine manifest/lock and exact architecture test.
Both `assets/install/*-profile-v2.json` assets move to `*-profile-v3.json`; the
execution inventory mapping follows those canonical replacements. Independent
JSON comparison confirms that only format version, the two old limits and the
new policy differ; all prior template, specialization and deployment fields are
preserved. No parallel legacy profile implementation is retained.

Existing fixed profile identity tests now contain independently computed hashes
of the LF asset bytes. The existing `refresh_sdk_installation_identity` helper
refreshes the SDK fixture through the actual read-only engine planner. Its missing
three quantization request scope updates were repaired after the validator
rejected the initial refresh; validation was not relaxed. Only nine fixture
identity/scope fields change, and the inert executable anchor stays unchanged.
Authored correctness fixtures are not training data or model quality evidence.

Reviewed: complete change-authoring procedure and prior Git-input journal;
README/AGENTS, D-03, POAM-006 and engine-data-flow owner sections; installation
load/validate/preview, attunement observation/control/replan and configuration
admission boundaries. New policy source, inventory implementation/tests, both
profile assets, SDK helper and complete diff were read. Large installation,
architecture and bootstrap test files were reviewed at affected boundaries;
no whole-file semantic review of those larger modules is claimed.

No new upstream algorithm or external dependency is introduced. This composes
the already researched pure Git matcher and native observation boundary. No new
runtime tracing, benchmark or debug service is needed: the actual filesystem
oracle exposes secret opens, while existing engine tests cover identity and
installation behavior. Stop conditions remain undeclared configuration authority,
unsafe traversal, independent lifecycle/state, or unsupported completion claims.

## Failures and verification

Windows, with `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2`, and an explicit
existing development build cache; no sibling source/runtime dependency.

- Initial `.env` locked-file oracle passed; broader review identified the missing
  `*.env` and `.swag` exclusions. The extended test failed with native sharing
  violation before correction. It now holds `.env`, `server.env` and
  `.swag/session` exclusively open: their presence leaves the digest unchanged,
  while an eligible locked `ordinary.rs` still fails. No real secret was read.
- Initial SDK identity refresh failed because quantization scopes retained the
  prior instance. Updating the canonical helper fixed all three projections.
- First full engine suite: 261 passed, one failed, four ignored. The failure was
  the pinned old profile hash. Expected new asset and precondition hashes were
  independently computed using Python hashlib; no assertion was removed.
- `cargo test -p rrd-engine --lib --locked secret_exclusions_precede_actual_file_opening`:
  one passed after reproducing the failure.
- `cargo test -p rrd-engine --lib --locked engine::installation::`: all 10 passed.
- `cargo run -p rrd-engine --example refresh_sdk_installation_identity --locked`
  and the same command with `-- --check`: both passed, with no installation.
- `cargo test -p rrd-engine --lib --locked`: 262 passed, zero failed, four existing
  ignored tests; 12.10 seconds executing tests in this local run.
- `cargo fmt --all -- --check`: passed.

- `cargo clippy -p rrd-engine --lib --tests --example refresh_sdk_installation_identity --locked -- -D warnings`: passed.
- `cargo test -p rrd-engine --no-default-features --features edge --test workspace_architecture --locked`: all 32 passed.

Navigation regeneration/check and documentation policy passed (815 statuses,
813 classified coordinates). All 10 execution-inventory tooling tests passed;
the generated inventory contains 2079 records and passes its drift check.
Generated-surface parity passed for 70 HTTP operations and seven signal
projections. Staged diff whitespace checks passed.

Local retained evidence under `20261030/engine-source-inventory/`:

- `policy-secret-regression-before.log`: `684e919f25a62d48f1af089f1f2c914a75d392974e545ab03f163bfb13e5c746`.
- `policy-corrected-results.json`: `66773456e9e92c9a655466cb5b30cccbe92bf6de19482458076e6da6f6ccdb02`.
- `policy-final-results.json`: `f1f0431ecceb1a838d319999c4e112ae15cd0a742fc805f308f8fc9c01517a0c`.
- `policy-independent-identities.json`: `cbc5a98a8760b0dd94970a055577a9c769a4b18fa50e2a50e47d45fe4ef376f5`.

- `policy-check-results.json`: `ff1770b55d95294d481a21a5a86881f43f805b1c1afaadbdbf9d1234a17f095d`.

- `policy-repository-results.json`: `424d90c4aa9207c081e66c52fd126cbdcf3eefae0f044eb8e67fe30f69a79e25`.

## Remaining work and checklist

The published brief, owner mapping, before/after oracle, failure retention,
resource/research decisions, profile replacement traceability and diff review
are recorded. No objective, roadmap or POAM completion status changes. The known
remaining scope stays with D-03/POAM-006. PR224 remains draft and cannot inherit
PR223's CI or merge acceptance. Linux/macOS policy/native verification, clean-machine
and installed new-binary trials, and full candidate/protected CI were not run for
this unfinished package. No new provider call, training or release artifact.

Next implement the actual bounded Git/native walker, validated prior snapshot
reload and atomic source/containment/change-set plus checkpoint publication in
the existing runtime/control transaction. The checkpoint must bind the resulting
runtime read stamp. Then prove ordinary-edit successor planning and installed
restart without duplicating accepted effects or resetting spent budgets. A passing
installation policy does not establish any of those workflow outcomes.

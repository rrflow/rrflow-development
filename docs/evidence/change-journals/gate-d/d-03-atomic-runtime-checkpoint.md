# D-03 atomic runtime checkpoint composition

**Status:** active partial implementation evidence; Inventory execution remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-atomic-runtime-checkpoint`
**Owner:** OBJ-14 / D-03 / POAM-006; PR224 driver: active SWAG implementation session

## Bound change and advancement

Starting revision `de29d92eb8c6921b158164f156da86362691e331`, tree
`fd0e82de7e581a25fd15af289254a9bc2031ee09`, on
[PR224](https://github.com/rrflow/rrflow-development/pull/224). The PR description
admitted this exact store/test increment before implementation. PR223 landing and
PR221 installation-plan integration remain dependencies of the full package.

Previously `RuntimeRepository::commit_with_control_transitions` accepted only
pre-encoded control bytes. An Inventory checkpoint could not be constructed from
the resulting authenticated runtime read stamp at that boundary. The same
repository now exposes `commit_with_control_builder`: prepare and stage the
runtime plan, derive its read stamp through the same transaction, build and
validate control transitions, then publish everything through one authoritative
commit. The caller's return value is delivered only on commit success. The
builder runs once without automatic retries and must not perform external effects
or independent commits. Engine-specific validation errors retain their type.

The existing method delegates to this implementation and borrows its input slice;
it does not clone an arbitrarily large control batch. Installation and reasoning
budget callers retain their API and use the same authority. No public protocol,
store, raw-write lane, product version or external dependency is introduced.
Builder rejection, stale control CAS and known storage admission rejection drop
staged writes. An uncertain native I/O error still requires outcome reconciliation;
this change does not turn uncertainty into proof of no effects.

## Scope, review and decisions

Changed source: `crates/persistence/rrd-store/src/repository/runtime.rs` and its
existing `tests/installed_lifecycle.rs`; evidence: this journal and its generated
Gate D index. The old composition body is replaced by delegation to the one
canonical implementation, preserving existing bootstrap/control characterization.
No alternate persistence authority or schema for Inventory is introduced here.

Read: the complete installed-lifecycle test, new composition implementation and
authored diff; the owning README/AGENTS/change procedure and D-03 source/checkpoint
requirements; runtime read-stamp and semantic-plan apply/preparation sections;
control validation/preparation; engine attunement publication and checkpoint
construction; installation and reasoning-budget composition callers. Large
existing files were reviewed at these boundaries, not claimed read in full.

No external research or source adaptation was required: this extends the existing
transaction and authenticated read-stamp owners. No benchmark or new runtime
tracing was added. Actual row scans, one-frame WAL inspection and native reopen
provide the relevant atomicity evidence. Authored test records are storage
correctness fixtures, never routing evaluations or learning data. Stop conditions
were missing transaction read-your-writes, leaked rejected state, a second commit
owner or unsupported installed-workflow claims.

## Oracles and validation

The first new test compilation failed with missing `commit_with_control_builder`;
that interface did not previously exist. After implementation, all five
installed-lifecycle tests passed, including the three preserved cases. The new
actual rrflowMX/rrflowKV cases verify:

- A staged schema, record and claim produce a checkpoint containing exactly the
  resulting authenticated runtime read stamp, including the new schema revision.
- Builder rejection and stale control CAS leave every physical fixture row
  unchanged; no rejected runtime outcome or audit record appears.
- A subsequent accepted checkpoint succeeds, its builder runs once, and native
  reopen recovers the identical checkpoint/read stamp. Rejected attempts produce
  no WAL frame; the successful runtime/control publication produces exactly one.

Windows validation used `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2`, and the
existing explicit development target cache, without a sibling source dependency.

- `cargo test -p rrd-store --test installed_lifecycle --locked`: five passed.
- `cargo test -p rrd-store --locked`: 251 passed, zero failed, two existing ignored
  manual diagnostic/checkpoint measurements across 37 result summaries.
- `cargo clippy -p rrd-store --all-targets --locked -- -D warnings`: passed.
- `cargo test -p rrd-engine --lib --locked`: 262 passed, zero failed, four existing
  ignored; includes the existing installation and reasoning-budget callers.
- `cargo fmt --all -- --check`: passed after formatting.
- Navigation regeneration/check and documentation policy passed (816 statuses,
  814 classified coordinates). Execution inventory regenerated and passed its
  drift check with 2080 records. Staged diff whitespace checks passed.

The first PowerShell-captured logs include stderr presentation wrappers; native
exit codes distinguish the expected compile failure from passing tests. During
review, the test's proposed head-digest comparison was corrected before execution:
a runtime change-chain head is not a commit ID. Final tests compare the complete
resulting read stamp and separately verify the commit ID against the request.

Local retained evidence under `20261030/engine-source-inventory/`:

- `checkpoint-first-oracle.log`: `9defbe752c7dc40b67e1c7477c941a57496bc2241f9c839b3ddeb14085b0e559`.
- `checkpoint-focused.log`: `e4b7adedd9abecdc3da74087df2d09b752c03007addaf9081dc2033d9562494a`.
- `checkpoint-storage.log`: `80bb45956b1870d238cdfa9bb0a5dcbba64ab0125fa808646565f2168dc41c6c`.
- `checkpoint-clippy.log`: `ec2f3f74a88bb91ec50a7cda822c3bbe55c8d48c1feac7c720b3f8940e60e135`.
- `checkpoint-results.json`: `340ab9ae47874bd0918f02a8d3fe95a2b9fca7836e4fd325d7c70113a1fe015d`.

- `checkpoint-repository-results.json`: `d464f93dec335492cb99f17ff54d14f4a6760fea6ed2da0e364024377f67260b`.

## Remaining scope and checklist

The brief, source replacement map, before/after oracle, first compile failure,
actual storage evidence, owning and affected-engine suites, resource/research
choices and diff review are recorded. No objective/roadmap/POAM completion status
changes. PR224 stays draft; no candidate or protected merge acceptance is claimed
for this increment. Linux/macOS execution, new installed binaries, complete crash
campaigns and release qualification were not run. No provider call or training.

D-03 still requires the bounded Git/native walker, validated prior-snapshot reload,
source/containment/change-set records and actual Inventory phase integration. The
engine must revalidate authorization, source identity, budget, deadline and lease
at publication and construct its checkpoint using this boundary. Installed
ordinary-edit successor planning and restart proof remain required before claiming
a usable controller workflow. The helper's passing storage tests do not substitute
for that integration.

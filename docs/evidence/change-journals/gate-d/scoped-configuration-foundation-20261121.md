# Shared user configuration at query and recall admission

**Status:** active bounded implementation evidence; local Windows acceptance passed, candidate and queue pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/scoped-configuration-foundation-20261121`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), D-01/D-06 and [POAM-077](../../../poam/rrflow-1.0-alpha/poam-077.md)

## Brief and traceability

Driver: this implementation session, one branch `feat/scoped-configuration`,
[PR456](https://github.com/rrflow/rrflow-development/pull/456). Starting revision:
`b5bfb358b0b7e17e24dce2861c556633bba1b155`. The draft PR's brief preceded implementation.
This advances the alpha prerequisite of governed, explainable configuration
at the existing installed engine boundary. It does not close the full
[alpha objective](../../../objectives/rrflow-1.0-alpha.md).

Before: installed project query and recall ceilings were immutable, with no
durable shared user source or scoped resolution. After: the OS owner can
preview and publish an exact user snapshot in one private rrflowKV control
store; two project installations explicitly bind it, enforce numeric policies
and expose their effective values and source coordinates. Each project retains
its own knowledge, credentials, engram and instance. Query/context results
carry the actual configuration admission stamp, distinct from the data read.

| Existing behavior and source | Canonical destination and retained oracle |
|---|---|
| `deployment/configuration.rs` seals project capacities and digests | Retained; scoped input participates in the same digest, validation and installation preview. Existing deployment tests remain. |
| `engine/core.rs` enforces installed query/recall ceilings | Retained, then composes exact policies through `engine/scoped_configuration/resolution.rs`; existing query/recall tests remain. |
| Native `RrflowKvStore` and `ControlRepository` provide ownership and atomic CAS | Reused in `engine/scoped_configuration/store.rs`, with immutable history and one CAS head; native reopen, stale-plan, lease and history tests. No new storage implementation. |
| Endpoint configuration has authenticated public read and CLI/engine separation | Same boundary pattern for effective-read; OS-owner publication stays engine-owned with thin primary-CLI translation. Installed HTTP/CLI driver exercises it. |
| Contract root manually reexports 21 deployment names | Replaced with the same module's wildcard reexport after comparing the exact public-name set; code-shape ratchet tightens 326 to 321 lines. No deployment API is removed or added by this simplification. |

Scope includes contract/publication types, engine resolution/publication/storage,
primary CLI, authenticated effective-read and client operation, admission stamps,
their tests, generated SDK/schema/signal projections and owning documentation.
No implementation is moved or deleted. Product version remains `1.0.0`.
Stop conditions: another authority, unauthorized cross-project read or global
write, an unbounded history scan, discarded requirements, unsafe filesystem
ownership, or scope expanding to arbitrary policy/model/UI implementation.

## Grounding and decisions

Owner reads: README's identity/configuration and relevant warp points; ADR-0006,
POAM-077 and the change-authoring procedure; Gate D's scoped configuration
acceptance; session standard and joined-delivery skill. This is a bounded
source review, not a claim that the repository's complete source was reviewed.

Full implementation reads include existing contract deployment configuration,
engine endpoint configuration, the new scoped contract/publication modules,
engine scoped modules and tests, CLI scoped modules and installed driver,
and client configuration operations. Relevant surrounding ranges were read in
engine core (1–140, 370–495), installation open/planning, control repository
get/commit_batch, native KV create/open, anchored private-directory helpers,
instance registry, query/recall/flight/cold-history admission, server routing,
operation catalogue/reference/schema dispatch and installed test harnesses.
Only changed behavior and its dependencies are claimed as reviewed; generated
files are checked against their generators, not presented as handwritten review.

Research: use ADR-0006's already sourced scope/precedence decision and existing
native APIs. No upstream algorithm or source copied, no new “latest practice”
claim and no additional external research needed for this bounded composition.
Trace: preserve authorization/audit, expose source revision, policy provenance,
units and exact admission stamps. No secret content or model reasoning logs.
Resource decision: bound input/revision size to 64 KiB and historical traversal
to an explicit maximum of 4096 reads. Native lease contention fails closed;
there is no concurrency-throughput or hardware-write-amplification claim.

## Installed oracle and validation

Named joined driver:
`scoped_configuration_two_installed_projects_observe_corrections_without_restart`
in `rrflow-cli`'s `installed_lifecycle` target. This capability is absent at the
starting revision: the publication CLI, binding, resolver and admission stamp
do not exist. The driver uses the primary binary to publish and install, real
server processes and authenticated HTTP, not fabricated installed identities.

The focused Windows run passed (one test, 5.69 seconds after build). It proves:
two different projects use one user source; project/default precedence is
visible; each reads only its own claim; updating shared row/recall limits
refuses formerly admitted requests on both running servers; smaller requests
succeed with the new revision; historical known-revision reads retain the old
policy; project restart observes the correction; an invalid stored preference
reports conflict; an unavailable required source refuses work; source state
survives both project shutdowns. Unit tests additionally cover stale CAS,
exact retry, half-open late/future corrections, foreign identity, bounded
history, lease contention and expired plans that cannot write.

Commands use `CARGO_BUILD_JOBS=4` and checkout-specific
`CARGO_TARGET_DIR=D:/.usr/Temp/rrflow-root-target`:

- `cargo test --locked -p rrd-contract --test scoped_configuration`: two passed.
- `cargo test --locked -p rrd-engine --lib scoped_configuration -- --nocapture`:
  initial two native tests passed; later expiry/future and Unix permissions
  checks belong to the broader run below.
- `cargo test --locked -p rrflow-cli --test installed_lifecycle scoped_configuration_two_installed_projects_observe_corrections_without_restart -- --nocapture`: passed.
- `cargo test --locked -p rrd-contract -p rrd-engine --lib --tests -- --test-threads=4`:
  contract suites passed; engine library had 525 passed, one stale SDK fixture
  failure and 12 existing ignored tests (451.11 seconds). The fixture correction
  changed no runtime implementation or test oracle. The exact failed case
  `cargo test --locked -p rrd-engine --lib shared_sdk_corpus_identity_matches_bundled_installation`
  then passed. The expiry/future scoped test passed in the library run.
- `cargo test --locked -p rrd-contract -p rrd-engine --test '*' -- --test-threads=4`:
  all 39 integration targets passed, 254 tests, after the fixture correction.
- `cargo test --locked -p rrflow-cli --test installed_lifecycle -- --test-threads=4`:
  all 16 passed, 48.52 seconds, including the scoped driver and existing restart/restore.
- `cargo test --locked -p rrd-client -p rrd-server -p rrflow-qualification-storage --lib --tests -- --test-threads=4`:
  124 passed across 20 targets, one existing SDK harness-dependent test ignored.
- `cargo clippy --locked -p rrd-contract -p rrd-engine -p rrd-client -p rrd-server -p rrflow-cli --all-targets -- -D warnings`:
  passed; `cargo fmt --all --check` and `git diff --check HEAD` passed.
- Python/Java/.NET `scripts/generate.py`, Go `go run ./cmd/generate`, TypeScript
  `pnpm generate`, `python scripts/ci/check_generated_surfaces.py --write` and
  the storage `refresh_storage_contract` example refreshed canonical projections.
  Subsequent `python scripts/ci/check_generated_surfaces.py` passed: 84 HTTP
  operations and seven signal projections; no storage qualification is implied
  by refreshing synthetic contract fixtures.
- TypeScript `pnpm check`: generation, lint, typecheck and seven tests passed.
- Navigation, documentation ownership/link policy and code-shape checks passed.
- `python scripts/ci/run_sdk_conformance.py` could not start because this Windows
  environment lacks `mvn`. No language run is claimed from that attempt; the
  required candidate SDK lane must supply six-language live-daemon evidence.
  Unix permission tests and other native platforms are left to required CI.

Surfaced local failures: the operation-reference list must remain sorted for
binary search; corrected its new entry's position. The installed test initially
used a nonexistent client constructor; corrected it to `connect_local`.
The code-shape check rejected root-module growth; the export simplification
above tightens its ratchet. Regression tests identified stale configuration,
SDK and OpenAPI golden identities; canonical projections and fixtures are
refreshed intentionally, retaining their strict equality assertions. One early
rerun attempted to relink the still-running Windows engine test executable and
failed with LNK1104; after the original suite completed, the same corrected
fixture test passed. No process or test was killed to hide a failure. Initial
documentation checks found the missing parent index and pending journal links;
both were supplied and the check passed. No failing oracle is removed or loosened.

## Limits, checklist and next boundary

This covers eleven numeric query/recall capacities. It does not implement
arbitrary behavioral obligations, model/controller routing, organization
policy, live sealed project changes, remote global writes, Connectome controls,
vector freshness or the complete focus/pivot/correct/return journey. Global
source contention, interrupted initialization recovery, backup/removal/relocation
and synchronization need further lifecycle qualification, recorded in POAM-077.
Same-user filesystem attacker immunity and cross-store atomic admission are
not claimed. Already admitted work retains its admitted revision.

The brief, owner, scope, failure oracle, research, trace/resource and stop
decisions are recorded above and in PR456. The new implementation files and
changed source regions were reread against the full package diff; generated
changes were verified through their owners and golden checks. This completes
the bounded change checklist, not an exhaustive review of unchanged source.
No roadmap gate or
POA&M item closes. CI and merge evidence will be attached to PR456 for its exact
candidate/queue revision; this draft record is not a green-CI claim.

Session scratch is `D:/.usr/Temp/rrflow-scoped-configuration-20261121`; the build
cache above is retained. Tests own and clean their temporary fixtures and
processes. No containers, other worktrees, user services or unrelated data are
removed. Next dependent boundary is governed project-policy editing and
Connectome effective-configuration controls using this public engine contract.

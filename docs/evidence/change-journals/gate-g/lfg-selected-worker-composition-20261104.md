# LFG advisory and selected-worker composition preflight

**Status:** active local integration preflight; not a landed candidate
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/lfg-selected-worker-composition-20261104`
**Owner:** OBJ-06, G-01b/G-02a, H-05 and POAM-009; PR249 and PR229

## Scope and source

The [LFG PR249](https://github.com/rrflow/rrflow-development/pull/249) brief
was amended before this preflight. This isolated worktree composes LFG head
`f17d62b94d6178033186fbc8efb0f7c8dbe69a7a` with worker implementation
`30d2304c3b45904c5e468f94c03eb23e74d02c26`; the worker PR's subsequent
`5185abf90204330619815c99ef30983dac07b923` adds evidence only. These are
separate unmerged candidates; the dependency PRs and their protected CI
must land before this becomes an integrable PR head. No roadmap status changes.

The preflight preserves the analyzer checkpoint and selected-worker output
as separate optional fields under the same engine task-attempt authority.
Selection retains the LFG advisory; the worker must bind the accepted selection
digest and has a distinct attempt. Both public attempt validation and internal
worker output admission reject an advisory attached directly to a worker
result. This strengthens type and provenance separation without inventing a
second budget, replay authority or provider-specific fixed route. No project
mutation or model-quality claim follows.

Read the two PR briefs, the relevant flight and worker boundaries, the existing
LFG flow and worker tests, authored conflict hunks in `endpoint_proposal.rs`,
`reasoning_budget.rs`, `reasoning_budget/public.rs`, contract reasoning
budget, POAM-009 and LFG/SWAG reference. This is not an exhaustive line review
of either full branch. The merge preview identified 25 conflicts: the named
authored boundaries, digest citations and generated client projections. Both
worker and LFG journal histories were retained; generated surfaces derive from
the combined operation catalogue. No upstream source was copied.

## Test oracle and evidence

A new controlled joined test supplies an LFG observation to the real engine,
checks that it reaches the controlled selector, invokes a separate controlled
worker bound to the accepted selection digest, rejects a forged worker/advisory
combination, reopens and checks both attempt records, and refuses replay.
An ignored variant uses an explicitly configured running local LFG service,
with the selector and worker still controlled HTTP fixtures. Such a fixture
proves advisory carriage and selected-worker binding, not that a real SWAG
router used the advisory to change its decision or that a real model ran.

The combined OpenAPI digest is
`7ff11cd88d5fa1e5c1f4b4810b7daeb5f90037de3c801522ce625725a3e79d40`.
`python scripts/ci/check_generated_surfaces.py --write` and `--check` passed:
79 HTTP operations and seven signal projections. TypeScript typecheck and
seven tests, plus Go `go test ./...`, passed on the combined source.
The worker contract test initially lacked the newly composed optional `lfg`
field; its fixture now specifies `None`, and `cargo test --offline --locked
-j 1 -p rrd-contract --test endpoint_worker` passed four tests.
`cargo test --offline --locked -j 1 -p rrd-engine --lib
  lfg_selection_drives_a_separate_governed_worker_and_both_reopen --
  --test-threads=1` passed one test. The separate ignored
`real_lfg_selection_drives_governed_worker_and_reopens` passed with
`RRFLOW_TEST_LFG_URL=http://127.0.0.1:54330`; selector and worker were
controlled fixtures. The LFG process's loaded image was not independently
attested. The fresh isolated Rust test build took 14m 39s on this shared
Windows machine; that is one development-build observation, not a benchmark
or a runtime latency measurement. The full engine suite subsequently passed
361 tests with six ignored; the joined real-LFG ignored case was run separately
as above. The full `cargo test --offline --locked -j 1 -p rrd-contract --tests`
and `cargo test --offline --locked -j 1 -p rrflow-eval` suites passed.
`cargo fmt --all -- --check` passed. No candidate CI or installed-process
result is implied.

The worker and analyzer input/output digests, retained attempts and existing
opt-in redacted diagnostics are the trace/debugging path. They do not attest
loaded model weights, source edit freshness, a full prompt flight, completion
or model quality. [Temporal's activity guidance](https://docs.temporal.io/develop/python/integrations/openrouter)
illustrates why an external call may execute despite a lost response; no
replay or billing semantics are inferred for RRFlow providers. One-job offline
Rust compilation limits contention on the shared host; it is not performance
evidence. Candidate CI, protected merge-group CI, source edits, real SWAG,
real model execution on the composed source and installed product evidence
remain open.

The first documentation check raced the journal update and correctly rejected
a changed Markdown snapshot. It passed when rerun after the record was stable.


## Change checklist and next boundary

The PR249 brief binds the composed source heads, owning outcome, preserved
authority, conflict boundaries, failure oracle, tests and stop conditions.
The production changes are the shared attempt fields, worker/advisory
exclusivity validation and preserved proposal helper; the new controlled
joined tests exercise those paths. Generated SDK and signal outputs and the
linked navigation index are mechanical projections. Authored conflict hunks were reviewed and generated conflicts were
regenerated, with the broader source-review limitation stated above.
Research, correlation and resource choices are recorded above. The next
meaningful proof is an installed binary built from the final composition,
using live local LFG and controlled selection and worker endpoints, followed
by real SWAG/model provider trials and protected CI as those dependencies
land. The user-facing prompt-flight provider arm still remains absent.

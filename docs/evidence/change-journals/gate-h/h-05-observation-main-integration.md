# H-05: observation delivery and accepted main

**Status:** active integration evidence; local validation passed, candidate acceptance pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-05-observation-main-integration`
**Owner:** H-05 observation delivery prerequisite and POAM-008

## Bound change

The SWAG controller session drives [PR235](https://github.com/rrflow/rrflow-development/pull/235).
Starting head `ebeecabb9815a891611f312c99bfa0afe5afee9e` passed all 23 candidate
jobs in [run 41263686147](https://github.com/rrflow/rrflow-development/actions/runs/41263686147).
The merge queue could not compose its generated projections with preceding
entries. Those entries have landed. This ordinary merge incorporates exact
public main `daec1f1a2959f16ce41da9fb691ab1791ed06db5`; the common ancestor is
`6c069e1462d2b9cbb0237e9c117c379d1482bbaa`.

The PR description recorded scope, traceability, failure oracle, checks and stop
conditions before mutation. `git merge-tree` reproduced 19 conflicts. The only
conflicted Rust hunk was the derived OpenAPI digest constant; other conflicts
were generated SDK/signal projections and current digest citations. No engine
or adapter moves, replacements or removals occur. Main retains its vector,
storage, packaging and test behavior; the observation package retains its
contracts, authorization, durable delivery, adapter and tests at their owners.

## Resolution and preservation

The canonical contract retains main's estimator and incremental-generation
fields alongside observation operations. Temporary old projections were used
only to bootstrap the exporter, then all five SDK generators and the canonical
signal generator ran. The resulting OpenAPI SHA-256 is
`fefbaa7fb31b733a731b48f884fb3adaddc4942aed2882c31a6efc1d8b65af69`.
The public operation count remains 74 and the seven signal projections agree.
Current digest citations were updated from the actual export.

Exact Git mode/blob comparison verifies 2,046 non-overlapping parent entries.
Seventeen overlapping generated/reference files differ from the starting head
only in digest values. The other overlaps are the combined CI workflow,
canonical contract and generated TypeScript schema. Main's archive/conflict-tool
checks and the observation adapter's suite inclusion are both retained.
The owning evaluator generator leaves all three storage fixtures unchanged;
there are no new workload results, labels or model data.

## Surfaced failures

The initial Python SDK generation used an ambient environment without `ruff`.
Its failure is retained; generation passed using the existing SDK environment
without dependency or product changes. A local comparison initially expected
four evaluator identity fields to change, as in the earlier operation addition.
That assertion failed because this merge changes the OpenAPI schema but not the
operation catalogue digest. Full JSON comparison instead confirms the refreshed
fixtures are unchanged. No expected workload outcome was relaxed or edited.

## Validation and review

Canonical generated parity and all 165 contract/adapter tests pass. The required
CLI binaries build. Engine/server/client/evaluator all-target suites report 499
passes, zero failures and four existing ignored helpers/native fixtures. One
Rust SDK test returns early without `RRD_SDK_CONFORMANCE_MANIFEST`; that entry
does not prove live SDK conformance. The new candidate must run the actual shared
daemon harness. Exact commands, raw logs and parent-comparison receipts are
retained locally under the `sender-main-integration` evidence directory.

| Command | Observed result |
|---|---|
| `python scripts/ci/check_generated_surfaces.py --check` | 74 operations and seven signals agree. |
| `cargo test -p rrd-contract -p rrd-observation --all-targets --locked` | 165 passed, none failed or ignored. |
| `cargo build -p rrflow-cli --bins --locked` | Pass. |
| `cargo test -p rrd-engine -p rrd-server -p rrd-client -p rrflow-eval --all-targets --locked` | 499 reported passed, four ignored; SDK limitation above. |
| `python scripts/knowledge/render_navigation.py --check` | 18 indexes, 890 nodes, 3,207 edges; no drift. |
| `python scripts/ci/check_documentation.py` | 891 document statuses, 889 classified coordinates and local links pass. |
| `python scripts/ci/build_execution_inventory.py` then `--check` | 2,281 current/generated/planned records; no drift. |
| `python scripts/test_controller_trial_package.py` | 11 rejection tests pass; one actual-archive class skipped without an archive. |
| `python scripts/ci/test_inspect_merge_conflicts.py` | Five tests pass. |
| `cargo clippy -p rrd-contract -p rrd-observation -p rrd-engine -p rrd-server -p rrd-client -p rrflow-eval --all-targets --locked -- -D warnings` | Pass. |
| `cargo fmt --all --check` | Pass. |
| `pnpm --dir sdks/typescript generate:check` and `typecheck` | Both pass. |
| `python scripts/ci/check_workflow.py` | Pass. |

Owning-suite log SHA-256:
`e254112945b955c09813ecdaf0b03b92439f5d1d6337de5e56b61f3a04b1483d`.
Contract/adapter log SHA-256:
`49b54ce2b609ab85d61decce91adbd0c8faefa239a1a8c0ae94546b78a294bd1`.
Strict-lint log SHA-256:
`8d8ae5d1b683ee7439e5481537de8359a02bd806585d57b4897f8bc956f2147e`.

Reviewed owners include root instructions/README, change-authoring, the prior
observation journals, the public-contract and SDK references, overlapping
workflow/contract changes and the canonical generator entry points. Review
combines changed-region inspection, whole-file parent identity checks and
generator parity; it does not claim independent reviewer approval or a fresh
manual reread of unchanged generated schema text. The complete overlapping
source/workflow/schema diff, new journal and index have been reviewed; final
navigation/inventory checks are enforced before publication. No new native
archive, installed sender/receiver trial, model evaluation or non-Windows local
test is claimed by these checks.

Checklist: the existing PR carries its pre-edit reconciliation brief; the
conflict preview supplies the before oracle; canonical generation and parent
identity comparisons supply the reconciliation oracle; the owning suites pass;
failures and omitted live qualification remain explicit; this new journal and
generated index preserve previous immutable evidence. No files are moved or
deleted by the reconciliation. Candidate and protected CI remain the next
acceptance steps, not inferred passes.

No external research, upstream adaptation, new runtime tracing or resource
experiment is needed for this reconciliation. Stop on unexpected semantic
conflicts, lost parent behavior, changed fixture outcomes, failed owning checks
or a changed remote head. Original binary/candidate evidence keeps its original
identity. A new exact-candidate and protected run is required. No roadmap,
POA&M, model-quality, complete project-workflow or release status changes.

## Links

- [Observation implementation evidence](h-05-engine-observation-delivery.md)
- [Original candidate corrections](h-05-observation-ci-coverage.md)
- [H-05 roadmap owner](../../../roadmap/rrflow-1.0/gate-h.md)

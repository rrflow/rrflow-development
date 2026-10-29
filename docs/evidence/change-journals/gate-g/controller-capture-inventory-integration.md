# PR188: preserve the landed inventory owner

**Status:** active integration evidence; fresh candidate and native failure reconciliation pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-capture-inventory-integration`
**Owner:** G-06a observation preparation
**Pull request:** [188](https://github.com/rrflow/rrflow-development/pull/188)

## Starting source and scope

The collector started clean at `fc7ccb2ef929b141304d8f13976a50c67b5e59fe`.
Landed main is `291fee511887932572ecc9935e9786e098727677`, including PR180's
merge `45a0cd82733abc93a1374796c47bf5d68bd2bbcc` and PR193's Git-derived
inventory change. An ordinary no-commit merge reproduced the sole preflight
conflict in `docs/roadmap/rrflow-1.0-file-plan.jsonl`.

The existing PR description was extended with this exact integration brief
before mutation, preserving the separate session's native-failure hold.
Read owners: AGENTS, README, the existing collector and integration journals,
the change-authoring procedure, both repository inventory journals, the
inventory generator and its diff, the conflicted inventory, the merged CI
diff, the navigation generator and current PR/check state.

## Resolution and implementation trace

Main deliberately retires the tracked inventory to a small historical marker;
the current projection is now generated under ignored `target/`. Preserve that
exact marker from main, rather than restoring a competing tracked projection.
Generate the current inventory from the combined Git index. Collector and
review implementations/tests stay at their canonical `scripts/inference`
paths, byte-identical to the starting head. All three CI commands remain.
Other incoming engine/SDK changes are main's landed work, not authored here.
No algorithm, model, data labels, provider effects or training change is made.

This journal and its generated Gate G index are the only additional evidence
paths. There are no source deletions or moves by this repair. The old inventory
blob remains recoverable through Git and main's explicit retirement marker.

## Failure and validation scope

Candidate run `40810426309`, attempt one, failed macOS repeated APFS capacity
setup: `hdiutil create` returned `Resource busy`. The first failed log and job
state were retained before a single retry on unchanged source. That retry was
still running at integration preparation; it cannot qualify this combined head.
POAM-035 already owns the failure class. No check is disabled or weakened.

The integration oracle is an empty unmerged index, exact preservation of all
collector/review source files, main's retirement marker, and successful owning
workflow/documentation/navigation/inventory checks. A read-only invocation of
the actual collector against an existing repository and retained real task
context must run and deduplicate in a fresh process. It creates unreviewed
observations only, with no fabricated outcomes or training admission.

No new external research, GPU run, resource benchmark or provider trace is
needed for this generated-file integration. The previous 30 focused test
results remain evidence for their recorded source, not a claim that the native
combined candidate passed. Required full CI must exercise this exact candidate.
No gate completion, live integration or qualified learning is claimed.

## Completed local checks

- `git ls-files -u` returned no unresolved entries. `git diff --cached --check`
  passed. `git diff --exit-code HEAD -- scripts/inference` confirmed unchanged
  collector/review sources and tests; the retirement marker exactly matches main.
- `python scripts/ci/check_workflow.py` passed. All three collector/review test
  commands remain in the existing CI block.
- `python scripts/ci/check_documentation.py` passed: 734 document statuses,
  732 classified coordinates. Navigation generation/check passed with 18
  indexes, 733 nodes and 2,832 edges.
- `python scripts/ci/build_execution_inventory.py` and `--check` passed with
  1,890 records at the new ignored target. This journal's final addition requires
  a final regeneration before publication; the clean candidate is bound by CI.
- The actual collector ran with `python -S`, using the existing PR180 worktree
  and retained public PR180 task context. One record was captured; a fresh
  process returned `unchanged` with zero appended records. No project files were
  edited, outcome labels supplied, provider called or data admitted for training.
  Local evidence is retained in the SWAG workspace's 20261029 goal-start receipt.

The full candidate and protected merge checks remain required. The unchanged-head
native retry was still running when these checks completed.

# RRFlow 1.0 execution checklist

**Status:** active canonical RRFlow 1.0 roadmap chapter
**Coordinate:** `rrflow://rrflow-instance/data/roadmap/rrflow-1.0/execution-checklist`
**Owner:** [RRFlow 1.0 release roadmap](../rrflow-1.0.md)

This chapter is part of the canonical RRFlow 1.0 roadmap. The parent
record owns release order and completion status; this file cannot change
either independently.

This checklist is the release order, not an inventory of aspirations. Work may
not skip a gate because a later subsystem already has partial code. A checkbox
changes to `[x]` only in the same reviewed change that supplies its required
behavioral evidence.

Checklist rules:

- execute the dependency spine below; gate letters classify release outcomes
  and are not a false total order;
- keep one checklist item per coherent commit unless two items cannot be tested
  independently;
- update **Current status** when an item changes observable product behavior;
- ship one accepted RRFlow 1.0 path for each operation and physical format; do
  not retain alternate pre-release entrypoints, forwarding aliases, backend
  selectors, migration executors, or dual writes;
- recognize no legacy/deprecation class during pre-release convergence:
  classify inventory as accepted target behavior or superseded residue,
  absorb every required behavior into its canonical owner, and remove the
  competing path plus its successful old-shape fixtures in the owning gate;
- trace every affected current behavior, module, test, and fixture into its
  canonical boundary before a deletion, move, merge, or rewrite; Git ancestry,
  merge status, and compilation are not consolidation evidence;
- do not count compilation, mocked UI state, generated schemas, or an artifact
  file existing as behavioral proof;
- require exact/reference comparison before enabling an approximate index;
- require close/reopen evidence for persisted state and crash/failure evidence
  for acknowledged writes;
- require every public surface to reach the same `RrdEngine` operation; and
- stop at the first failed gate, repair it, and rerun the smallest owning test
  before continuing.

## [Executable dependency spine](executable-dependency-spine.md)

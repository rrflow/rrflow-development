# Recovered backup-scheduling stash integration

**Status:** active integration; merge and validation pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/backup-stash-integration-20261121`
**Owner:** [Gate C](../../../roadmap/rrflow-1.0/gate-c.md), C-07 and [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md)

## Change brief

Baseline ffb4c4b1779212aa8fa80e2d5908f3685f98d4ab. Integrate recovered stash
a7f8457c7f790e0d0431c2f75488983c864a9903, including the four formerly untracked
files in third parent 5888eb031df3cd577440ea89cf8aa470664d4964. Use ordinary
merge history and retain the behavior at the current qualification boundary;
do not recreate the removed rrflow-eval crate. This is the owner's requested
source integration, not a new comparison-only inventory.

Source map: CLI tests/support/sustained scheduler, lifecycle and management;
qualification-storage sustained, progress and sustained_backup modules/tests;
existing backup journal, two generated schemas, C-07 and POAM-035 evidence.
The third parent contains management.rs, sustained_backup.rs, its tests and
the source journal. Read the complete source journal and tracked path list
before opening the PR; read affected source while resolving each conflict.

Preserve one owned backup task, original deadlines, cancellation, nonblocking
foreground work and truthful cross-segment completion/overlap evidence. Keep
current module splits, stronger later checks and all recovered assertions.
First oracles: qualification backup histories and CLI management tests. Then
owning qualification suite and installed_reopen target, formatting, generated
projections, documentation, candidate CI and merge queue. Stop on lost effects,
weakened evidence or unknown ownership. No benchmark/throughput claim, external
research or new runtime architecture is needed. No release gate is closed.

The stash, its untracked tree and the verified recovery bundle are readable.
No source is deleted before integration; exact resolutions and results follow.

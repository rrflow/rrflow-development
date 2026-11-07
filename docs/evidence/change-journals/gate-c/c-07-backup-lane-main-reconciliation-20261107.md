# C-07: backup-lane repair reconciled with main

**Status:** active candidate preparation; C-07 and alpha remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-backup-lane-main-reconciliation-20261107`
**Owner:** C-07g, OBJ-10/11, [POAM-035](../../../poam/rrflow-1.0-alpha/poam-035.md), [PR #294](https://github.com/rrflow/rrflow-development/pull/294).

The starting draft revision was `2f35f9acb839aa79369ae84bfe7b2a9d81f963ae`,
tree `7b7b9b0ff7cbf057c19106d02c4d5d1aa008b200`. The actual public main
merged from PR #293 was `838c85120621d5d9dde989d7677cc6ec89d20f03`, tree
`762d03792871a7a193a9e43a815b1e6ba61923ac`. Its protected merge-group
pipeline passed before that merge. The PR brief was amended before the
ordinary integration merge.

The merge had one conflict in the generated Gate C evidence index. Both
branches' authored records were retained; `python scripts/knowledge/render_navigation.py`
regenerated the index and a subsequent check verified each of the seven
conflicting or adjacent new record links appears exactly once, with no conflict
markers. The merge preview against PR #270 was clean; that separate UI pin's
initial candidate timed out and is not a dependency of the backup admission
repair. No side of the conflict was selected wholesale.

The only code changed by this package relative to merged main is the existing
installed sustained driver's selected backup-lane admission and its test.
The [earlier installed record](c-07-backup-lane-installed-20261107.md) retains
the actual selected and unselected eight-record runs and independent verifier
results. Main's changes since their starting baseline contained no paths under
`crates`, `Cargo.lock`, `Cargo.toml`, `rust-toolchain.toml` or `.cargo`;
the rerun's debug primary SHA-256
`7dd21bbe27d3be8a0c6a97a541fe31bebb3a5d22b92cc5349b4411587e9af022`
matches the previously tested primary. The runs are characterization, not
native capacity or soak qualification. The failed ARM campaign is unchanged.

On Windows with `CARGO_TARGET_DIR=C:\Users\jessa\AppData\Local\Temp\rrflow-c07-observer-7e60a4ac`,
`CARGO_INCREMENTAL=0` and `CARGO_PROFILE_TEST_DEBUG=0`:

- `cargo test --offline --locked -p rrflow-cli --test installed_reopen`
  passed 33, failed 0, ignored 14.
- `cargo clippy --offline --locked -p rrflow-cli --test installed_reopen -- -D warnings`
  and `cargo fmt --all --check` passed.
- `python scripts/knowledge/render_navigation.py --check`,
  `python scripts/ci/check_documentation.py` and
  `python scripts/ci/check_workflow.py` passed; the latter reported the
  merged five-workflow CI policy.

The candidate and protected merge-group CI for this exact combined revision
have not run yet. No result here resolves the failed ARM warmup conflicts,
latency overruns, native matrix, soak or project source visibility. No roadmap
or POA&M status changed. The code and linked records were reviewed for their
existing package; this merge changes no upstream source or runtime policy.

Change checklist: current and target behavior and failure oracle remain in
the earlier package records; exact starting revisions and PR were recorded;
the conflicting generated index and both sets of source records were
inspected and regenerated; the integrated package diff, local test and policy
checks were reviewed; the linked journal records unrun candidate, native and
soak checks. No external source adaptation or new trace was needed for this
mechanical reconciliation.

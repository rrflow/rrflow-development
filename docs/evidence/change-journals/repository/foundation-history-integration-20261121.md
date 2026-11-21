# Recovered foundation documentation integration

**Status:** active integration evidence; no compatibility or alpha qualification
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/foundation-history-integration-20261121`
**Owner:** source recovery under POAM-011, H-08 and repository authority

## Before-edit brief

Baseline fec02e3778ee5100c4b9975c56b03abd0c7b450a. Integrate PR24
f661cc965093f9ed4b3ae7a222da35c65599b43e, PR25
d134734bd765545475d5d6020b22749feae8045e and consolidation branch
73c75a852b0e576d135cda543103105cb5aa93ef through ordinary merges.
The first two define database identity and a PostgreSQL target, not working
compatibility. Current README, system overview, protocol contract, H-08 and
POAM-011 already own those requirements. Preserve later scope and evidence.

The consolidation branch's only commit proposes one engine, generated project
inventory and separate Connectome. Read its full contract, Python generator,
two tests and README change before implementation. Map inventory/digests to
current Git-based build_execution_inventory.py and governed engine inventory;
map one engine and client separation to RrdEngine and the pinned submodule.
ADR-0002 supersedes its mandatory shape-before-all-thinking sequence. Do not
restore the retired engine name/state directory, a duplicate crawler or a second
planning authority. Preserve useful deterministic-inventory behavior and tests
at the current owner if missing. No runtime rewrite or endpoint activation.

Check the smallest inventory tests first, then documentation, workflow and
format policy, followed by required candidate/queue gates. No new research or
runtime traces are necessary for historical source integration. Stop on lost
requirement, unknown behavior, generated ownership drift or red CI. This
package does not establish PostgreSQL compatibility or complete other archives.

## Resolutions and evidence

Ordinary merges: identity d779a9eb, PostgreSQL plan 7a5bbe23, consolidation
774d0bdb. Read the complete original consolidation contract, generator and two
tests; identity changes, plan changes and their conflicts; current README,
system overview, protocol contract, objective/roadmap dependency records,
POAM-011/013, ADR-0002, inventory git_files/current_record/records and tests.

The two identity branches add no application behavior. Today's owners retain
the native database, independent installation/release and explicit unqualified
PostgreSQL target, with newer multimodel/SQLite scope and project engram naming.
The old generated file-plan rows cannot overwrite the current retired descriptor
and Git-derived inventory. Older gate counts and active-item claims are not
restored. An automatically duplicated licence-evidence paragraph in POAM-013
was removed; the original current paragraph remains unchanged.

The consolidation branch's one-engine requirement maps to RrdEngine, client
separation to connectome/ and its public contract, and deterministic repository
shape to build_execution_inventory.py. ADR-0002 explicitly rejects requiring
a complete shape before exploratory thought. The retired engine contract and
crawler are not introduced as competing owners. Its useful repeatability,
sorted-path and build-noise exclusion oracle is carried into
ConflictFreeInventoryTests.test_inventory_is_deterministic_sorted_and_ignores_build_state.
The current suite already proves content changes invalidate recorded digests,
tampered/absent output fails, regeneration repairs it, and child checkout contents
cannot change the parent-pinned inventory. Old Markdown rendering and a separate
aggregate tree hash are not new runtime authorities; current per-file digests
and source-bound evidence are the owning representation.

`python scripts/ci/test_execution_inventory.py`: 12 passed, including the carried
oracle. No engine application changes or native compatibility claims. Full
policy checks and required candidate/queue results follow. Other archives and
Connectome native parity remain unfinished; no roadmap status is advanced.

Navigation generation, documentation policy, workflow policy and diff whitespace
checks passed. Candidate and merge-queue checks remain pending.


Candidate 44938940719 failed workspace_architecture (34 passed, 1 failed): this
journal quoted a retired product identity forbidden in active repository text.
The journal now names it descriptively; the original remains identifiable by
source commit. No architecture check or exclusion was weakened. Merge-blocked
issue 470 stays open until the repaired candidate lands. Joined green main
6119e22a before the repair. Revalidation and fresh candidate are required.

Repair validation: `cargo test --locked -p rrd-engine --test workspace_architecture`
passed all 35 tests with Cargo exit 0.

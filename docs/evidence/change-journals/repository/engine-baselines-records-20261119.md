# Records corrected to the owner's engine baselines

**Status:** active change-journal evidence; records and one architecture-test list entry; nothing measured

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/engine-baselines-records-20261119`

**Owner:** [engine baselines direction](../../../roadmap/engine-baselines-owner-direction-20261119.md)

**Starting revision:** main `557c3e3d5`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage and [kernel contract](../../../roadmap/kernel-contract.md) K9 and K10.
- **Current behaviour.** The system overview said rrflowCLD is "started from Parquet" and rrflowLSM "started from Fjall". The kernel contract said "the Parquet generation first". No record held a prompt structure.
- **Target behaviour.** The records state the owner's direction of 2026-11-19: Vortex is the rrflowCLD baseline, Parquet is a Lab comparison arm, Fjall is the baseline rrflowLSM must be proven against, and the four pillars are the minimum prompt structure.
- **Scope.** The three records under "Changed paths" and one new direction record.
- **Failure oracle.** `python scripts/ci/check_documentation.py`.
- **Research and trace decisions.** None; whether Vortex fits the dependency rules is the first step of the rrflowCLD package.
- **Stop conditions.** Stop if a correction would change a roadmap or POA&M status.

## What was read

Three read-only passes over `crates/` at the starting revision, each returning file and line for every claim: storage (`rrd-lsm`, `rrd-store`, the cold-history modules), query and vector (`rrd-query`, `rrd-vector`, the Lab evidence), and the prompt path (`rrd-contract` catalogue, `rrflow-mcp`, `rrflow-cli`, the context and flight engine modules). Five findings were then read directly: the default of `VectorSearchMode`, the `parquet` dependency of `rrd-engine`, the isolation statement in `rrd-lsm/src/transaction.rs`, the single `PromptFlightProvider`, and `[storage.cold_history]` in the install configuration.

## Changed paths

- `docs/roadmap/engine-baselines-owner-direction-20261119.md`, new: the owner's words, the engine mapped against them, and the approved order of five packages.
- `docs/roadmap/kernel-contract.md`: K10, the borrow-or-build row and resolved contradiction 3 name Vortex; S8 gains the default structure `prompt-minimum-v1`.
- `docs/architecture/system-overview.md`: the rrflowLSM and rrflowCLD rows.
- `crates/authority/rrd-engine/tests/workspace_architecture/retired_and_inputs.rs`: the direction record joins `LINEAGE_RECORDS`, because it quotes the owner naming the earlier product.
- this record and the regenerated indexes.

## Evidence

| Command | Result |
|---|---|
| `python scripts/knowledge/render_navigation.py` | indexes regenerated |
| `python scripts/ci/check_documentation.py` | passed |
| `cargo test -p rrd-engine --test workspace_architecture --locked` | 35 passed, after the candidate gate failed it on the first push (the record named the earlier product and was not yet listed) |

## Not proven

- Nothing was built, run or measured. The numbers in the direction record are quoted from the records it links.
- The findings of the three passes other than the five named above were not each re-read by hand.
- Whether Vortex meets the pinned-dependency and all-platform rules.

## Roadmap and POA&M

No status change. The Parquet statements in the superseded [one-engine joins plan](../../../roadmap/one-engine-joins-engineering-plan.md) and [second-mind program](../../../roadmap/second-mind-program.md) are left as history.

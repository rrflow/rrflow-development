# Two HNSW read-path oracles carried from the WIP branch

**Status:** active test evidence; no gate acceptance claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/e-04-hnsw-read-path-oracles-20261110`

**Owner:** E-04 vector generations; `docs/reference/vector/search.md`

**Starting revision:** main `a8551dc9`. Part of the leftover-branch absorption recorded in `repository/leftover-branches-absorbed-20261110.md`.

**Carried.** From `agent/vector-generation-read-path-wip-20261109`: `distinct_source_addresses_receive_distinct_hnsw_indexes` (two source addresses sharing a field receive distinct indexes) and `an_emptied_hnsw_generation_does_not_answer_for_another_collection` (the empty foreign-generation oracle). Both pass on main because #304's authenticated source identity repaired the defect the WIP branch had characterized.

**Acceptance.** ACCEPTANCE_PLACEHOLDER

**Research decision:** none. **Checks not run:** Linux/macOS, candidate CI. **Roadmap/POA&M change:** none.

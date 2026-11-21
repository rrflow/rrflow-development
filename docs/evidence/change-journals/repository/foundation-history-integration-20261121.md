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
restore vyrm-node, .vyrm state, a duplicate filesystem crawler or a second
planning authority. Preserve useful deterministic-inventory behavior and tests
at the current owner if missing. No runtime rewrite or endpoint activation.

Check the smallest inventory tests first, then documentation, workflow and
format policy, followed by required candidate/queue gates. No new research or
runtime traces are necessary for historical source integration. Stop on lost
requirement, unknown behavior, generated ownership drift or red CI. This
package does not establish PostgreSQL compatibility or complete other archives.

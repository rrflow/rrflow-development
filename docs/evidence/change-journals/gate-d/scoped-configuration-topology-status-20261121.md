# Align scoped configuration in the instance topology

**Status:** active documentation correction; runtime implementation unchanged
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/scoped-configuration-topology-status-20261121`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), PR456 configuration reference consistency

Following the [discovery-row correction](scoped-configuration-status-followthrough-20261121.md),
reading the global-user-configuration paragraph in `docs/architecture/instance-topology.md`
at `b615a01df231692135cbae8b96a09eba3c6771ab` found the same stale statement
split across lines: persistence and binding still required implementation.
The PR brief was extended before changing that paragraph. It now links the
bounded native numeric implementation while retaining project isolation and
the remaining broader policy, project-editing and UI gaps.

This is the same documentation consistency repair, with no runtime, schema,
research, resource or trace change. Read coverage is the topology's global
configuration and surrounding isolation paragraphs; the complete paragraph diff
was reviewed. The oracle is agreement with ADR-0006 and the implementation
reference without claiming complete policy support. Navigation, documentation
policy and whitespace checks pass before commit. No gate or POA&M item closes;
the final candidate and queue validation remains required.

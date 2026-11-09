# POAM-057 — comparison lab cannot establish exclusive scratch ownership

**Status:** active parent-owned POA&M detail  
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-057`  
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

## Observed deficiency

The current dirty repair changes Qdrant and SurrealDB to random names and
create-only preparation, retains their objects, and disables mutations for
Milvus. A further review found custom v1's unscoped teardown; its adapter is
now disabled for every operation and the small reference manifest marks it
unavailable. These are local candidate changes, not merged or CI-qualified
fixes. This does not make RRFlow shared-instance use safe, provide a cleanup
policy, fence replacement after create, or implement the resource-scoped custom
protocol.

This concerns the unmerged [comparison-lab candidate PR #253](https://github.com/rrflow/rrflow-development/pull/253), not a claim that the lab has landed on main. At `23c3f16e776f2cd5dae49130acdb26a100926ff3`, its Qdrant and Milvus adapters delete a deterministic same-named collection before creation and its SurrealDB adapter removes a table before definition. The RRFlow adapter alters schema and collection state without an established exclusive scratch boundary, while custom protocol v1 exposes destructive preparation and teardown without an ownership identity. A configured endpoint may point to a shared service. These effects can touch pre-existing data.

A disposable pinned Milvus 3.0.5 container returned success to a duplicate create and preserved the existing sentinel. A successful create response is therefore not sufficient evidence of exclusive ownership in that test. An existence check alone cannot close the race between checking and creating.

## Impact

Accidental collision, concurrent replacement, lost acknowledgments or insufficiently scoped cleanup can damage data or leave unidentifiable resources. The candidate must remain draft and must not be run against user services.

## Owning gates

J-04a / OBJ-10 and OBJ-11 own the lab's measurement boundary; J-04d owns comparison qualification.

Roadmap owner: [Gate J](../../roadmap/rrflow-1.0/gate-j.md). The package driver for PR #253 owns remediation.

## Closure evidence

Demonstrate a provider-specific exclusive scratch or disposable-resource boundary before any mutation, bind the attempt and authorization to recoverable identity, and ensure uncertain acknowledgments, collision, foreign replacement and replay cannot mutate or delete unowned resources. Prove cleanup only when ownership is enforceable, otherwise retain and report the object. Test concurrent runs, interruption, restart and readback using controlled servers and real isolated supported versions, including RRFlow, Qdrant, SurrealDB, Milvus and the revised custom contract. Show retained diagnostics do not expose credentials or sensitive server responses. Candidate and merge-group CI must pass at the actual integrated head. Documentation, a random name, or a prior green CI run is not this proof.

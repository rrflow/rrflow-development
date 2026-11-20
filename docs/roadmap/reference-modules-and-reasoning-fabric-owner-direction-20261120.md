# Reference modules and the reasoning fabric: owner direction of 2026-11-20

**Status:** active owner direction record; not an accepted decision, plan or status. The [kernel contract](kernel-contract.md) remains the one active engineering plan
**Coordinate:** `rrflow://rrflow-instance/data/work-package/reference-modules-and-reasoning-fabric-owner-direction`
**Owner:** records, verbatim, what the repository owner said on 2026-11-20 about Parquet, Vortex and Fjall as reference modules and about the engine as a reasoning fabric, and maps each part to what `main` holds; the [roadmap](rrflow-1.0.md) and [POA&M](../poam/rrflow-1.0-alpha.md) keep status authority
**Verified against:** `main` at `33159f8e2` on 2026-11-20, by reading the named files

This record continues the [engine baselines direction](engine-baselines-owner-direction-20261119.md) and corrects one line of it.

## The owner's words

Typos are the owner's and are kept.

> we need to make sure the work above is fully getting established correctly paraquet, vorttex, fjall need to be plugnplay modules for testing and reverse engineering and learning from them we need to truly ensure that our engine is true that rrflowKV rrflowQL rrflowDB rrflowCLD rrflowLSM turboquant are truely being potential fabric activated paths for reasoning and never rigid our schemas are minimum parameteres, requirements the constraints that the user has defined the schemas are behavioral and absorbing the latest bleeding edge methods that our dev mode will trace and allow us to optimize for efficiency and clear paths and finding and triggering the engine effectively allowing it to be tailored to absorb multiple fed data to a scratchpad and combine for answers and planning and different internal schemas based on the goal and intent of the prompt and user

## The correction

The 2026-11-19 record listed "Vortex as the product format" for rrflowCLD. That was the recording session's reading, and it was wrong. Parquet, Vortex and Fjall are all **reference modules**: none is the product. Vortex is the baseline rrflowCLD is measured against and learns from, as Fjall is for rrflowLSM. rrflowCLD and rrflowLSM are native.

## What a reference module is

A reference module is a proven outside implementation that can be plugged in beside the native one, for testing, reverse engineering and learning. It meets five conditions:

1. It is an optional build input behind a feature that is off by default. No product build contains it.
2. It sits behind the same port as the native implementation, so both receive identical inputs.
3. It answers the same transcripts line for line, so a difference in behaviour is a test failure.
4. It runs in the same comparison harness, so a difference in cost is a recorded number.
5. An architecture test fails if a product package declares or enables it.

| Module | For | On `main` |
|---|---|---|
| Fjall | rrflowLSM | Meets all five: `rrd-store` feature `fjall-baseline`, the `StorageEngine` port, `tests/fjall_baseline.rs`, `examples/storage_comparison.rs` weekly on Linux, and the Fjall rule in `workspace_architecture/storage.rs` |
| Parquet | rrflowCLD | As a module it meets all five since 2026-11-20: `rrd-store` feature `parquet-reference` behind the columnar port ([journal](../evidence/change-journals/gate-f/columnar-port-reference-modules-20261120.md)). Separately, it is still a dependency of the product: `rrd-query` writes the cold tier and the query export with it, on by default ([POAM-090](../poam/rrflow-1.0-alpha/poam-090.md)) |
| Vortex | rrflowCLD | Meets all five since 2026-11-20: `rrd-store` feature `vortex-reference` behind the columnar port, `vortex` `=0.93.0`, Apache-2.0. Built and tested on Windows; Linux and macOS are for the candidate gate |

rrflowCLD has no native format to compare either against: the cold tier is the Parquet writer.

## The engine as a fabric

| Direction | On `main` | Gap |
|---|---|---|
| rrflowKV, rrflowQL, rrflowDB, rrflowCLD, rrflowLSM and TurboQuant are paths a request may activate, never a fixed pipeline | `context-assemble` runs five stages in one fixed order (`ContextPlanSnapshot` requires exactly five), and records why each was selected or skipped. The vector planner picks an access path by cost. rrflowCLD is read by no query or context path. TurboQuant is off unless a request allows approximate search | The stage list is closed and ordered. Activation is not driven by the request's goal. Owned by [kernel contract](kernel-contract.md) K4 (planner) and S4 (trigger table), neither built |
| Schemas are minimum parameters, requirements and the user's own constraints; behavioural; never rigid | The four-pillar minimum is recorded as `prompt-minimum-v1` in S8. The contract's request types refuse unknown fields and use closed enums | S8 is not implemented. No schema is applied to a prompt |
| Schemas absorb the latest methods | No record | Open. Nothing states how a new method becomes a structure, beyond S8's "generated from examples" |
| Dev mode traces the paths taken so they can be optimized | The context packet's plan and evidence per item; write-path diagnostics in rrflowLSM; projected-read evidence | No one trace follows a request across the components. Owned by K1 (trace), not built |
| The engine absorbs several fed inputs into a scratchpad and combines them for answers and planning | No record and no code. A context packet is assembled per request and returned; nothing accumulates inputs across steps | Open. "Scratchpad" is new wording with no owner |
| Different internal schemas by the goal and intent of the prompt and user | The LFG observation carries an intent category, advisory only | Open. Nothing selects a structure from intent. S8 `scope` names a task kind but nothing fills it |

## Order

1. This record, and the deficiency it verifies.
2. One columnar port in the storage tier, with Parquet and Vortex as reference modules behind it and a comparison run. Done on 2026-11-20; all three modules are plug-and-play.
3. A native rrflowCLD format behind that port, so the product stops writing Parquet and POAM-090 can close.
4. The remaining Fjall capabilities ([parity work list](../reference/storage/rrflowlsm-fjall-parity.md)).

The fabric rows are the kernel contract's K1, K4, S4 and S8. Nothing here changes their order.

## Open

1. Whether the query export keeps a Parquet output. It is an interchange format a user asks for, not storage, but it is the product's other use of the Parquet crate.
2. What the scratchpad holds, for how long, and whether it is the task record, the prompt flight, or a new record.

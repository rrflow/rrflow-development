# POAM-012 — complete workload benchmark and failure corpus

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-012`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail record is part of the canonical RRFlow 1.0 alpha POA&M. The parent
ledger owns priority and lifecycle status; this file cannot change either
independently.

## Observed deficiency

There is no fixed-hardware end-to-end benchmark and failure corpus for the complete RRFlow
workload.

The current `rrflow-eval/src/main.rs::Trial` records paired provider trials;
it does not implement the full typed engine workload manifest, offered-load
schedule, physical resource series and capacity/failure protocol now specified
under [J-04](../../roadmap/rrflow-1.0/gate-j.md#capacity-and-optimization-qualification).
The proposed desktop/service/fleet/large-data envelopes are unmeasured.
J-04a through J-04d must supply schema validation, installed execution,
sustained interference/recovery and quality-matched optimization evidence.
Documented targets are not benchmark results and leave this item sequenced.

The draft comparison lab's adapter may include untrusted response text in an
error, and a run retains the first case error in its output. Credential values
are excluded from the manifest, but that alone does not prove a server cannot
echo them into an error. Before sharing reports across trust boundaries, the
lab needs a bounded, testable disclosure policy and an adversarial response
test. Per-request raw failure samples in the partial-run candidate retain only
timing and a fixed category; that does not repair the existing case summary.
See the [partial-run evidence](../../evidence/change-journals/gate-j/comparison-lab-truthful-partial-runs.md).

At main `3a72a6b4`, the provider-summary verifier accepted a synthetic report
claiming eight successes when all eight detailed rows reported failure and the
provider/repository lists contained duplicates. The provider runner's arm is a
label and its substring match is not an independent task oracle. The
[research and repair record](../../research/rrflow-adaptive-planning-measurement-architecture-research.md)
documents the failure, screening-only candidate and remaining treatment,
quality, provenance and held-out-evaluation gaps. The historical provider
summary lacks per-trial repository and raw-output evidence; it is not upgraded
into a quality claim. J-04 remains open regardless of screening-test results.

## Impact

Performance or competitive claims cannot be evaluated honestly.

## Owning gates

J-02 through J-05

Roadmap owner: [Gate J](../../roadmap/rrflow-1.0/gate-j.md).

## Closure evidence

Raw reproducible correctness, latency, throughput, memory, storage, recall, crash, and
deployment evidence is published.

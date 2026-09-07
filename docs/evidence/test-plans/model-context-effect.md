# RRFlow model-context effect test plan

**Status:** active acceptance test plan; no completed experiment is claimed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/test-plan/model-context-effect`
**Owner:** provider-neutral measurement of how RRFlow context changes model-assisted task outcomes

This plan measures one question: for the same declared task and project state,
what observable effect does an RRFlow-generated `ContextPacket` have on task
quality, resource use, and failure behavior? It evaluates the engine's context
selection; it is not a provider lifecycle, a prompt runner, a reasoning trace,
or a Connectome state machine.

The [engine data flow](../../architecture/engine-data-flow.md) owns context
assembly and correlated evidence. Gate H-05 owns instrumentation and Gate J-04
owns release-grade context benchmarks. A row or run becomes evidence only when
its raw output and provenance are retained at an exact source revision.

## Experiment identity

An experiment manifest is content-addressed and pins:

- task input bytes, expected outcome or evaluation rubric, and corpus digest;
- project-tree snapshot, relevant canonical-data stamp, schema/catalogue
  revisions, and dirty-state declaration;
- model descriptor, artifact or provider version, adapter revision, decoding
  controls, and declared context-window/tokenizer revision;
- tool/capability policy, network policy, authorization identity, timeout, and
  resource budgets;
- context intervention definition and every resulting packet/projection
  identity;
- evaluator implementation/model revision and decision thresholds; and
- trial count, arm order/randomization, warm-up/cache policy, machine,
  toolchain, and hardware/filesystem provenance when performance is compared.

Any changed pinned field creates a different experiment manifest. A display
may group related experiments, but it cannot pool their results silently.

## Declared intervention arms

Arms are versioned inputs in the manifest, not hardcoded runtime modes. A
useful first corpus normally declares:

1. **No injected RRFlow context** — the model receives the fixed task and
   declared tool policy without a `ContextPacket` payload.
2. **Engine-selected context** — `RrdEngine` assembles one bounded packet using
   the normal stamped planner.
3. **Declared comparison projection** — an explicitly identified broader,
   narrower, previous, or candidate projection is evaluated under the same
   task and resource policy.

The no-context arm does not claim that a provider session, model cache, or
external host is globally fresh unless the adapter can prove and record that
property. An arm may observe routing without injecting its results, but that is
a distinct intervention. Each injected packet retains its exact encoded bytes,
digest, stamp, selected/skipped avenues, sources, and truncation state.

## Execution and observable evidence

Every trial receives a unique identity under the experiment and records:

- arm, ordinal, randomized position, start/finish, and terminal status;
- RRFlow ingress, authorization, plan, graph/BM25/vector/DataFusion/cache work,
  packet, model/tool attempt, verification, and delivery correlations;
- exact model request/response envelopes permitted by retention policy, with
  secrets and sensitive content redacted through declared rules;
- provider-reported input, output, cached-input, or reasoning-token counts only
  when available, labeled with their provider and measurement semantics;
- RRFlow-estimated context tokens alongside exact packet bytes and tokenizer
  identity; estimates never replace provider accounting;
- latency, retries, timeouts, tool calls, failures, and engine physical-work
  counters; and
- evaluator inputs, output, confidence when applicable, explanation category,
  and digest.

RRFlow records only observable envelopes and typed outcome evidence. It does
not request, infer, reconstruct, or display hidden chain-of-thought. UI
animation or playback time is presentation state and cannot become benchmark
latency.

Provider adapters use structured process or protocol invocation without shell
interpolation, obtain credentials by secret reference, and report the exact
capabilities they can observe. Prompts and retained payloads must not be placed
in process arguments when a bounded standard-input or request-body channel is
available.

## Evaluation rules

Transport success, process exit zero, or a literal acceptance marker is not
task correctness. Each task declares a deterministic oracle where possible;
model-graded or operator-graded outcomes identify the rubric and evaluator and
remain distinguishable from deterministic verification.

The report retains individual trials and arm-level aggregates. It includes at
least success/failure counts, evaluator results, context bytes/tokens, model
tokens when observable, elapsed time, engine work, retries, and failure
categories. Latency claims require warm-up/cache disclosure and percentile or
confidence treatment appropriate to the trial count. One successful trial is
a trace, not a conclusion.

Promotion of a context policy or projection requires the minimum repetitions,
quality/non-regression thresholds, protected-recall checks, and rollback rule
declared before execution. Provider diversity is required only when the claim
is provider-general; otherwise the result is scoped to the tested model and
adapter. Failed, timed-out, denied, truncated, and malformed-output trials stay
in the denominator under the declared analysis.

## Persistence and presentation

Experiment manifests, trial identities, packet/evidence coordinates,
evaluator results, and retirement decisions are canonical rrflowDB records.
Large permitted envelopes may be content-addressed objects referenced by those
records. Temporary Arrow batches, provider sessions, and Connectome playback
state are not authorities.

Connectome may compare arms, filter trials, and replay the observable event
timeline through public RRD operations. Every value links to retained evidence
and identifies whether it is exact, estimated, provider-reported, or evaluated.
Client controls cannot launch an undeclared arm, rewrite completed trials, or
promote a policy without a separately authorized engine operation.

Retention or retirement follows estate policy and preserves experiment,
source, packet, result, and decision digests required to reproduce accepted
claims. A context-maintenance routine cannot discard failed trials merely to
improve an aggregate.

## Acceptance scenarios

The eventual harness must prove:

- identical manifests produce identical arm inputs and packet digests where
  the engine path is deterministic;
- arm ordering cannot change semantic inputs and declared cache differences
  remain visible;
- provider failure, retry, timeout, denial, malformed output, cancellation,
  and process restart retain one attributable terminal trial;
- incompatible project stamps, model manifests, tokenizers, evaluator
  revisions, or context packets cannot enter the same experiment silently;
- missing optional provider metrics remain missing rather than synthesized;
- context contribution and engine work can be compared with outcome evidence
  without exposing hidden reasoning; and
- close/reopen plus every supported public surface resolves the same
  experiment, trial, packet, and result identities.

No current package implements this complete harness. Passing context, provider,
or trace component tests alone does not satisfy the plan.

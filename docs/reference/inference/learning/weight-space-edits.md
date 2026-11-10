# Weight-space edit candidates

**Status:** active experimental method contract; complete control arm recorded and early stop passed; thinking arms not run, decision not reached and no RRFlow model edited
**Coordinate:** `rrflow://rrflow-instance/data/reference/inference/learning/weight-space-edits`
**Owner:** the weight-space edit candidate method, its replication protocol and its frozen decision rule

Weight-space programming is a technique from an early MIT CSAIL report: a
general-purpose LLM reads and edits another network's weights directly, without
gradient descent. This record decides whether RRFlow should spend more on it.
The [learning contract](controller-training.md) still owns datasets, experiments
and promotion. An edit is one more way to produce a candidate, under the method
extension that [OBJ-12](../../../objectives/rrflow-1.0-alpha.md#required-outcomes)
already allows. It adds no engine surface, operation, manifest or lifecycle.

## What the published evidence supports

The [authors' post](https://rikiyat.github.io/blog/weight-space-programming/)
and [release](https://github.com/RikiyaT/lm_understand-logs) were reviewed on
2026-11-02 at revision `4d09ba6d6a7e891dd8063974dfe81b3d78e3494f`. The strongest
language-model result is run `p50c`, which edited Qwen3.5-0.8B on a 50-pass
budget. RRFlow recomputed the release's per-problem results and rebuilt its
weights; the [package journal](../../../evidence/change-journals/gate-g/weight-space-edit-replication.md)
holds that evidence. A fresh control arm has run at full scale, but the release's
edited and unedited generations have not been repeated.

| Finding | Basis | Consequence for RRFlow |
|---|---|---|
| MATH-500 rose from 7.9 to 48.1 and GSM8K from 13.8 to 67.1. Problem-level regressions were nearly absent. | Recomputed from released per-problem files; not an independent generation. | The published effect is internally consistent and large enough to test. |
| The edit changed 130,188 entries, 0.016% of the model, but 10.6% and 8.1% of the two edited matrices by norm. | Rebuilt bit for bit from public weights. | Report matrix-level change. A small entry count does not show a small change. |
| The strongest result used 50 forward passes, a Python kernel and 20.3 hours elapsed, 4.7 of them waiting on an unreachable model. | Release run records. | Treat it as measurement-guided editing, with budgets, not reasoning alone. |
| The edit fixed runaway loops. Answers cut at the token limit fell from 93.0% to about 33% on MATH-500, yet edited answers still averaged 22,037 tokens. The editing model reported that 576 of its 579 rescued practice items had been truncated. | Recomputed release scores and the run's final report. | It removed an obstacle. The authors state that it could not add a missing ability. |
| The model card says the model "operates in non-thinking mode by default" and warns about thinking loops. The baseline was measured with thinking on. | [Model card](https://huggingface.co/Qwen/Qwen3.5-0.8B) at revision `2fc06364715b967f1860aea9cf38778875588b17`. | The release gives no non-thinking baseline, so RRFlow measured one. |
| With thinking off, the unedited model scored 44.6 on MATH-500 and 56.6 on GSM8K. The published edit leads by 3.5 and 10.5 points, with intervals above zero, while using about 6 and 10 times the tokens per answer. | Complete fresh control arm against released per-problem results, on different setups. | The edit keeps an advantage, but the setting alone recovers about 91% and 80% of its gain at far lower cost. Compare any weight edit against such a setting. |
| Gemma 3 270M, which rarely loops, did not improve. Other runs in the results table submitted no edit. Each post run is a single attempt per budget. | Release results table. | One success does not establish a repeatable method. |
| The release omits the editing harness and two graders, and its `mathqa.py` cannot be imported without an unreleased module. | Release README and code. | The editing process cannot be repeated from the release. The rebuild and the evaluation can. |
| Editing used a hosted model. The reading experiments sent it weights as text, and in the Qwen run it received whatever its kernel printed. | Release setup and post. | Information derived from the target's weights leaves the machine unless the editing model runs locally. |

The report is a blog post and states that these are very early results. RRFlow
found no peer review or independent replication on the review date.

## Boundaries if this advances

| Concern | Required boundary |
|---|---|
| Target | Only weights the operator holds. Hosted provider models cannot be edited. RRFlow admits no trained model today. The separate SWAG project holds an unqualified experimental checkpoint, so G-06a must first supply a qualified baseline and a real candidate with a diagnosed failure. |
| Fit to controllers | The published gain came from runaway reasoning. A controller's short, grammar-bound decision may rarely fail that way. This is an untested inference, and a candidate experiment must test it before relying on it. |
| Editing model | An admitted external activity with explicit egress authorization and budget, never a default dependency. Weights trained on project data may carry that data, so sharing them is a separate permission from context sharing or training consent. |
| Sandbox | Bounded code runs and forward passes, no network, and no bypass of the [experiment limits](controller-training.md#experiments-and-release). |
| Output | Fully bound bytes with new digests through existing pre-load admission. A weight diff is evidence and is not an activatable sidecar. |
| Evaluation | The G-06 held-out rules apply. Forward passes during editing use a development split, and the acceptance set stays unseen. Compare against a small fine-tune and a non-weight fix, on equal budgets. |
| Promotion | I-03 only. [The controller cannot promote its own weights](../lfg-swag.md#inputs-and-configuration-ownership), so self-editing stays a proposal. |
| Weight reading | A read-only audit can suggest a hypothesis about a candidate. The published sycophancy test failed from weights alone, so a reading never admits or rejects a candidate. |

## Replication protocol

The offline tool is `scripts/inference/replicate_weight_edit.py`. It tests the
published edit, makes no call to an editing LLM, and activates nothing. Run it on
a Linux GPU host. Operator-owned paths below are examples.

Pin these inputs: the release at the revision above, and from the model
repository at the revision above, `tokenizer.json`
(`5f9e4d4901a92b997e463c1f46055088b6cca5ca61a6522d1b9f64c4bb81cb42`) and
`model.safetensors-00001-of-00001.safetensors`
(`04b1c301231dd422b8860db31311ab2721511346a32cb1e079c4c4e5f1fe4696`). Install
`vllm`, `datasets`, `math-verify==0.9.0`, `latex2sympy2_extended==1.11.0` and
`antlr4-python3-runtime==4.13.2`. The tool refuses another `math-verify` version.
The published vLLM version is not recorded, so each run records its own. The
control arm used vLLM 0.33.0; the 0.20.1 in NVIDIA's `vllm:26.05` image does not
register this text-only architecture. On a host with no CUDA compiler, vLLM's
default sampler cannot start; set `VLLM_USE_FLASHINFER_SAMPLER=0`, which the run
also records.

Rebuild and prepare first. These steps need no GPU and fail closed on any
digest, problem or gold-answer mismatch. The sparse-edit archive reader accepts
only the declared unique NPY members, decodes them without pickle, and caps both
each array and aggregate expanded array data at 64 MiB. This bounds the local
parser; it does not make an unpinned source artifact trustworthy:

```bash
TOOL=scripts/inference/replicate_weight_edit.py
EXP=lm_understand-logs/exp8_math
PIN=04b1c301231dd422b8860db31311ab2721511346a32cb1e079c4c4e5f1fe4696
COMMON="--base-file qwen/model.safetensors-00001-of-00001.safetensors --tokenizer qwen/tokenizer.json \
  --config-dir $EXP/base_model/Qwen3.5-0.8B-text --expect-base-sha256 $PIN"
python3 $TOOL rebuild $COMMON work/unedited
python3 $TOOL rebuild $COMMON --edit-dir $EXP/runs/p50c/edit work/p50c
python3 $TOOL prepare --published $EXP/eval/unedited work/bench
python3 $TOOL regrade --arm $EXP/eval/p50c --set math500
```

Then run a smoke test. `--limit` arms can never reach a decision. vLLM writes log
lines to stdout, so the arm folder is the record and only the last line is JSON:

```bash
python3 $TOOL evaluate --checkpoint work/unedited --benchmark work/bench/math500.jsonl \
  --arm work/smoke --set math500 --protocol nothink35 --limit 4
```

Run the control first. It needs about 5% of the tokens of the two thinking arms,
and the published per-problem scores let it answer the main open question alone.
`--shard I/K` splits a set so an interrupted run loses only one shard; an
identical shard is not generated again.

```bash
for set in math500 gsm8k; do
  python3 $TOOL evaluate --checkpoint work/unedited --benchmark work/bench/$set.jsonl \
    --arm work/control --set $set --protocol nothink35
  python3 $TOOL compare --baseline work/control --candidate $EXP/eval/p50c --set $set
done
```

Run the two thinking arms only if the early-stop rule below allows it. By the
published mean token counts they need about 215 million generated tokens, of which
the edited arm needs about 76 million. Run the edited arm first:

```bash
for set in math500 gsm8k; do
  python3 $TOOL evaluate --checkpoint work/p50c --benchmark work/bench/$set.jsonl \
    --arm work/edited --set $set --protocol think35
  python3 $TOOL compare --baseline $EXP/eval/p50c --candidate work/edited --set $set
  python3 $TOOL compare --baseline work/control --candidate work/edited --set $set
done
```

If those comparisons already rule out `reproduced` or `edit_better_than_setting`,
the decision below can only be `stop`, so record that and skip the unedited arm.
Otherwise run it and decide:

```bash
for set in math500 gsm8k; do
  python3 $TOOL evaluate --checkpoint work/unedited --benchmark work/bench/$set.jsonl \
    --arm work/base --set $set --protocol think35
done
python3 $TOOL decide --published $EXP/eval/p50c --edited work/edited --base work/base --control work/control
```

Retain every arm folder and the decision output in a linked journal, including
a result that stops the work.

## Frozen decision rule

This rule was fixed before any complete replication run. Only a four-problem
smoke test had run, and the early-stop clause was added after seeing it; the
[package journal](../../../evidence/change-journals/gate-g/weight-space-edit-replication.md)
records both. A later change needs a new record and cannot reuse results already
observed. Comparisons pair the same problems, and intervals are 95% from the
problem-level standard error.

| Question | Outcome |
|---|---|
| Early stop: does the published edit beat a fresh control? | If the published `p50c` arm does not beat the complete control arm with an interval above zero on both sets, stop and record it. The expensive arms are not run. |
| Does the effect reproduce? | `reproduced` when, on both sets, the edit beats the unedited thinking arm with an interval above zero, and its accuracy is within 5 points of the published edited run. `partial` when it beats the arm but misses that margin. Otherwise `not_reproduced`. |
| Does it beat the documented setting? | `edit_better_than_setting` when the edit beats the unedited non-thinking arm with an interval above zero on both sets. `setting_better_than_edit` when that arm wins on either set with an interval below zero. Otherwise `no_demonstrated_advantage`. |
| Decision | `continue` only for `reproduced` with `edit_better_than_setting`. Everything else is `stop`: record the result and do not start a candidate experiment on this evidence. |

The complete control arm passed the early-stop check on 2026-11-03. That permits
the thinking arms and proves nothing further: the comparison pairs a fresh run
with the published scores, so only a same-setup edited arm can settle the
remaining questions.

`continue` means only that the published edit holds on this model and beats that
setting. It says nothing about RRFlow's controllers, activates nothing and
changes no gate status. The next step is then a G-06 candidate experiment.
Accuracy alone decides this rule, but each arm also records tokens and elapsed
time. Report that cost, because an edited model that spends far more tokens may
still lose on total cost.

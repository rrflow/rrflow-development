# Weight-space edit replication tool and contract

**Status:** active offline preparation; tool verified on real released data and a real GPU smoke test; no complete replication run
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/weight-space-edit-replication`
**Owner:** G-06a offline preparation; OBJ-12 learning method extension; driver: Claude Code session for the repository owner
**Pull request:** [250](https://github.com/rrflow/rrflow-development/pull/250)

## Baseline and change brief

Starting revision: `development/main` at `e6f3632b32f1c9be33c1951886a9d9a2be648ddf`.
The opening commit `3563e96a4a58c1abfeedbd44ec43e280bf44a8b0` was pushed to
`https://github.com/rrflow/rrflow-development.git`, ref
`agent/weight-space-edit-replication`, and the draft pull request and change
brief existed before any file was edited. The repository owner asked on
2026-11-02 how to bring weight-space programming into RRFlow, then approved this
package.

Previously RRFlow had no record of the technique and no way to test its claim.
The [new contract](../../../reference/inference/learning/weight-space-edits.md)
names it as an experimental candidate method, and
`scripts/inference/replicate_weight_edit.py` tests the strongest published result
under a frozen decision rule, with a control the post lacks. Engine, contract,
manifest, admission and Connectome behavior are unchanged. The tool calls no
editing LLM and edits no RRFlow model. Stop conditions were: any needed engine or
admission change, a reimplemented grader that disagrees with released results, or
a replication claim without a real complete GPU run.

## Inspection

Read in full: `README.md`, `AGENTS.md`, `docs/reference/inference/lfg-swag.md`,
`docs/reference/inference/learning/controller-training.md`, `spark.md` and the
learning index. Read in part, by named section: `change-authoring.md`, Gate G
controller learning packages, Gate I RROFLOW packages, the OBJ-12 row, POAM-009
impact, `model_manifest.rs` artifact descriptor, `review_outcomes.py` and
`review_decisions.py` helpers and their test idiom.

From the release at `4d09ba6d6a7e891dd8063974dfe81b3d78e3494f`, read as data:
the Experiment 8 README, `apply_edit.py`, `make_edit_diff.py`, `st_numpy.py`,
`merge.py`, `build_ckpt.py`, `evaluate.py`, `profiles.py`, the results excerpt,
and the grading sections of `mathqa.py`. None of the release's code was executed
or copied.

## Paths changed

Created `scripts/inference/replicate_weight_edit.py`,
`scripts/inference/test_replicate_weight_edit.py`,
`docs/reference/inference/learning/weight-space-edits.md` and this journal.
Changed `docs/reference/inference/learning/controller-training.md` and added one
test line to `.github/workflows/ci-reusable.yml`. Regenerated the learning and
Gate G indexes. Nothing was moved or deleted.

## Research and adaptation

Primary sources: the [authors' post](https://rikiyat.github.io/blog/weight-space-programming/),
their [release](https://github.com/RikiyaT/lm_understand-logs) and the
[Qwen3.5-0.8B model card](https://huggingface.co/Qwen/Qwen3.5-0.8B) at revision
`2fc06364715b967f1860aea9cf38778875588b17`. One
[third-party analysis](https://kingy.ai/blog/weight-space-programming-ai-agent-model-weights/)
was read for caveats; its claims were checked against the release where used.
A search found no independent replication on 2026-11-02.

The release's `mathqa.py` imports an unreleased `mcqa` module, so the grader could
not be used as shipped. Its documented behavior was reimplemented and then checked
against released results below. The release's edit file format is read as data.
The tool's core uses only the standard library because CI installs no Python
packages for this step. Only the sparse edit form is supported, which covers `p50c`
and `p25c`; the low-rank `p10c` is refused.

## Trace, resource and debugging decisions

There is no engine path, so no trace change applies. Each evaluate part records
package versions, device, platform, tokens, elapsed generation seconds and the
allow-listed sampler setting. The setting is allow-listed because other `VLLM_`
variables can hold credentials.

## Observed results

Host: Windows 11 with WSL2 Ubuntu, RTX 5090 Laptop GPU with 24,463 MiB. GPU runs
used vLLM 0.33.0, torch 2.14.0, transformers 5.20.2, math-verify 0.9.0 and Python
3.12.12, with `VLLM_USE_FLASHINFER_SAMPLER=0`.

| Oracle | Command or method | Result |
|---|---|---|
| Unit boundaries | `python scripts/inference/test_replicate_weight_edit.py` | 42 tests pass. |
| Tests can fail | A scratch script disabled each of 31 defenses in turn | 31 of 31 mutants killed. |
| Base identity | SHA-256 of the downloaded pinned file | `04b1c301…1fe4696`, equal to the repository's LFS digest. |
| Bit-exact rebuild | `rebuild` for `p50c` and `p25c` from public weights | Every tensor matched its published base and edited SHA-256. `p50c` changed 65,260 + 64,928 = 130,188 entries, with independently computed relative changes of 0.106362 and 0.081063 against the manifest's 0.10636 and 0.081061. |
| Real tamper refusal | One flipped bit in a real edit, a wrong base pin, and real `p10c` | All refused with exit 1 and no published output. |
| Benchmarks | `prepare` against the release | 500 and 1,319 problems. Identities and released gold answers match. SHA-256 `fcb033f5…` and `aa0ab152…`. |
| Grader | `regrade` of all released first samples for four arms and two sets | Extraction and grading both agree on 1,600 of 1,600. |
| Published scores | `compare` on released per-problem files | See below. |
| GPU path | `evaluate --limit 4` on all three arm types | All complete; see below. |
| Final bytes | `evaluate --limit 1` and the full `regrade` on tool SHA-256 `348887a0…` | Complete; regrade again 1,600 of 1,600. |
| Decision refusal | Real `decide` on the limited arms | Refused, exit 1: arm does not cover every problem. |

Recomputed from released per-problem files, not new generations:

| Edit | Set | Unedited to edited | Paired gain, 95% interval | Improved / regressed / tied | Mean tokens |
|---|---|---|---|---|---|
| `p50c` | MATH-500 | 7.90 to 48.10 | 40.20, [36.82, 43.58] | 306 / 0 / 194 | 30,868 to 22,037 |
| `p50c` | GSM8K | 13.84 to 67.10 | 53.26, [51.03, 55.49] | 916 / 4 / 399 | 29,289 to 11,993 |
| `p25c` | MATH-500 | 7.90 to 14.10 | 6.20, [4.53, 7.87] | 98 / 21 / 381 | 30,868 to 29,659 |
| `p10c` | MATH-500 | 7.90 to 12.35 | 4.45, [2.75, 6.15] | 87 / 29 / 384 | 30,868 to 30,084 |

The GPU smoke test used the first four MATH-500 problems at four samples each.

| Arm | Correct | Cut at limit | Tokens | Generation seconds | Tokens per second |
|---|---|---|---|---|---|
| Unedited, thinking on | 3 of 16 | 14 | 469,566 | 362 | 1,297 |
| `p50c`, thinking on | 12 of 16 | 3 | 214,868 | 206 | 1,043 |
| Unedited, thinking off | 12 of 16 | 3 | 37,093 | 40 | 927 |

Four problems cannot support a conclusion, and this table is not replication
evidence. It proves the path works. It also motivated running the control first.
By the published means, the two thinking arms need about 215 million tokens. The
control needs roughly 11 million, estimated from these four problems only. At the
rates above, that is about 46 to 64 hours and 2 to 3 hours on this host. Those
rates came from at most 16 concurrent samples with FlashInfer kernels disabled, so
a full batch may differ. One published shard reached about 12,400 tokens per second
on undisclosed hardware.

## Surfaced failures and corrections

- A first draft reported real `p10c` as an archive mismatch. The sparse-only check
  now runs first, so the refusal states the reason.
- Malformed archives, array headers and safetensors records raised non-`ValueError`
  exceptions. They now refuse cleanly, with tests. A leftover nonsense line in a
  test was removed.
- `prepare` originally could not detect a changed dataset until comparison, after
  hours of generation. It now checks against the release first.
- Rereading the tool found three more defects, all fixed: a parse exception on one
  odd answer could discard a finished shard, the grader version was checked only after
  the model loaded, and an empty selection ended in `ZeroDivisionError`.
- All three GPU arms first failed at engine start: `flashinfer` could not compile
  its sampler without `nvcc`. The documented `VLLM_USE_FLASHINFER_SAMPLER=0` setting
  resolved it. This may differ from the published setup, whose vLLM version and
  settings are not recorded, so every run records it.
- A smoke-result reader failed to parse the redirected output, and the author wrongly
  concluded that stdout was lost at engine shutdown and added a flush. Inspection
  showed the JSON was present on the last line after vLLM's log lines. The flush and
  its false comment were removed, and the contract documents the real behavior.
- The early-stop clause and the control-first order were added after seeing the
  four-problem smoke results. The thresholds did not change. A reader should weigh
  that the control's promise was visible when the order was chosen.
- vLLM warned that it casts 18 `linear_attn.norm` tensors from float32 to bfloat16.
  None is an edited tensor, and the warning appears in every arm.

## Checks not run

No complete arm has run, so nothing here replicates the published result or decides
the rule. Also not run: the release's editing process, which needs an unreleased
harness and a hosted model; the low-rank `p10c` rebuild; IFEval and the multiple-choice
sets, whose graders are unreleased; FlashInfer's default sampler; `vllm download-kernels`;
any evaluate path outside WSL2 Ubuntu; and any weight edit of an RRFlow model.

## Remaining errors and owner

No RRFlow-owned deficiency was found, so the POA&M is unchanged. The next action is the
complete control arm, which the repository owner decides because it occupies a GPU
for hours. G-06a still needs a real reviewed corpus before any candidate experiment.

## Review and status

The changed tool, tests and contract were reread in full and the complete diff was
reviewed before commit. Documentation navigation was regenerated. Change checklist:
owner records named, scope bounded, oracle recorded, research recorded, stop conditions
checked, none reached. Roadmap, objective and POA&M status: no change. Pull request, CI
run and merge evidence are carried by the pull request.

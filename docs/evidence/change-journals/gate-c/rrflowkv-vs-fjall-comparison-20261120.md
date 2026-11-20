# rrflowKV against Fjall: first disk-backed comparison, and the comparison in the weekly workflow

**Status:** active change-journal evidence; one machine, diagnostic, no parity or superiority claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/rrflowkv-vs-fjall-comparison-20261120`

**Owner:** [rrflowLSM against Fjall](../../../reference/storage/rrflowlsm-fjall-parity.md); continues [the restored baseline](../repository/fjall-baseline-20261117.md)

**Starting revision:** main `b63f7d447`.

## Change brief

- **Outcome and roadmap item.** OBJ-13 storage; [kernel contract](../../../roadmap/kernel-contract.md) K10, "LSM proven against Fjall at benchmark scale and in CI".
- **Current behaviour.** The comparison had run twice, in a debug build on Windows. Its ratio table left out the reads after maintenance. No workflow ran it. No record listed which Fjall capabilities rrflowLSM has.
- **Target behaviour.** The comparison reports the reads after maintenance, runs weekly on Linux on six profiles, and its first disk-backed results are recorded. A parity record lists every Fjall capability and the order the missing ones are built in.
- **Scope.** `crates/persistence/rrd-store/examples/storage_comparison.rs`, `.github/workflows/rrd-lsm-benchmark.yml`, six result files under `eval/results/`, the harness and CI records, and the new parity record.
- **Failure oracle.** The comparison exits non-zero if either engine fails full-corpus verification in any trial; the workflow job fails on a memory-backed data directory.
- **Research and trace decisions.** Fjall `3.1.8` and `lsm-tree 3.1.8` were read from their published source for the parity record. No profile was taken; the causes of the read loss are candidates.
- **Stop conditions.** Stop if the comparison needs Fjall in a product build.

## What changed

- The harness records the read latency after maintenance and adds `maintained_read_throughput` and `maintained_read_p95` to its ratios. Output format 2.
- The weekly workflow gains the job `rrflow-kv-vs-fjall`: six profiles, a refusal of `tmpfs`, every ratio in the job summary, raw output uploaded.
- [rrflowLSM against Fjall](../../../reference/storage/rrflowlsm-fjall-parity.md): 84 capabilities, 35 present, 22 partial, 27 absent.

## Result

x86_64 Linux under WSL2, 24 logical CPUs, data on ext4, release build, Fjall `3.1.8`, both engines syncing every batch. Each figure is rrflowKV divided by Fjall, the median of the trials. Seven trials per profile; three for `large`.

| Profile | Claims | Write rate | Read rate | Write p95 | Read p95 | **Read rate after maintenance** | **Read p95 after maintenance** | Recovery | Peak memory | Bytes reopened | Bytes after maintenance |
|---|---|---|---|---|---|---|---|---|---|---|---|
| small-batch | 2,048 | 1.06 | 0.99 | 1.01 | 0.99 | **0.71** | **1.40** | 0.21 | 0.83 | 0.69 | 0.13 |
| standard | 2,048 | 1.11 | 1.04 | 1.12 | 0.94 | **0.75** | **1.31** | 0.23 | 0.84 | 0.79 | 0.13 |
| read-heavy | 4,096 | 1.12 | 1.05 | 0.89 | 1.01 | **0.76** | **1.36** | 0.18 | 0.79 | 0.71 | 0.12 |
| sustained | 16,384 | 1.38 | 1.06 | 0.74 | 0.94 | **0.80** | **1.23** | 0.15 | 0.69 | 0.71 | 0.12 |
| extended | 70,000 | 1.40 | 1.12 | 0.75 | 0.74 | **0.81** | **1.26** | 0.14 | 0.64 | 0.72 | 0.12 |
| large | 1,000,000 | 1.02 | 14.88 | 0.52 | 0.05 | **0.64** | **1.59** | 0.16 | **1.43** | 0.27 | 0.58 |

Throughput above 1 favours rrflowKV. Latency, recovery, memory and bytes below 1 favour it.

- **rrflowKV is ahead** on write rate, recovery time and bytes on disk on every profile, and on peak memory up to 70,000 claims.
- **rrflowKV is behind** on reads after maintenance on every profile: 19 to 36 percent fewer reads a second, and a p95 23 to 59 percent higher. It is also behind on peak memory at 1,000,000 claims, by 43 percent.
- The read rate of 14.88 on `large` compares against Fjall before Fjall's own maintenance, on three trials. It is one observation, not a result to quote.

An earlier run of the five small profiles on the same day, with the format 1 harness, gave the same picture: reads after maintenance at 0.68 to 0.81.

## Surfaced failures

- The first run put its data on `/tmp`, which is `tmpfs` on this host. Sync cost nothing and rrflowKV appeared to write 2.4 times faster than Fjall. Those results were discarded, and the workflow job now refuses a memory-backed directory.

## Evidence

| Command | Result |
|---|---|
| `cargo build --release --locked -p rrd-store --features fjall-baseline --example storage_comparison` (Linux) | built |
| `storage_comparison --trials 7 …` on five profiles and `--trials 3 --operations 1000000 …`, `TMPDIR` on ext4 | six files, both engines verified in every trial |
| `python scripts/ci/check_workflow.py` | CI policy OK |
| `cargo fmt -p rrd-store -- --check` | clean |
| `python scripts/ci/check_documentation.py` | passed |

Raw output: `eval/results/2026-11-20-rrflowkv-vs-fjall-<profile>.json`.

## Not proven

- One machine, a host running other containers, and WSL2's virtual disk. No hosted-runner result exists yet: the workflow job runs after this lands.
- The parity record matched tests by name, and parts of Fjall's recovery and blob code were read at signature level.
- No profile was taken, so no cause of the read loss or the memory loss is confirmed.
- The AI-access matrix has still not run against Fjall.

## Roadmap and POA&M

No status change. The read loss and the memory loss are owned by step 1 of the parity record's order.

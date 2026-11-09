# C-07 selected-segment native characterization launch

**Status:** active launch evidence; outcome pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-selected-segment-native-launch`
**Owner:** OBJ-02 / C-07g,h / POAM-035 / [PR169](https://github.com/rrflow/rrflow-development/pull/169)

[PR169](https://github.com/rrflow/rrflow-development/pull/169) is an evidence-only package. Its baseline is public main `4c4ef33c49ad3799276787a4d6bcec18c39793c9` and opening commit `58dc91b37b3680694156f98c8a38c6c8c16a04a3`. It records an actual launch, not a workload pass. No source, previous failed artifacts, configured budgets or roadmap status changes. The driver is this session; stop if any source, allocation or receipt identity differs. The source candidate [PR168](https://github.com/rrflow/rrflow-development/pull/168) and its dependencies remain unmerged and require candidate CI and integration. Its linked journal preserves the earlier 100000-record failure and the new selected-segment component oracles.

## Frozen candidate and native receipt

The clean tested source commit is `c02f33c8254d5bbac95344a8dd6de5bf71c8cc96`, tree `7abd791b81482cab9150c92b8839e017cd6ec511`. A fresh copied primary executable passed installation preview and apply in a new bootstrap estate, then supplied authenticated public storage diagnostics. Release build and `cargo test --release --offline --locked -p rrflow-cli --test installed_reopen --no-run` passed; the copied executable and Cargo binary hashes matched. `cargo run --offline --locked -p rrflow-eval -- storage campaign validate <stage>/campaign.json` returned the canonical characterization identity below and explicitly ran no workload.

| Identity | SHA-256 |
|---|---|
| Primary executable | `f950ab3d223dbe6f137b74a7a25f05bbfa37ee3261b259b196a651ccb78d5257` |
| Configuration file | `fed73738ac4df13564f0cfc4c3f9b753b367aa9bd733823f157a4bb74951669f` |
| Fresh storage observation file | `5161076331176eddc850b1626b2b9dea769adc1fce35c844ab203ae432eb556a` |
| Campaign file | `b31928d0354ce38641714f1e12d988e6f46976cf43342ff0ecb3b3bd3184c842` |
| Canonical campaign identity | `67b400736f9951e5da38db468a670ca9582a1b06a868f7b79d08234aa460e12e` |
| Installed-reopen harness | `0b5b4aff3ff5dbd035de2a9ed59e9d72d53a65344ccba225009d52f2ae1613d8` |

At 2026-10-27 23:29:53 UTC the launcher started as PID 77524, invoking `python -u scripts/ci/run_sustained_capacity.py --campaign <stage>/campaign.json --configuration <stage>/configuration.toml --evidence <stage>/evidence`. Its actual native enforcement receipt records a Windows Job with 2147483648-byte committed-memory ceiling and four CPU equivalents, using a rate of 1666/10000 of 24 host logical CPUs rather than exclusive cores. Installed module memory is 64 GiB; no 8 GiB host limit is claimed. Same frozen 100000 records, 500000 edges, 100000 vectors of 384 dimensions, seed 7, two forced reopens, existing deadlines and ten-minute soak field are used. This warm-only characterization does not qualify the required full profile matrix or 24-hour soak.

A read-only early observation counted 4800 `corpus_acknowledged` events in `evidence/progress.jsonl`. The process was still running. This is initial setup progress, not independent full readback, measured throughput, terminal cleanup, or a pass. Other Rust compilation was observed on the host immediately before launcher start and was left running; the evidence worktree also performed brief Git and Python documentation checks during setup. Host isolation and an unbiased timing comparison are not claimed. The local stage `D:/localdev/devlabs/rrflow/target/c07-windows-selected-segments-c02f33c8` retains `prepare.py`, frozen metadata, bootstrap estate, launch logs, host-interference note and `evidence/`; hashes are identities, not claims that artifacts were uploaded. The eventual terminal outcome requires a successor evidence record.

## Review and limits

Read in full: the user-provided `AGENTS.md`, `docs/roadmap/rrflow-1.0-execution/change-authoring.md`, PR168's linked candidate journal and PR167's linked prior launch record. Relevant README/owner, runner, candidate tests, failed result and local campaign inputs were inspected as detailed in those journals. No code adaptation or new algorithm is introduced here. The native Job receipt and progress trace are preserved because they establish what actually ran, not whether the gate passed.

Documentation, workflow, navigation, staged diff and execution inventory checks are required before push. Native outcome, independent reopened readback, measured segments, cleanup, full native profile and 24-hour soak remain pending or unrun. No provider ingestion, nearest-neighbor index quality, build-plan implementation or alpha completion is inferred. No objective, roadmap or POAM status changes; exact candidate CI and merge remain pending.

Checklist: owner, exact source and stop conditions bound before repository edits; executable and fresh diagnostics frozen; configuration and native limits verified; prior failure retained; progress distinguished from completion; observed host activity disclosed; generated projections and full diff reviewed before push; terminal result left to a successor.

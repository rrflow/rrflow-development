# The missing lexicon is recorded as POAM-095

**Status:** active planning evidence; no code changed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/lexicon-deficiency-20261125`
**Owner:** [POAM-095](../../../poam/rrflow-1.0-alpha/poam-095.md) and the lexicon row of the [second-mind program](../../../roadmap/second-mind-program.md).

## Change brief

Baseline `95d4c69cc`. On 2026-11-25 the owner said the project is supposed to have a proper lexicon for the AI and the project that takes terms in, and a separate one for the second brain. Current behavior: neither exists; the second-mind program has listed the lexicon with "Nothing" in place since 2026-11-10 (`fd1d74c36`). Target: the deficiency is on the POA&M with closure evidence, and the program row names both lexicons. Scope: POAM-095, its ledger row, one row of the second-mind program, this journal. No code and no lexicon is written here. Stop if an objective or roadmap status would have to change.

## What was read

- `crates/`: no Rust file contains "lexicon". Files containing "dictionary" refer to the lexical index's term dictionary and to column dictionary encoding.
- Term lists written by hand: the system overview table (44 rows), `PLATFORM_TERMS` (25 terms, checked by `platform_terminology.rs`), Connectome's `glossary.tsx` (31 terms), the platform vocabulary note, and the kernel's trace vocabulary.
- The owner's Devagotchi hub exposes a dictionary and a per-person and per-project lexicon. Asked for the six component names, RRD, engram, second brain and DevUniverse, it returned no known phrase. Its dictionary holds general product boundaries and language stacks only.
- Not read: whether the five term lists contradict each other.

## Verification

- `python scripts/knowledge/render_navigation.py`, `check_documentation.py` and the `workspace_architecture` test; results in the pull request.

POA&M: POAM-095 added, High, Open. No objective or roadmap status changes.

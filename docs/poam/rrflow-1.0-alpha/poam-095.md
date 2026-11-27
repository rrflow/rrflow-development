# POAM-095 — no lexicon exists; terms are defined in five unconnected places

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-095`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Read on 2026-11-25 at `main` after pull request 515 merged. The owner stated the same day: "we are suppose to have a proper lexicon/dictionary for the AI and this project properly taking terms and having a separate one for the dedicated second brain/DevUniverse".

Neither lexicon exists. No engine operation, record kind or stored data holds a term with its meaning, and no code path takes a term in. A search of `crates/` for "lexicon" finds no Rust file; the files that say "dictionary" mean the search index's term dictionary and column dictionary encoding, which are unrelated. The [second-mind program](../../roadmap/second-mind-program.md) has listed "a lexicon that grows" with "Nothing" in place since 2026-11-10.

The project's terms are instead written by hand in five places that no check keeps equal:

| Place | Holds | Read by |
|---|---|---|
| [System overview, canonical component terminology](../../architecture/system-overview.md#canonical-component-terminology) | 44 terms with meaning and code boundary | people and agents |
| `PLATFORM_TERMS` in `rrd/contract/src/platform.rs` | 25 terms with a role; none of rrflowDB, rrflowKV, rrflowTX, rrflowCX, rrflowQL, rrflowMX | the public contract and its test |
| `connectome/src/features/runtime/components/glossary.tsx` | 31 terms with plain-language text | the Connectome screens |
| [Platform vocabulary note](../../history/platform-vocabulary-note.md) | an older term table | history |
| `rrflow-core/src/trace/vocabulary.rs` | trace names | the trace code |

One effect was seen on 2026-11-25: the alignment plan retired the `rrd-` prefix although RRD is the owner's name for the running daemon, and the owner had to correct it (pull request 515).

Not examined: whether the five places contradict each other term by term.

## Impact

An agent or a person meeting a project word has no single place that says what it means, so a word can be redefined or retired by mistake. The second lexicon, for the owner's own words across projects, has no place to live at all. [ADR-0006](../../decisions/0006-scoped-policies-and-preferences.md) calls SecondBrain and DevUniverse historical names; the owner still uses both, so that sentence does not reflect the owner's current words.

## Owning gates

H-09 owns the baseline lexicon among the prompt graph families. Kernel package K14 also names the lexicon.

Roadmap owner: [Gate H](../../roadmap/rrflow-1.0/gate-h.md).

## Closure evidence

Closed when (1) one project lexicon is the only hand-written source of the project's terms, and the contract's term list, the Connectome glossary and the overview table are generated from it or checked against it by a test that fails on a difference; (2) a public engine operation takes a term with its meaning and source into that lexicon and reads it back at a stamp on a real instance; and (3) a second lexicon, separate from any project's, holds the owner's own terms and is read by two different projects' instances in one recorded run.

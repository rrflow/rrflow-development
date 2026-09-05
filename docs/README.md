# RRFlow knowledge map

**Status:** active documentation index and migration scaffold
**Coordinate:** `rrflow://rrflow-instance/data/documentation-index/rrflow-knowledge-map`
**Owner:** documentation memory structure; linked from the repository root `README.md`

The checkout is the bootstrap form of RRFlow's project memory until the same
records can be imported into and resolved from rrflowKV. The repository root
[README](../README.md) is the product entry point. It links to one owning
record per subject instead of duplicating long-lived knowledge in one file.

## Record rules

Every active knowledge record must have:

- one stable, path-safe filename and one eventual `rrflow://` coordinate;
- a lifecycle status in its first 12 lines;
- one declared subject and owner, with no competing definition elsewhere;
- links to the code boundary, decision, roadmap item, and evidence it affects;
- explicit supersession links when replaced, rather than compatibility copies;
- machine-checkable identifiers that survive Markdown-to-rrflowKV ingestion.

## Record header and indexing pattern

An active record starts with this machine-readable shape:

```markdown
# Human-readable subject

**Status:** active <honest maturity or qualification>
**Coordinate:** `rrflow://rrflow-instance/data/<record-kind>/<stable-id>`
**Owner:** <the one subject this record is allowed to define>
```

The coordinate is an identifier, not a checkout path. It must be unique across
the classified memory tree. The record's nearest parent `README.md` must list
both that coordinate and its relative checkout fallback. Nested index records
follow the same rule and are linked from their parent index. This gives both a
future rrflowKV lookup and a working repository lookup without duplicating the
record body.

For a concrete current example, the [reference index](reference/) routes to
the [storage index](reference/storage/), which routes to the
[rrflowKV current-format record](reference/storage/rrflowkv-current-format.md).
The last record owns implemented bytes and tests only; it links back to the
architecture, roadmap, and POA&M instead of restating their decisions or
status.

A directory exists only when it owns at least one real record. Empty taxonomy
scaffolding is not added merely to make the tree look complete.

## Documentation taxonomy

| Directory | Owns | Does not own |
|---|---|---|
| `architecture/` | system context, boundaries, dependency direction, and end-to-end data flows | delivery order or test results |
| `decisions/` | accepted or superseded architecture decision records and rationale | speculative research |
| `objectives/` | measurable release outcomes and their acceptance conditions | delivery order or remediation tracking |
| `roadmap/` | versioned outcomes, ordered gates, acceptance criteria, and completion ledger | architecture definitions duplicated from their owner |
| `poam/` | observed deficiencies, risk, gate mapping, and closure evidence | aspirational features or a second delivery sequence |
| `reference/` | protocols, schemas, configuration, commands, and stable terminology detail | tutorials or planning |
| `guides/` | task-oriented installation, development, attunement, and troubleshooting procedures | normative architecture |
| `operations/` | deployment, observability, backup, recovery, and incident runbooks | product semantics |
| `research/` | source-backed investigations, comparisons, and unresolved findings | accepted decisions unless linked to an ADR |
| `evidence/` | generated or measured proof tied to an exact revision and gate | assertions without reproducible inputs |
| `history/` | superseded designs retained for provenance | active guidance or current status |

This separates explanation, decisions, reference, task guidance, delivery
planning, and evidence. It also gives each record a stable identity suitable
for later graph edges and retrieval.

The structure is grounded in the standard Cargo workspace/package layout,
DataFusion's recommendation to keep architecture close to source and extend
through explicit interfaces, Diátaxis's separation of explanation, how-to,
tutorial, and reference material, and MADR's one-decision-per-record model:

- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
  and [package layout](https://doc.rust-lang.org/cargo/guide/project-layout.html);
- [DataFusion architecture and extension APIs](https://datafusion.apache.org/contributor-guide/architecture.html);
- [Diátaxis documentation structure](https://diataxis.fr/);
- [Markdown Architectural Decision Records](https://adr.github.io/madr/).

## Active memory roots

| Subject | Owner |
|---|---|
| System architecture and detailed engine data flow | [`architecture/`](architecture/) |
| Accepted architecture decisions | [`decisions/`](decisions/) |
| RRFlow 1.0 alpha outcomes | [`objectives/`](objectives/) |
| RRFlow 1.0 delivery | [`roadmap/`](roadmap/) |
| RRFlow 1.0 alpha deficiencies and remediation | [`poam/`](poam/) |
| Stable provider-neutral contracts | [`reference/`](reference/) |
| Operations and CI | [`operations/`](operations/) |
| Platform industry research | [`research/`](research/) |
| Machine evidence | [`evidence/`](evidence/) |
| Superseded records | [`history/`](history/) |

The remaining flat `docs/*.md` files are an acknowledged pre-release
classification backlog. They must move in small, link-preserving batches only
after their active, historical, or superseded status is verified. Bulk moves
must not be used to imply that their content is correct.

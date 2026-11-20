# Typed System One controller through engine selection

**Status:** draft package evidence; implementation and acceptance pending

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-e/system-one-controller-20261120`

**Owner:** [kernel contract](../../../roadmap/kernel-contract.md), K07.

Starting revision: `081282df0af594ba3ded2554c9ee5acd2ab88308`.

The audit found that endpoint selection only prepares chat-completion requests.
Its narrowed allowance validation and worker native-adapter guard already exist.
This package adds an explicitly configured System One Choice wire protocol at
the same engine admission boundary, with real worker consumption as its oracle.
It does not establish generic internal reasoning decisions, provider quality,
cold recall, rrflowCLD suitability or rrflowLSM parity.

Baseline evidence on Windows, four Cargo jobs, isolated target directory
`D:/.usr/Temp/rrflow-selective-recall-audit-20261120/target`:

- `cargo test --locked -p rrd-core temporal::`: five selected tests passed.
- `cargo test --locked -p rrd-contract --test endpoint_proposal --test router_contract --test reasoning_tree_contract`: twelve tests passed.
- Worktree was clean; no open merge-blocked or Main-is-red issue was returned.

Primary research: [System One API](https://docs.typesafe.ai/api/system-one)
and [Choice](https://docs.typesafe.ai/primitives/choice), read 2026-11-20.
The native wire supports typed questions; endpoint and permitted effort can be
one choice, avoiding a dependent second request. This is a design decision,
not a latency or quality measurement. No upstream source is copied.

Scope: contract policy, prepared inference transport, engine proposal consumer,
focused and joined driver tests, generated contract surfaces and this evidence.
No protocol fallback, automatic retry, compatibility configuration, external
credential discovery or network activation is authorized by configuration alone.

Acceptance, complete reviewed-file inventory, diff review and CI evidence will
be recorded before this candidate is made ready. No roadmap status changes.

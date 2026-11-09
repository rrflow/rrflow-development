# Contributing to RRFlow

**Status:** active repository policy
**Owner:** how people outside the project report problems, suggest changes and propose code

RRFlow is pre-alpha. The most useful help right now is trying it and telling us
what happened, good or bad.

## Try it and report what happened

Follow [Try RRFlow from source](docs/guides/getting-started.md), then
[open an issue](https://github.com/rrflow/rrflow-development/issues/new/choose).
A useful report says:

- what you ran, what you expected, and what happened instead;
- the output of `rrflow version` and `git rev-parse HEAD`;
- your operating system and CPU architecture;
- the exact command and everything it printed.

Remove secrets first. Never attach a project's `.rrflow/` folder, which holds a
local credential, or any API key, token or private data.

Report a vulnerability privately, as [SECURITY.md](SECURITY.md) describes, and
never in a public issue.

## Suggest a change

Describe the problem you want solved in a feedback issue before proposing a
design. The [alpha objective](docs/objectives/rrflow-1.0-alpha.md) says what the
first alpha must do, the [roadmap](docs/roadmap/rrflow-1.0.md) says in what
order, and the [POA&M](docs/poam/rrflow-1.0-alpha.md) lists the gaps already
known.

## Propose code

Open an issue first. Terms for outside code contributions are not settled yet,
so a pull request without an agreed issue may not be mergeable.

Every change follows [AGENTS.md](AGENTS.md): one bounded package per pull
request, a change brief in its description, failing tests first where behavior
changes, a linked change journal, and candidate CI before the merge queue.
[CI operations](docs/operations/ci.md) describes the checks.

RRFlow is source-available under the [Business Source License 1.1](LICENSE).

# SDK and AI bridge plan expansion

**Status:** active planning evidence; implementation and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/one-engine-sdk-bridge-plan`
**Owner:** H-04, D-06, I-01/I-04 and existing repository change-authoring procedure

## Brief and baseline

[PR176](https://github.com/rrflow/rrflow-development/pull/176) carried the expanded scope before these edits. Starting branch head `2778b82e22eeecf218509fbfceded7cb53afdc71`, public development main `9dea4eae5f904ba479a2df1898987b807e276172`; the worktree was clean. The owner requested a complete SDK/bridge and Linux/Windows/ARM/macOS plan, then implementation in bounded packages. This change adds a supporting design, links it from the prior plan and architecture index, and records a newly verified SDK harness deficiency in POAM-011 without changing its status. It does not implement a collector or claim a platform, gate, release or provider integration is complete.

## Review and findings

Reviewed the previous plan and journal, Gate I, generated surfaces, repository change procedure and CI, POAM-006 and POAM-011, SDK file inventory, the engine's observation-source configuration, `scripts/ci/run_sdk_conformance.py`, and `crates/transport/rrd-client/examples/sdk_conformance_server.rs`. Public main was refreshed with `git fetch development main`. The existing five language SDK directories and Rust client do not establish complete generated models or provider ingestion. The engine module explicitly distinguishes configuration from collector activation. Public action gaps require canonical admission; the requested global option does not itself activate or authorize a source.

Research consulted the primary Codex and Claude Code telemetry documentation, Grok headless documentation, OTLP specification, Rust target list and GitHub-hosted runner reference, linked in the plan. Documented transports, content and version-specific coverage are distinct; no upstream implementation was copied. Provider access and runtime tracing were not attempted. Native qualification was not run.

On this Windows host a disposable Python `tempfile` was written and `os.chmod(p, 0o600)` applied; `stat.S_IMODE(p.stat().st_mode)` printed `0o666`. Source review shows the harness checks for `0600` unconditionally, and the Rust writer uses ordinary non-Unix file creation. This establishes a native readiness-check incompatibility and absence of an explicit protected DACL in that writer, not a demonstrated unauthorized read. The required repair must validate native access control, not bypass the check. `Get-Command cargo, uv, pnpm, go, mvn, dotnet -ErrorAction SilentlyContinue` found the first four and dotnet but not Maven; full six-client conformance was not run. A source-read command initially named a nonexistent `private_file/mod.rs`, and an `rg` command named a nonexistent example subdirectory; neither changed files or established an absence of relevant code.

## Validation and limits

- [x] PR description amended before edits with scope, ownership, sources, checks and stop conditions; POAM finding added to brief before its edit.
- [x] Project-default and explicitly enabled global routing, source authentication, boundaries, SDK/compiler ownership, provider-specific coverage, native matrix and sequenced acceptance recorded.
- [x] Navigation, documentation, workflow and execution inventory checked after generation; complete diff and whitespace reviewed before commit.
- [ ] No real provider was configured or invoked; no Windows SDK corpus, ARM/macOS execution, C-07 matrix or soak was performed.
- [ ] Candidate CI, merge and all product implementation remain separate; planning documents and schema do not enforce runtime behavior.

Created `docs/architecture/rroflow-sdk-and-bridge-plan.md` and this journal; changed the architecture index, parent plan and POAM-011 detail, with generated navigation and inventory. User content-scope selection remains open; metadata-first is a temporary planning assumption, not permission to activate capture. No objective, roadmap or POAM status was changed.

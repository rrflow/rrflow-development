# H-07 guided sharing and session resumption correction

**Status:** active context correction; integrated UX remains unqualified
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-07-guided-sharing-resumption`
**Owner:** OBJ-11, H-06/H-07 and POAM-037

## Brief and baseline

Starting revision `903cfe21005eb89d2987d60a61ce89a6ea11fcc7`, tree `ac233c4c32b3b239ee8c50e39704fd0ad6c16a44`.
[PR137](https://github.com/rrflow/rrflow-development/pull/137)'s audit brief was
extended before edits for the owner's correction: automatic local connection;
one simple offer to share project data/access and sync; automatic bindings,
valid-session reuse and resume after restart/offline operation; reconnect only
when needed. RRFlow.ai is the hosted access target, extending to rroflow.ai
recursive learning. Nontechnical users must not administer individual grants.

Before, the contract overemphasized separate choices, disabled defaults and no
account discovery even for returning users. After, fresh setup offers a clear
bundle, remembered authorization restores it, and only changed decisions need
interaction. Supported Claude/Codex/other clients must consume real project
context using individually qualified subscription or API modes. A limited
usable adapter set is acceptable first delivery; hidden billing substitution
or invented integrations are not.

Exact scope: README delivery passage; existing devpulse-cloud identity,
subscriptions, required user behavior and optional-login sections; Connectome
turnkey entry; Gate H-07 UX acceptance; POAM-037. This new journal and generated
navigation/inventory are the only new evidence. No runtime/API/version or gate
status changes, no implementation moves/deletes, no earlier journal rewritten.

## Inspection, research and oracle

Read relevant sections of README, devpulse-cloud, Connectome, Gate H and
POAM-037; inspected current PR brief and clean worktree. This is a targeted
owner-context correction, not a full code audit. The existing change-authoring
procedure and previous identity journal remain applicable. A guessed provider
file/directory search failed; actual repository file discovery corrected the
path assumption. Some broad search output was truncated; bounded reads supplied
the changed passages.

Primary sources opened on 2026-10-25:
[Codex authentication](https://developers.openai.com/codex/auth) and
[Claude account authentication](https://support.claude.com/en/articles/13189465-log-in-to-your-claude-account).
These distinguish supported authentication/billing modes. Design inference:
qualify provider-specific adapters and prefer supported client/tool integration
for subscription use; Nomen does not manufacture third-party entitlement.
No upstream code, undocumented token flow or provider API was implemented.

Trace/resource/debug instrumentation is inapplicable to context-only edits.
Oracle is consistent fresh-versus-returning behavior in existing owners:
automatic local attach, offered sharing, one confirmation, saved session reuse,
expired-session return, offline resume and changed-scope handling. Runtime
acceptance must prove a real supported AI client reads actual project context;
no such test was run or claimed here. Prior runtime and compatibility gaps
remain open under their owners.

Checklist: owner request and before/after scope in PR before edits; targeted
owner inspection; primary-source decision recorded; UX acceptance bound to
H-06/H-07; no second identity/lifecycle authority; immutable prior evidence;
linked journal and generated records; diff reviewed; results and omissions
recorded. Stop before runtime changes or unsupported subscription/login claims.
No Cargo/native/GUI/provider call or soak launched for this documentation change.
PR remains draft/unmerged; no CI or feature pass inferred from documentation.

## Validation

- `python -X utf8 scripts/ci/check_documentation.py`: passed; 613 statuses,
  611 coordinates, valid ownership, indexes and local links.
- `python -X utf8 scripts/knowledge/render_navigation.py --check`: passed;
  17 indexes, 612 nodes, 2440 edges, zero drift.
- `git diff --check`: passed. Full five-owner diff reviewed.
- Execution inventory regenerated to 1735 records; final `--check` before commit.

These checks validate the context correction; no runtime integration is claimed.

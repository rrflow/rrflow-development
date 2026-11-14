# H-07 Nomen identity and local-cloud priority

**Status:** active context correction; identity and synchronization remain unimplemented/unqualified
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-07-nomen-local-cloud-priority`
**Owner:** OBJ-11 delivery prerequisite, H-07 and POAM-037

## Brief and baseline

Starting revision `497b911f136c0418651dbf53b5890953ab386d0c`, tree `32ac4dd6334aa5449a3021eeacf12425ea299293`.
[PR137](https://github.com/rrflow/rrflow-development/pull/137)'s existing audit
brief was extended before edits for the owner's 2026-10-25 clarification.
Before: the cloud contract deferred shared identity and presented shippin.cloud
mainly as a devspace; Nomen was unnamed. After: Nomen is the central login
across products; local bidirectional synchronization to shippin.cloud is the
first hosted integration. Hosted engine/data and custom/self-hosted deployment
remain explicit targets. Shared login does not grant project/service access.

Exact changed scope: README product-role passage; devpulse-cloud service table,
identity and delivery subsection; Gate H-07 sequencing; POAM-037 identity gap;
this journal and generated navigation/inventory. No implementation moves,
deletions, runtime changes, new API, version change or gate closure. Existing
journals are preserved. Local engine correctness precedes hosted integration;
engine-only sync proof does not wait for DevPulse VM/container provisioning.

## Inspection and decisions

Relevant passages read: README identity/status/delivery; devpulse-cloud local
independence, service roles and learning handoff; Gate H-07 packages and H-08
compatibility boundary; seat-identity durable-seat and representation model;
POAM-011 account-sync gap; POAM-037 (full record); previous H-06 audit; execution
map and change-authoring procedure through journal template. The PR brief was
read before extension. Reads were targeted, not a fresh full engine audit.

`git grep -n -i nomen 497b911f136c0418651dbf53b5890953ab386d0c -- crates docs`
returned no matches. No Nomen-named immediate sibling directory was found in
D:/localdev/devlabs. Neither establishes absence outside the inspected scope.
The actual Nomen protocol, issuer/trust configuration, registration, credential
lifecycle and deployment contract remain to be identified from their owner.
No OIDC, OAuth or proprietary endpoint is invented or selected here.

Research decision: this correction captures the user's product authority, not
an externally inferable architecture. No new algorithm or upstream code is
adapted. Runtime traces, resource measurements and diagnostic instrumentation
are inapplicable to these documentation-only changes. Existing public APIs and
engine/seat authorization remain unchanged.

Oracle: all four changed owners agree on central Nomen login and local-to-
shippin.cloud-first delivery, retain local offline independence and configurable
custom hosting, and make missing runtime integration explicit. Readback and
whole-diff review confirm these clauses. A guessed Gate H path and PowerShell
literal wildcard search failed; actual paths/directory search corrected them.
An oversized initial read was truncated; targeted reads supplied the needed
passages. These were discovery errors, not engine test failures.

## Acceptance and remaining work

No Cargo/native tests, live login, sync transfer, UI launch or new soak was run:
no runtime changed and the owner's planning hold remains. Documentation checks
below validate context consistency only. H-07a resolves the Nomen contract;
H-07b/c/d/g implement and qualify actual account/estate binding, two-way transfer,
revocation and offline recovery. No status in the roadmap or POA&M is closed.
PR137 remains draft and unmerged; no candidate-CI or product-pass claim.

Checklist: outcome/owner and exact scope in PR before edits; existing behavior
and owner passages inspected; before/after oracle explicit; research and
trace/resource decision recorded; no new semantic authority; previous receipts
preserved; journal/index/inventory regenerated; diff reviewed; known missing
capabilities and checks not run explicit. Stop before runtime changes or any
claim of implemented identity/sync or comparative performance.

Validation passed:

- `python -X utf8 scripts/ci/check_documentation.py`: 612 document statuses,
  610 coordinates; ownership, indexes and local links valid.
- `python -X utf8 scripts/knowledge/render_navigation.py --check`: 17 indexes,
  611 nodes, 2437 edges; no drift.
- `python -X utf8 scripts/ci/check_workflow.py`: 3 workflows, 9 substantive jobs,
  5 engine suites, 20 default/6 optional packages, 19 pins.
- `git diff --check`: clean. Reviewed the full four-owner diff and journal.

Execution inventory regenerated for the final journal and checked before commit.
These are documentation results, not identity or synchronization runtime proof.

# H-05: native HTTP audit regression check

**Status:** active bounded validation evidence; candidate CI and full H-05 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-05-audit-http-validation`
**Owner:** OBJ-10 / H-05 / POAM-030 / PR144

The [audit page correction](h-05-audit-partial-page.md) was committed as
`8dd19d1b9797366ee26269e257044b13cbca14a8` on its draft branch. As additional
cross-boundary verification, Windows ran `cargo test --offline --locked -p
rrd-server --test http_process
initialized_security_authority_binds_sessions_and_denies_ungranted_routes`:
one passed, zero failed, 17 filtered out. The fixture starts a real HTTP server
against native rrflowKV, exercises authenticated audit read and export plus
denied operations, checks redaction and export digest, stops the server and
reopens its store. It is an existing process fixture, not the installed primary
CLI or a full UI/production workload.

No runtime or test source changed for this evidence. Read the complete fixture
and its entry-point/security setup; the preceding journal owns the change brief,
baseline failure, page-size tests, research and resource decisions. The raw
command result was observed directly; no omitted failure is relabelled a pass.
This record, generated journal index and execution inventory are the changed
paths. Candidate CI, POAM-029, full H-05 and the user's complete memory activity
experience remain open; no roadmap or POA&M status changes. PR/CI/merge evidence
remains pending. Checklist: exact source bound, real HTTP/store path identified,
command/result and limits stated, navigation/inventory checks required.

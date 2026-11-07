# Independent SWAG decision-service boundary observation

**Status:** active controlled observation; adapter and integrated qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/swag-decision-boundary-controlled-20261104`
**Owner:** G-01b/G-02a and POAM-009; PR249 / PR229

The separate SWAG checkout was clean at
`c60ae40a42c0928636cf925a4949bf9743a61821`. Reviewed its `AGENTS.md`, relevant
README sections, `tools/swag_decision_service.py::DecisionService.decide`,
the HTTP handler, `tools/swag_lfg.py`, and RRFlow's
`crates/compute/rrd-inference/src/endpoint_chat.rs::PreparedEndpointChat`.
SWAG's independent decision service exposes `POST /v1/decision`; the present
RRFlow selection adapter sends an OpenAI-compatible chat-completions request
and validates a bounded endpoint choice. Those interfaces are not the same
contract. SWAG's MCP LFG bridge is an independent advisory operation. No SWAG
source or runtime dependency is copied into RRFlow.

A local Python driver constructed SWAG's actual service with its shipped
template configuration and an owned loopback HTTP server. Against a synthetic
Rust project, it sent two requests with `execute_local` and
`execute_frontier` false. Both model-provider functions were replaced by
assertion-failing guards; neither was called. Both requests returned HTTP
200. Adding one synthetic `lfg` field did not change the four compared
response fields (`routing`, `counts`, `provider_plan`, `decision`). The
[localized receipt](swag-decision-boundary-controlled-20261104.json) is a
small, content-free summary; transient trace files remain under ignored
`target/`. This is two synthetic observations, not an assertion about all
inputs, a real LFG-guided SWAG choice, provider quality or performance.

The actual RRFlow/LFG/controlled-worker installed trial is separately
recorded [here](lfg-selected-worker-installed-controlled-20261104.md).
A real joined SWAG path needs an explicit, versioned and configurable
protocol boundary whose allowed inputs, candidate choices, failure semantics,
authorization, receipts and unknown fields are validated at the engine; it
must not silently map SWAG's question-selection output into an arbitrary
endpoint choice. Until a separately scoped adapter and matching service
contract are implemented and tested, this independent service is not proof
of RRFlow-to-SWAG routing. No roadmap or POAM status changes.

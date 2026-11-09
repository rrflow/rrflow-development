# Current worker candidate: installed model execution and recovery

**Status:** active installed developer evidence; candidate CI and UI integration pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-main-installed-live-20261106`
**Owner:** OBJ-06, G-01b/G-02a and POAM-009; PR229

## Source and scope

This follows the [main reconciliation](task-worker-main-reconciliation-20261106.md)
and the installed-readback acceptance work in
[PR229](https://github.com/rrflow/rrflow-development/pull/229). The clean native
source is `604c196df473105b041998e1266bb5727f846a04`. Its developer executable
SHA-256 is `76e2572ddaca05ce2ae00f434984b6006ece6d6b8f37e5ebf3153406a1c54951`.
The provenance-capture builder observed the source, Cargo inputs and compiled
unit closure and embedded the manifest. This is an unqualified Windows developer
build, not a signed distribution or performance result.

The input is the [actual PR task](task-worker-current-main-live-receipts/task-source.json),
including its captured head identity, rather than a synthetic benchmark task.
The [task projection](task-worker-current-main-live-receipts/task-projection.json)
records the complete 3,413-byte body and its digest; it was not truncated.
The existing explicit operator profile supplied one candidate and a worker
prompt requesting a short draft. Neither the response nor its choice was supplied
by a fixture. One eligible candidate does not measure comparative routing quality.

## Commands and failure history

The initial plain `cargo build` produced an executable without embedded source
provenance. Receipt preparation rejected the missing manifest; the attempted
installed driver then stopped at the absent receipt. No installation or model
call occurred. The failed directory and logs remain private and separate.

The corrected build used `python scripts/build_rrflow.py --profile dev --output
<new-private-receipt>`, with `CARGO_BUILD_JOBS=1`. Its two capture/embedding passes
succeeded and the manifest reported clean source at the revision above. The native
client and contract examples were built from that same checkout. Their paths and
credentials remain private; the recovery projection retains the copied native
client executable hash. The trial used the existing operator profile without
response-driven changes.

`scripts/inference/installed_controller_proof.py --worker --native-client
<built-task-model-client> --native-client-timeout-ms 180000` ran with the explicit
build receipt, contract helper, operator profile, actual task source and provider
credential references. It exited zero. The driver exercised missing credentials,
actual native selection and worker calls, retained attempt readback, process
restart and same-attempt replay rejection. Only owned child servers were stopped;
the existing developer workspace was not replaced.

An independent `--recover-proof <retained-trial> --output <new-readback>` also
exited zero without provider credentials or any requested model call. Engine
authentication still applied. Exact comparison verified the retained selection,
worker attempt and budget, the worker content hash, its task/configuration and
selection bindings, the copied engine/client executable hashes, and both replay
HTTP statuses of 412.

## Published evidence and limits

The [trial receipt](task-worker-current-main-live-receipts/trial.json) is a
byte-for-byte copy of the original result, including the actual model draft.
The [recovery projection](task-worker-current-main-live-receipts/recovery-projection.json)
omits the private proof-directory locator, retains the original receipt digest
and unchanged readback payloads, and labels the verification checks. Original
private receipts remain intact. Credential and private-location checks ran
before publication. An initial location pattern incorrectly matched an HTTPS
URL; its drive-letter boundary was corrected before any files were published.

The worker reported 1,301 prompt tokens and 222 completion tokens. These are
provider observations, not independently attested usage or quality labels.
The draft repeats implementation goals; it has not been independently reviewed
as a correct or useful result. No training data or model-quality approval was
created. No serving-memory measurement or loaded-weight attestation is claimed.

This advances installed selected-worker admission and recovery on the reconciled
code. The public UI still cannot launch a model-backed prompt flight; ordinary
source edits, successor plans, project effects, LFG/SWAG triggers and reviewed
learning remain open. Continue the same installed driver through that consumer
after this dependency passes candidate and protected merge CI. This evidence-only
increment changes no runtime/SDK source or roadmap completion status.

Reviewed the canonical build script, installed driver input/completion paths,
actual receipts and the prior installed journals. No new external research or
upstream adaptation was needed. Existing attempt/configuration digests and
bounded token/elapsed observations provide diagnostics; no new tracing authority
or runtime dependency was introduced. The journal, four linked receipt files,
POAM link and generated index are the only evidence additions.

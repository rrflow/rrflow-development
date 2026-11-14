# Selected worker through the canonical native client

**Status:** active client composition; installed native trial and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/task-worker-native-client`
**Owner:** G-01b / G-02a and H-04; POAM-009 / POAM-011; driver: active SWAG session

## Source, brief and ownership

PR229's native-client brief was recorded before mutation. Starting worker head:
`adc167691bdc4d1bd500e6db05809eee706761d4`; ordinarily merged PR222 dependency:
`c376e27126c0fd513f8ffeb1e8e17a8befe87602`. PR222 remains independently open.
The sole textual conflict was the installed proof runner's native-client and
worker modes. Both are retained; no history is rewritten or implementation
discarded. The earlier [installed HTTP proof](task-worker-installed-live.md)
remains bound to its original binary, runner and actual outcomes.

The existing typed worker method now delegates to `call_operation` rather than
cloning the legacy client and changing its retry count. The generated worker
marker implements the same sealed semantic interface as the proposal marker.
`TaskWorkerOutput::validate_request_identity` owns the request-known identity,
content and requested-context checks. One native transport, one executable
catalogue and the existing engine authorization/attempt ledger remain owners.
Client validation does not attest capabilities, loaded weights or output quality.

The one installed example is renamed from `controller_proposal` to
`task_model_call`, with explicit operation/project/address/request/timeout inputs.
Both calls share its locator/session bootstrap and closure. The existing Python
runner copies and hashes this example, records each native call's intent/result,
retains timeout/failure output and performs no automatic redispatch. Model calls
still go through the public engine API. Credential-free recovery stays read-only.

## Characterization and actual conformance inputs

The first composed client compilation failed on the obsolete private
`generated_operations` import. The first worker-identity test compilation failed
on the absent contract helper and a digest formatting assumption; both logs are
retained. Byte-wise hex formatting corrected the test without changing assertions.
The focused worker contract run then passed both tests.

`rrd-contract/tests/fixtures/worker-observed` contains the actual prior installed
worker request and HTTP response, with a manifest and byte-preserving Git rules.
Request SHA-256: `68161fff5a81c07c1ab12ef65c6fe4acb63563f21e6803a67d9fe5daee776cb3`.
Response SHA-256: `6a9902d0f1287298e53a72815ad08991345329540f0582e203d3566dc50c0b5f`.
These are public-task protocol inputs, not new provider observations or admitted
training/evaluation data. Controlled mutations of scope, task, attempts, selection,
configuration, content and context provenance must be rejected. The original
captured response passes; the test verifies fixture hashes before use.

The merged catalogue test now requires the worker marker to implement its
semantic interface. Existing actual server authorization and socket-loss tests
are preserved, including a worker call configured for eight general attempts
that receives no response and must send only once.

## Review and acceptance

Read applicable root/README and objective/POAM owners, PR222's native call and
example, the worker wrapper, contract validation/context hashing, transport and
catalogue projection sections, focused fixtures/tests and the installed runner.
The changed small modules/example and helper were read in full; large engine and
SDK documents were inspected at relevant sections. No external implementation,
new dependency, routing rank or model/provider default is introduced. Existing
bounded errors/hashes suffice; no additional unbounded tracing or performance
claim is justified.

Changed paths comprise the ordinary PR222 composition, contract helper and
captured fixture/tests, worker wrapper and generated-marker test, renamed example,
installed runner, Rust SDK/LFG references, POAM-009 and this linked journal plus
generated navigation/inventory. The old example source has one canonical
replacement; historical frozen artifacts and journals remain unchanged.
Seven invalid worker/native argument combinations exit 2 without creating an
estate. The merge has no remaining unmerged files.

Owner contract/client tests, strict all-target/all-feature Clippy, formatting,
Python syntax, generated-surface parity and repository checks gate publication.
The following local checks passed:

- `cargo test --locked -p rrd-contract -p rrd-client --all-features`: 189 passed,
  none failed or ignored, including the compile-fail semantic-interface example.
  Log SHA-256: `fc60ce7dba65b1a5e75886461069e14ce913e04c319d05201c2bb46c4dda516b`.
- `cargo clippy --locked -p rrd-contract -p rrd-client --all-targets --all-features -- -D warnings`:
  passed, SHA-256 `73ec0e5b5f59adcd82a2e800ba42eda605b4273e4feea674d2e6f6354cee9abc`.
- `cargo fmt --all -- --check` and Python compilation: passed.
- `python scripts/ci/check_generated_surfaces.py`: 72 HTTP operations, seven
  signal projections, unchanged OpenAPI digest `a0cc8919d190031c4e0e1493be97e66e6b9ae86479a407e29395dc66cee01f45`.
- `cargo test --locked -p rrd-engine --test workspace_architecture --all-features`:
  all 32 passed after staging the new test source. The first run retained its
  expected untracked-target failure; no assertion was removed.
- Navigation, execution inventory and its check, documentation policy and
  unstaged/staged whitespace checks passed. Inventory first caught the old
  indexed example path after its rename; staging the canonical replacement
  corrected the index before regeneration. The failed record remains retained.

Final staged whitespace inspection also detected CRLF in the byte-preserved
captures. Apply the repository's existing captured-fixture convention
`*.json -text whitespace=cr-at-eol`, with LF for the authored attributes file;
the observed JSON bytes and hashes remain unchanged. Staged whitespace then
passes. A premature native-build invocation correctly refused the still-dirty
source; no artifact or accepted receipt was produced by that invocation.

Raw commands, exits and logs are under SWAG's
`.swag/evidence/20261031/worker-native-client/`. Repository checks reported 853
navigation nodes, 2,203 inventory paths and 854 document statuses. The merge
diff and changed small files were reread; the scope/authority/validation checklist
is satisfied for this client increment. Broad engine behavior and model quality
are not newly qualified by client tests.

Build the native example from the resulting clean commit and retain its identity
separately from the installed engine before claiming an actual native trial.
Required candidate/protected CI, remaining worker faults, Connectome acceptance,
project effects, RROFLOW and reviewed learning remain open. No roadmap gate or
POAM closes; no full SWAG delivery, public release or model-quality claim applies.

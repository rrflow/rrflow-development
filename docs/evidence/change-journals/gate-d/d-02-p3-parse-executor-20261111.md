# D-02 P3: Parse executor

**Status:** active implementation series; D-02, OBJ-07 and POAM-059 remain open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-p3-parse-executor-20261111`

**Owner:** OBJ-07 / OBJ-14; [Gate D project-workflow packages](../../../roadmap/rrflow-1.0/gate-d.md#project-workflow-build-packages); package P3 of the [D-02 evidence-driven attunement plan](../../../roadmap/d02-evidence-driven-attunement-engineering-plan.md)

**Starting revision:** main `b28c5719`.

**Current behavior.** `rrd-attunement` parses one file into a flat list of named nodes (`parse_source`, `ParseProposal`) and the worker protocol carries that list. The Windows and Linux supervisors bound a child process, but nothing launches the parser through them. `EXECUTABLE_PHASES` is Connect and Inventory, so a derived plan never selects Parse.

**Target behavior.** Every Rust, Python and Go file an Inventory recorded is parsed by the primary binary's worker under native supervision. Definitions and references from each grammar's tag query, top-level spans and AST chunks are published as `source-symbol`, `source-span` and `source-chunk` records, with a per-file `source-parse` state, in the transaction that carries the job's checkpoint or lease renewal. A restart resumes from the last commit, an unchanged file is not parsed again, and a deleted file's facts are retired.

**Series.** The package lands as pull requests of at most 400 changed lines, each leaving main green: the parse step as its own function; symbol facts; native supervision for the worker; spans and chunks; worker protocol v2; the parsed-source records; the executor; enabling Parse with the oracles and the real run. Each adds its section below.

## The parse step as its own function

- **Change.** `parse::parse_tree` validates the request and returns the tree, grammar revision, start time and deadline; `parse_source` runs its unchanged walk on that tree. The facts extraction that follows reuses one bounded parse. No behavior change.
- **Oracle.** Characterization only: `tests/parse.rs` (7) and `tests/worker.rs` (6) pass unchanged.
- **Commands.** `cargo test --locked -p rrd-attunement`: 49 passed, 0 failed. `cargo clippy --locked -p rrd-attunement --all-targets -- -D warnings`: clean.
- **Not run.** Linux and macOS; candidate CI is the oracle there.

## Spans and chunks

- **Change.** `rrd_attunement::facts::source_facts` also returns top-level spans and AST chunks. Each top-level item, not a comment, is a `SourceSpan`. Chunks follow the cAST rule: syntax nodes merge up to the configured `chunk_bytes`, and a node larger than the bound splits into its children. Text no node divides further splits on line, then character, edges. Every fact is a byte range, so the engine slices text from bytes it read and verified itself and a worker can never supply source content.
- **Oracle.** `tests/facts.rs` adds five cases: top-level items become spans and comments do not; a small file is one chunk covering its code; chunks stay inside the bound and lose no code; one oversized token splits on character boundaries; delimiters the grammar hides stay in the chunks.
- **Commands.** `cargo test --locked -p rrd-attunement`: 60 passed, 0 failed. `cargo clippy --locked -p rrd-attunement --all-targets -- -D warnings`: clean. `cargo fmt --check`: clean.
- **Not run.** Linux and macOS; candidate CI is the oracle there. Nothing publishes spans or chunks until the executor lands.

## Worker protocol v2

- **Change.** The worker returns `SourceFacts` (symbols, spans and chunks) instead of the flat node list, and its request carries `chunk_bytes`. One worker now serves a whole batch, reading framed requests until the parent closes the pipes at a frame boundary, because starting the primary binary per file costs more than parsing most files. The parent validates every response against its request; a chunk longer than `chunk_bytes` is refused. The flat node proposal is removed in the next slice.
- **Oracle.** `tests/worker.rs` covers the new response and several requests on one pipe; `rrflow-cli --test parser_worker` drives the real primary binary; the Windows supervisor tests use the new frames.
- **Commands.** `cargo test --locked -p rrd-attunement`: 61 passed. `cargo test --locked -p rrflow-cli --test parser_worker`: 2 passed. `cargo test --locked -p rrd-engine --lib windows_supervisor`: 8 passed, 5 ignored. `cargo clippy --locked -p rrd-attunement -p rrflow-cli -p rrd-engine --all-targets -- -D warnings`: clean. `cargo fmt --check`: clean.
- **Not run.** Linux and macOS locally; candidate CI is the oracle there.

## Fact publication

- **Change.** `parse::records::file_mutations` turns one Step's outcome for a file into records. It retires every fact the previous state named that the new one does not, writes symbols, spans and chunks, and writes the file's `source-parse` state. Names and chunk text are sliced from the bytes the engine read and verified, so a worker's facts never supply content. A fact whose range lies outside those bytes or splits a UTF-8 character, or whose digest, language or parser revision differs from the state, is refused. A refusal or a lost worker retires the file's old facts and records why.
- **Oracle.** Four in-module tests over real `source_facts` output: a first parse writes facts sliced from the source; an edit retires every fact of the old content; a refusal retires old facts and records its reason; facts that do not match the verified bytes are refused. They were written with the function, not observed red first.
- **Commands.** `cargo test --locked -p rrd-engine --lib parse::records`: 7 passed. `cargo clippy --locked -p rrd-engine --all-targets -- -D warnings`: clean. `cargo fmt --check`: clean.
- **Not run.** Linux and macOS locally; candidate CI is the oracle there. Nothing calls `file_mutations` until the executor lands.

## Worker driver

- **Change.** `parse::worker::Worker` sends each verified source to one supervised worker and waits for its answer within a deadline. Two threads drive the pipes, so a worker that stalls, exits or breaks the protocol ends that file as `Lost` and is stopped instead of blocking the Step. `start` launches the engine's own executable under the Windows or Linux supervisor and refuses on other platforms.
- **Oracle.** Three in-module tests over in-process pipes with the real `run_worker` at the far end: a batch of two files parses on one worker; a stalled worker is lost at the deadline, stopped once and unusable after; a worker that exits or writes a malformed frame is lost. Written with the module, not observed red first.
- **Commands.** `cargo test --locked -p rrd-engine --lib parse::`: 6 passed. `cargo clippy --locked -p rrd-engine --all-targets -- -D warnings`: clean. `cargo fmt --check`: clean.
- **Not run.** Linux and macOS locally; candidate CI is the oracle there. A real supervised start is exercised by the executor's installed oracles.

## Commit path

- **Change.** `parse::publish::publish` commits one Parse Step: the batch's records and the job's transition in one transaction, or the transition alone when the batch changed no record, because a runtime commit must change something. An unfinished batch renews the worker's lease for the span it was granted, as the job contract requires of a running self-transition; a finished batch appends the Parse checkpoint, whose output digest summarizes every file's answer. `records` gains `schema_mutation`, `retire_file`, `state_of` and `summary_sha256`; `worker` gains an in-process worker so engine tests can parse without starting the primary executable.
- **Oracle.** Two more records tests: a removed file retires every fact and its state; states are found by their record and the summary changes with a file's answer. `publish` is exercised end to end by the executor's tests in the next PR.
- **Commands.** `cargo test --locked -p rrd-engine --lib parse::`: 12 passed. `cargo clippy --locked -p rrd-engine --all-targets -- -D warnings`: clean. `cargo fmt --check`: clean.
- **Not run.** Linux and macOS locally; candidate CI is the oracle there. Nothing calls `publish` until the executor lands.

## The Parse step's batch

- **Change.** `parse::step::execute` loads the committed Inventory and each file's `source-parse` state, then selects the files the policy binds to a grammar whose facts are missing or stale. It reads each through the anchored source reader, parses it on one supervised worker, and commits the batch with the job's transition. Each Step stops at the policy's file, input-byte and commit-byte bounds. A file larger than the file bound, or whose facts cannot fit any commit, is refused. Files that left the project or the bindings have their facts retired. A file the worker was lost on in this job is not retried until the next job, so one file that kills the worker cannot keep the job from finishing. At most three workers start in one Step. The AST chunk bound is fixed at 1,500 bytes until the policy carries it.
- **Oracle.** Two in-module tests of the selection rule: a final answer for the same content is not parsed again and an edit is; a lost file waits for the next job instead of looping. The end-to-end tests arrive with the dispatch in the next PR; locally they pass on the combined change.
- **Commands.** `cargo test --locked -p rrd-engine --lib parse::`: 14 passed. `cargo clippy --locked -p rrd-engine --all-targets -- -D warnings`: clean. `cargo fmt --check`: clean.
- **Not run.** Linux and macOS locally; candidate CI is the oracle there. Nothing calls `execute` until the Step dispatches to it.

## Running the Parse step

- **Change.** A Step whose selected phase is Parse, and whose plan step carries a parse policy, now runs `parse::execute`. A Parse step without a policy still fails as `phase-unavailable`, retryable, because it has nothing to parse with. Derived plans still never select Parse; enabling it is the last PR of the series.
- **Oracle.** Two end-to-end tests on an installed project with two Rust files, using an in-process worker. The first commits the facts with the Parse checkpoint and reads definitions and references back through rrflowQL before and after a restart. The second sets a one-file limit, so the first Step renews the lease without a checkpoint and the second checkpoints. They were written with the executor; the second first failed on the contract rule that a running self-transition must renew its lease, which the commit path now does.
- **Commands.** `cargo test --locked -p rrd-engine --lib`: 473 passed, 13 ignored. `cargo clippy --locked -p rrd-engine --all-targets -- -D warnings`: clean. `cargo fmt --check`: clean.
- **Not run.** Linux and macOS locally; candidate CI is the oracle there. A real supervised worker is exercised by the installed oracles when Parse is enabled.
- **Candidate CI finding.** The first candidate run failed on macOS only: `Worker::over` is reached there only from test builds, because macOS has no native supervisor, so the warnings-denied installed build refused it as dead code. Windows and Linux call it through their supervisors, which is why local runs passed. The fix allows it as unused only on platforms without a supervisor.

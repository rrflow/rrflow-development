# Find running instances and stop one

**Status:** active implementation evidence; proved on Windows only, Linux and macOS not yet run
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/instance-registry-and-stop-20261118`
**Owner:** [Gate D](../../../roadmap/rrflow-1.0/gate-d.md), installed lifecycle ([instance registry](../../../reference/deployment/installed-lifecycle.md#instance-registry), [stop request](../../../reference/deployment/installed-lifecycle.md#stop-request))

## Change brief

- **Outcome and package:** the installed lifecycle (Gate D, D-01 command tree). A client can find the instances this user installed or served, and can stop one.
- **Starting revision:** `3fb03316a` on branch `engine/instance-registry-and-stop`.
- **Current behaviour:** each served instance writes only `<project>/.rrflow/serving.json`. Nothing lists the projects a user installed or served, so a client must already know every project path. `rrflow serve` ends only on Ctrl+C. On Windows a client cannot send that to another process, so the only way to end a server it did not start is to kill it, which leaves the serving record behind. A stop-request file exists only on the separate `rrd-server` binary, with a completion file, caller-chosen paths and a literal 10 ms poll.
- **Target behaviour:** `rrflow install apply` and `rrflow serve` record the instance in a per-user registry. `rrflow instances` lists it with what each project records now. `rrflow stop --project <root>` asks the serving process to stop, waits until the project's serving record is withdrawn and reports the outcome. Every served engine, `rrflow serve` and `rrd-server` alike, ends on Ctrl+C or on that one stop request and runs one shutdown for both. The local process driver stops a managed instance with the same request.
- **Scope:** `rrd-engine` (`engine/instance_registry.rs`, `engine/serving.rs`, `engine/engram_control`), `rrd-server` (`stop.rs`, `process.rs`, `cli.rs`), `rrd-engram` (`local_process`, `rrd-deployment-catalog`), `rrflow-cli` (`command.rs`, `installed.rs`, `main.rs`), their tests and test harnesses, the installed-lifecycle and local-process-driver references, the getting-started guide, POAM-021 and POAM-088, `scripts/ci/selected_tests.py` and `scripts/ci/run_rrflowdb_standalone.py`.
- **Unchanged:** the readiness proof, the locator, the serving record's format, every HTTP operation, OpenAPI, the SDKs, installation plans and digests, and the version.
- **Failure oracle:** the new tests do not compile at the starting revision, because `InstanceRegistry`, `RrdEngine::request_serving_stop`, `ServingStopState`, `rrd_server::serve_published`, `LocalShutdown::StopRequest`, `rrflow instances` and `rrflow stop` do not exist there.
- **Research decision:** none needed. The registry location follows each platform's documented per-user directory and the XDG base directory rule that a relative base is ignored. No upstream code is adapted.
- **Trace decision:** extended. `serve.closed` now carries `stop_requested`, `serving_record_withdrawn` and `stop_request_removed`; `stop` and `instances` emit one completion event each; a registry that is not updated is a `warn` event and is also shown to the operator; a stop request that cannot be read is one `warn` event from `rrd_server::stop`.
- **Stop conditions:** stop if the registry would hold a credential or an endpoint, if a registry failure or a held registry lock could fail or hang install or serve, if a stop could be requested without write access to the project's `.rrflow` directory, if `rrflow stop` could report `stopped` while the addressed record is still published, or if the package would need a new HTTP operation.

## Change

- **Instance registry** (`engine/instance_registry.rs`, new). One private file per instance and project root, `<instance-id>.<sha256 of the root>.json`, in this user's state directory. The entry holds the format, instance ID, canonical project root, product version and whether a serving record was published. `InstanceRegistry::resolve` takes the explicit override, then `RRFLOW_INSTANCE_REGISTRY` (absolute only), then the platform directory. Writers and listers take a retained `.lock` file within a bounded lock wait; an entry is written through `write_private_anchored` beside its final name and moved into place with `AnchoredPath::rename_to`. `list` creates nothing, reads each registered project, and reports a missing project folder, a damaged entry, two folders installed as one instance, and a read made without the lock.
- **Stop request** (`engine/serving.rs`). `RrdEngine::request_serving_stop` publishes `.rrflow/stop-request.json`, which names the serving record it addresses. `ServingRegistration::stop_requested` reads it. `ServingRegistration::close` withdraws the record and then removes the request; dropping a registration withdraws the record too. `ServingStopRequest::observe` reads the project's serving record and answers `Pending`, `Stopped` or `Replaced`, and `wait` repeats that look on an interval up to a limit. A request is never taken back. Publication of a serving record removes a request left for an earlier process, and refuses an unusable request file by its own name.
- **One serve path** (`rrd-server/src/stop.rs`, new). `rrd_server::serve_published` serves until an interrupt or a stop request addressed to the published record, then closes the record. `rrflow serve` and `rrd-server` both call it.
- **`rrd-server`** publishes a serving record for a plain loopback HTTP listener, records the instance in the registry, and takes `--stop-poll-interval-ms`, `--instance-registry` and `--instance-registry-lock-wait-ms`. `--shutdown-request-file`, `--shutdown-complete-file`, `wait_for_shutdown_request`, `write_shutdown_completion` and the literal 10 ms poll are removed.
- **Local process driver** (`rrd-engram`). `LocalShutdown::RequestFile { request, complete, timeout_ms }` is replaced by `LocalShutdown::StopRequest { timeout_ms }`. The driver asks through a `ServingStop` the composition root supplies; `rrd-engine` supplies `EngineServingStop`, which is `RrdEngine::request_serving_stop`. A graceful exit is accepted only when the serving record is withdrawn; otherwise the authenticated kill follows as before. `LOCAL_DEPLOYMENT_FORMAT` is now 3 and the process record format 2, because the catalogue and the record carry the shutdown strategy.
- **`rrflow serve`** (`installed.rs`). Everything that can fail without the engine happens before publication. After publication every path reaches one shutdown: close the record, then bring the registry in line. The announcement gains an `instance_registry` member.
- **`rrflow stop`** and **`rrflow instances`** (`command.rs`, `installed.rs`), and the global `--instance-registry <dir>` and `--instance-registry-lock-wait-ms <ms>` flags.
- **`rrflow install apply`** records the instance and prints an `instance registry:` line. Under `--json` the answer is the engine's installation result with an `instance_registry` member beside it.

## Design decisions

- **One file per instance and root, not one registry file.** A writer never reads or rewrites another entry, so there is no read-modify-write to lose, and one damaged entry cannot hide the rest. The project root is part of the name so that a copied project folder, which keeps its original's instance ID, does not replace the original's entry. Rejected: a single JSON document; and a name from the instance ID alone, which silently dropped one of two such folders.
- **A lock as well as an atomic move.** On Windows, replacing a file while another process replaces or reads it can fail. The lock serializes the registry's own writers and listers. It is the installation lease's retained-empty-file pattern.
- **The lock wait is bounded, with no retry interval.** The operating system offers a lock that waits for ever and one that does not wait. A free lock is taken at once. A held one is awaited on a helper thread while the caller waits for the helper with the bound. Rejected: a blocking lock, which let a stuck holder hang `install` and `serve`; and a `try_lock` loop, which needs a second timing value.
- **A writer reads the project once it holds the lock.** Whether a serving record is published is read after the wait, not before it, so an installer that waited behind a closing server does not write a flag that stopped being true during the wait. Test: `a_writer_that_waited_records_what_the_project_shows_when_its_turn_comes`.
- **A lister never needs to write.** It opens the lock file and never creates it. Without a lock it reads anyway and says so, because entries are moved into place whole.
- **The registry is an explicit argument, never ambient in the engine.** `RrdEngine::apply_installation` does not write it; the adapters call `InstanceRegistry` after the engine operation, as the CLI does for the project map. Otherwise every engine test would write the developer's real state directory.
- **No substitute for `LOCALAPPDATA`.** An earlier draft fell back to `%USERPROFILE%\AppData\Local`. It is removed: without the platform's own variable there is no registry and the command says to name one. The test harnesses set `LOCALAPPDATA` to their temporary home.
- **The stop request is a file in the engram directory, not an HTTP operation.** The task asked for the request to be authorized by whatever protects the project's private state. A file in `.rrflow` is exactly that, needs no credential, and adds nothing to the operation catalogue. Rejected: an authenticated stop operation, which changes the public contract, OpenAPI and every SDK.
- **The request names the record it stops.** A request left for a killed process cannot stop the next server, even if a process ID or port is reused.
- **The withdrawn record is the completion, not the absent request.** Every requester of one record shares one request file, and a new server removes it, so its absence proves nothing. An earlier draft treated removal as the acknowledgement; two requesters, or a server restarted during the wait, then made `rrflow stop` report `stopped` for a live server. `observe` reads the serving record instead.
- **A request is never taken back.** The first draft removed the request when a wait ended, which removed it for every other requester too. A second draft kept that and had each waiting requester put the request back; requesters then wrote one file through one pending name at the same moments and failed each other. A stop that was asked for is not un-asked: the request stays until its server reads it, a `timed_out` answer says so, and the request names its record, so one left for a killed server stops nothing and the next server discards it.
- **A look that fails is looked at again.** The serving process removes the record and the request while requesters read them. A file removed during a look fails that look on every platform (it was opened, and its last link went before it could be admitted), and on Windows a file whose removal has not finished refuses to be opened at all. One read is therefore tried twice, `wait` repeats a failed look at its next interval and reports only a failure that is still there at the limit, and closing a record does not fail because the request it was tidying away was being removed by a requester at that moment.
- **`replaced` is its own outcome and exits 1.** The process that was asked has gone, but the project is served, which is not what the caller asked for. The new server is not stopped on the old request's authority.
- **`not_serving` exits 0 and `timed_out` exits 1.** Not being served is the state the caller asked for. A record still published at the limit is not.
- **One stop mechanism.** The first draft left `rrd-server` with its own request and completion files. They are removed, and the local process driver is moved onto the engine's request. `rrd-engram` sits below `rrd-engine`, so the engine's request reaches the driver through a small trait instead of a second copy of the file format.
- **Dropping a registration withdraws the record.** A serving process that fails after publication cannot leave a record that names it.
- **Polling with a named default, not a notification.** The workspace has no filesystem-notification dependency. Adding one for a five-times-a-second look was rejected.
- **`install apply --json` carries the registry outcome as a member.** A JSON client cannot read a stderr line. The member sits beside the installation result's own members, which `result_sha256` binds. Only a closing `serve`, which has no answer on stdout, reports a registry failure as a `warning:` line on stderr; the reference records that convention.

## Constants introduced

| Constant | Value | Kind | Justification or override |
|---|---|---|---|
| `DEFAULT_STOP_POLL_INTERVAL_MS` (`rrd-engine`) | 200 | tunable default | Five looks a second: one directory lookup each on an idle server, and a stop delay a person does not notice. Override: `--stop-poll-interval-ms` on `rrflow serve` and `rrd-server`, `rrflow stop --poll-interval-ms`. |
| `DEFAULT_STOP_WAIT_MS` (`rrd-engine`) | 60 000 | tunable default | A clean shutdown drains requests and joins the attunement follower after its current pass. Override: `rrflow stop --wait-ms`. |
| `DEFAULT_INSTANCE_REGISTRY_LOCK_WAIT_MS` (`rrd-engine`) | 5 000 | tunable default | A holder keeps the lock for one small write and one rename, so five seconds is far more than a healthy volume needs and short enough that install and serve report instead of appearing to hang. Override: `--instance-registry-lock-wait-ms` on `rrflow` and `rrd-server`, or `RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS`. |
| `INSTANCE_REGISTRY_ENVIRONMENT`, `INSTANCE_REGISTRY_LOCK_WAIT_ENVIRONMENT` | `RRFLOW_INSTANCE_REGISTRY`, `RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS` | names | The environment overrides; the flags take precedence. |
| `INSTANCE_ENTRY_FORMAT`, `STOP_REQUEST_FORMAT` | 1 | format invariant | Record format versions. |
| `LOCAL_DEPLOYMENT_FORMAT`, process record format (`rrd-engram`) | 3, 2 (were 2, 1) | format invariant | The catalogue and the record carry the shutdown strategy, whose shape changed. |
| `MAX_INSTANCE_ENTRY_BYTES` | 64 KiB (`MAX_LOCATOR_BYTES`) | format invariant | The engine's existing ceiling for a small non-secret record; an entry holds one path and three short values. |
| `.rrflow/stop-request.json`, `.rrflow/.stop-request.pending`, `<instance-id>.<sha256>.json`, `.lock`, `rrflow/instances` | names | format invariant | File and directory names. |

The stop request reuses `MAX_SERVING_RECORD_BYTES` (4 KiB), which already bounds the serving record it copies its fields from. No count limit, cap or threshold was added to product code. The literal 10 ms poll in `rrd-server` is removed.

Test-only values: 4 writers per project and 20 rounds in the engine concurrency test; 4 concurrent installer processes; lock waits of 0 and 150 ms for a writer meant to give up and 60 s or 120 s for one meant to wait; `--wait-ms 400 --poll-interval-ms 20` for the timed-out stop and `--wait-ms 120000` for the stop that outlives its server; a 3 600 000 ms stop poll interval for the driver's kill-fallback test.

## Review findings and what was done

An independent review of the first draft found the following. Each was reproduced in the code before it was changed.

| Finding | Disposition |
|---|---|
| `rrflow stop` could report `stopped` while the server ran: another requester's timeout, or a new `serve`, removes the shared request | Fixed. Completion is the withdrawn serving record (`ServingStopState`), and no requester removes a request another may be waiting on. Tests: `one_requester_giving_up_does_not_finish_or_undo_anothers_stop`, `a_stop_request_stays_in_place_until_its_server_reads_it`, `a_stop_request_for_an_earlier_record_does_not_stop_the_next_server`, and the `replaced` case of the process test. |
| A server interrupted between the requester's read and its publication made `stop` wait the whole limit | Fixed by the same change. Test: `a_server_that_ended_before_it_saw_the_request_is_reported_stopped`. |
| `timed_out` said the request was taken back without saying a server that had read it still stops | Fixed: the request is no longer taken back, and the answer and the reference say that a live server stops when it reads it. |
| The registry lock was blocking with no bound | Fixed with the bounded lock wait. Tests: `a_held_lock_never_hangs_a_writer_or_a_lister`, `a_held_registry_lock_never_hangs_install_serve_stop_or_the_listing`. |
| The shutdown-time registry update ran before the acknowledgement | Fixed: the record is closed first. The process test stops a server while the lock is held and gets `stopped`. |
| Three `?` returns between publication and withdrawal | Fixed: the build report and the runtime are made before publication, the announcement is inside the guarded block, and dropping a registration withdraws. Test: `dropping_a_registration_withdraws_its_record_and_no_newer_one`. |
| Listing created the lock file and failed on a read-only registry | Fixed. Test: `listing_changes_nothing_and_needs_no_lock_file`. |
| Two folders with one instance ID overwrote each other's entry | Fixed with the root digest in the name and a reported problem. Test: `two_folders_installed_as_one_instance_are_both_registered_and_reported`. |
| A relative `RRFLOW_INSTANCE_REGISTRY` was accepted | Fixed: refused with a contract error; the flag is resolved against the working directory. |
| The reference called `.rrflow` private although a retained one is not | Fixed in the reference and the engine comment: a retained directory's own permissions decide who may stop the instance. |
| An unusable stop-request file blocked `serve` with an error naming the serving record | Fixed. Test: `an_unusable_stop_request_refuses_publication_and_is_named`. |
| `rrd-server` kept a second stop mechanism and a literal 10 ms poll | Fixed: converged as described above. Tests: `rrd-server` `cli::tests` (three rewritten), `http_process` through the shared fixture, and all four `local_engram_driver` tests. |
| `RunningEngine::stop` killed the server on Windows and asserted a clean exit only on Unix | Fixed: an interrupt on Unix, `rrflow stop` elsewhere, and a successful exit plus a withdrawn record asserted on every platform. The same was done for the quantization harness's copy. |
| The `%USERPROFILE%` fallback | Removed; the four Rust harnesses set `LOCALAPPDATA`. |
| The cross-process concurrency test never wrote one entry from two processes | Fixed in `a_held_registry_lock_never_hangs_install_serve_stop_or_the_listing`: the test holds the lock as a barrier while a closed server and a replaying installer both wait to replace the one entry, then lets go and requires both to succeed and the entry to be whole. Only those two can contend for one project's entry: an installer and a server exclude each other while either has the engram open. |
| `install apply --json` reported a registry failure only on stderr | Fixed with the `instance_registry` member. |

Two further defects surfaced while fixing these:

- **A test wrote the developer's real registry.** The first run of the `rrd-server` process tests after the convergence started `rrd-server` with the inherited environment, which created `%LOCALAPPDATA%\rrflow\instances` with one entry for a temporary project. The fixture now passes `--instance-registry` and asserts on that registry; `scripts/ci/run_rrflowdb_standalone.py` names its own registry for the same reason. The three files the run created (`.lock`, the entry and the empty directory) were removed by name; the neighbouring `definitions` directory was not touched.
- **A forged process record in `local_engram_driver.rs` pinned process record format 1** as a literal. It was updated to 2 with the format.
- **A test of this package starved its neighbours.** A first version of the cross-process test retried four replaying installers against the non-blocking installation lease in a tight loop. Every `rrflow` start hashes its own executable, so the loop took the processor from the other fourteen tests of the suite, and their `rrflow ready` calls, which allow five seconds a request, timed out: the suite failed in three of five runs. The loop was removed and the lock is used as a barrier instead (above). Without the four instance tests the suite takes about 25 s here, with them about 28 s.
- **`engrams_and_history::a_served_engram_is_found_proved_and_not_served_twice` asserted the withdrawn record only on Unix.** The condition was removed; it holds on every platform now.
- **A server and its requesters removed and read the same files at the same moment.** In one full run `rrflow stop` ended with an error instead of an answer. A new engine test, `a_server_and_its_requesters_removing_the_same_files_together_never_fail`, reproduced it at once: a requester's look failed with `file link count denied: expected=1 observed=0` or `Access is denied`, and so did the server's removal of the request. The rules under "A look that fails is looked at again" fixed it; the test then passed in 12 of 12 runs of 40 rounds each.

## Evidence

<<EVIDENCE>>

## Files read

In full: `crates/authority/rrd-engine/src/engine/serving.rs`, `engine/instance_registry.rs`, `engine/tests/serving.rs`, `engine/tests/instance_registry.rs`, `engine/installation/lease.rs`, `engine/engram_control/drivers.rs` (to the backup driver); `crates/adapters/rrflow-cli/src/command.rs`, `installed.rs` and `main.rs`; `crates/adapters/rrflow-cli/tests/installed_lifecycle/main.rs` and `instances.rs`, `tests/support/running_engine.rs`; `crates/transport/rrd-server/src/process.rs`, `cli.rs`, `cli/tests.rs`, `main.rs` and `lib.rs`; `crates/authority/rrd-engram/src/local_process.rs`, `local_process/deployment.rs`, `lifecycle.rs`, `preparation.rs` and `driver.rs`; `docs/poam/rrflow-1.0-alpha/poam-088.md`.

In part: `engine/installation/storage.rs` (`publish_new_file`, `read_regular_bounded`), `engine/token_key/private_file.rs` and `private_file/windows.rs` (the lease open and its share mode), `engine/engram_layout.rs` (private directories, retained ancestors, `validate_recovery_engram`), `engine/engram_control/admin.rs` (driver construction), `crates/transport/rrd-server/src/http/server.rs` (`serve_until`), `crates/transport/rrd-server/tests/local_engram_driver.rs` and `tests/support/server_fixture.rs`, `crates/authority/rrd-engram/src/commands/rrd-deployment-catalog.rs` and `tests/deployment_catalog.rs`, `tests/installed_lifecycle/commands.rs`, `tests/installed_quantization.rs`, `tests/installed_reopen.rs` and `tests/installed_crash.rs` (their environments and stop helpers), `crates/transport/rrd-contract/src/attunement/installation.rs` (`InstallationResult`), `scripts/ci/selected_tests.py`, `scripts/ci/run_offline_lifecycle.py`, `scripts/ci/windows_offline.py` and `scripts/ci/run_rrflowdb_standalone.py` (their environments), `docs/reference/deployment/installed-lifecycle.md` and `local-process-driver.md`, and `docs/poam/rrflow-1.0-alpha/poam-021.md`.

## Not done and not proven

- **Linux and macOS are not run.** No test has a platform condition, but only Windows executed them. The one platform choice in a test helper is which clean stop it uses: an interrupt on Unix, `rrflow stop` elsewhere. The Unix half of that helper, the Unix lock behaviour and the macOS registry location were not executed here. The candidate CI matrix is the proof for the other two.
- **The local process driver still has fixed timings of its own**, a managed instance is registered only when its deployment names a registry, and a listener that is not plain loopback HTTP cannot be asked to stop. Recorded as [POAM-088](../../../poam/rrflow-1.0-alpha/poam-088.md).
- **Nothing removes a registry entry.** Uninstall (D-11) owns that, with `.rrflow/stop-request.json` and its pending file.
- **Install recovery does not admit the stop-request file.** `validate_recovery_engram` lists the entries allowed in a `.rrflow` that RRFlow created. It does not list `serving.json` today, and it does not list the stop request either. An interrupted install is not being served, so no case was found where this matters. The one neighbouring case that was tested is a replayed `install apply` beside a killed server's record, which succeeds.
- **A stop request is authorized by the directory, not by its writer.** For a retained `.rrflow` that may be wider than the user who serves the instance. The reference states it; the request is not written or read as an owner-only file.
- **Tools that run the real binary with the developer's environment** (for example `scripts/dev/connectome_dev.py` and `scripts/inference/installed_controller_proof.py`) write the developer's real registry. That is what the registry is for, but nothing prunes those entries.
- **Connectome is not changed.** It does not yet call `rrflow instances` or `rrflow stop`, and it has not been run against the `install apply --json` answer with its new member.
- **Not run:** the workspace-wide `cargo test` and `cargo clippy --workspace`; the offline lifecycle qualification (`run_offline_lifecycle.py`), which runs the four process tests under isolation; `scripts/ci/run_rrflowdb_standalone.py` after its one-line change.
- **No pull request exists yet**, so this brief is not on one. Nothing is committed or pushed.

## Roadmap and POA&M

No roadmap item changes status. [POAM-088](../../../poam/rrflow-1.0-alpha/poam-088.md) is added for what remains around managed instances. [POAM-021](../../../poam/rrflow-1.0-alpha/poam-021.md) is corrected where it said the driver treats file existence as shutdown completion.

## Links

- [Installed lifecycle: stop request](../../../reference/deployment/installed-lifecycle.md#stop-request)
- [Installed lifecycle: instance registry](../../../reference/deployment/installed-lifecycle.md#instance-registry)
- [Local process driver](../../../reference/deployment/local-process-driver.md)
- [H-07h served endpoint record](../gate-h/h-07h-served-endpoint-record.md)

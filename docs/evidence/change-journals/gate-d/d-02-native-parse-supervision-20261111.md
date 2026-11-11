# D-02 P3: native supervision for the parse worker

**Status:** active implementation evidence; the launch path is unused until the Parse executor lands; D-02 and POAM-059 remain open

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-02-native-parse-supervision-20261111`

**Owner:** OBJ-07 / OBJ-14; package P3 of the [D-02 evidence-driven attunement plan](../../../roadmap/d02-evidence-driven-attunement-engineering-plan.md); one pull request of the [P3 series](d-02-p3-parse-executor-20261111.md)

**Starting revision:** main `a75a47d6`, after #334.

**Current behavior.** The Windows supervisor puts a child in a job object with a memory cap and kill-on-close, and passes it only its two pipes through an explicit handle list. Its own header names the open stop conditions: the temporary inheritable pipe duplicates could leak to a concurrent launch that inherits every handle, and no executable identity is admitted. The Linux supervisor caps address space and ties the child to its supervising thread, but cannot pipe a worker, and its drop waits at most five seconds, so a child stuck in an uninterruptible wait is never reaped.

**Target behavior.** These are the D-04 supervision stop conditions P3 names.

- **Handle inheritance.** `rrd_core::child_launch_lock` is held across every child launch in the engine: the Windows supervisor's pipe creation and process start, and both launches in `rrd-estate`'s local process driver. No sibling launch can run while the duplicates are inheritable. A host that embeds the engine and launches children without the lock is outside the guarantee, as the header says.
- **Executable identity.** On Windows, `capture_primary_executable` records the volume and file index of the engine's executable when a persistent engine is composed. `spawn_worker` opens the current executable without write or delete sharing, compares its identity and refuses a mismatch, and keeps the file open until the process starts. On Linux the worker is `/proc/self/exe`, the running image itself.
- **Environment.** The worker starts with an empty environment: an explicit empty block on Windows, `env_clear` on Linux as before.
- **Reaping.** On Linux, a child not reaped within five seconds of the kill is handed to a `rrflow-parser-reaper` thread that blocks in `wait`. On Windows the job object's kill-on-close already owns the child.
- **Memory.** Unchanged: the job memory limit on Windows, `RLIMIT_AS` on Linux. On other platforms there is no supervisor, and Parse will report the capability unavailable.

**Changed paths.** `crates/kernel/rrd-core/src/lib.rs`; `crates/authority/rrd-estate/src/local_process.rs`; `crates/authority/rrd-engine/src/engine/{attunement.rs, core.rs, attunement/windows_supervisor.rs, attunement/linux_supervisor.rs}`; this record.

**Oracle.** No new test in this pull request: `spawn_worker` has no caller until the executor lands, and its proof is the installed binary. The existing supervisor tests (`native_job_transports_framed_parser_request_without_unrelated_handle_inheritance`, `native_job_runs_primary_binary_parser_worker`) pass unchanged through the locked launch path. The executor's pull requests turn on `installed_binary_replans_and_reads_receipt_after_forced_restart(true)`, which runs the worker from the installed release binary.

**Acceptance.** Windows, own target directory: `cargo clippy --locked -p rrd-engine -p rrd-core -p rrd-estate --all-targets -- -D warnings` clean; `cargo test --locked -p rrd-engine --lib attunement` 64 passed, 0 failed, 8 ignored; `cargo test --locked -p rrd-estate -p rrd-core` 121 passed, 0 failed.

**Research decision:** Windows handle-list inheritance and job objects as documented for `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` and `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`; serialising launches around inheritable handles is the mitigation Microsoft's "programmatically controlling which handles are inherited" note gives. No upstream code.

**Checks not run:** Linux compilation and tests locally (no Linux toolchain on this machine); candidate CI is the oracle for `linux_supervisor.rs`. macOS has no supervisor by design.

**Roadmap/POA&M change:** none.

# Two serving races behind the red nightly matrix, repaired

**Status:** active change-journal evidence; both causes fixed and tested on Windows; the Linux and ARM64 proof is the candidate gate and the next nightly run

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/serving-record-replace-and-interrupt-order-20261120`

**Owner:** the alarm [Main is red: nightly full matrix](https://github.com/rrflow/rrflow-development/issues/436); [POAM-088](../../../poam/rrflow-1.0-alpha/poam-088.md)

**Starting revision:** main `aeb76407b`.

## Change brief

- **Outcome and roadmap item.** OBJ-12 installed lifecycle; H-10. An open `Main is red` alarm is repaired before new work.
- **Current behaviour.** The nightly matrix of 2026-11-20 on `c158a0667` failed one test, `instances::a_registered_instance_is_listed_served_and_stopped_cleanly`, in two places: on Windows, "a stop reported success while the project is served"; on native Linux ARM64, "rrflow serve failed during shutdown" with an empty error stream.
- **Target behaviour.** A stop waiting on a killed server's record is told the project is served again when another server starts, never that its stop finished. A server interrupted the moment it announces itself shuts down cleanly.
- **Scope.** `crates/authority/rrd-engine/src/engine/serving.rs`, `crates/transport/rrd-server/src/stop.rs`, `process.rs`, `lib.rs`, `crates/adapters/rrflow-cli/src/installed/serving.rs`, and two tests.
- **Failure oracle.** A reader that looks at the serving record throughout 300 replacements of a stale record and must never find none; an interrupt sent to the test process before the listener is first awaited, which must be received.
- **Trace decision.** The two failures were read from the job logs and then from the code; neither needed a reproduction to locate.
- **Stop conditions.** Stop if a failed job is being re-run instead of its cause fixed.

## The two causes

**A record that vanished for an instant.** A server that starts beside a killed server's record removed that record and then created its own. `rrflow stop` decides by the record: no record means the stop finished. A requester that looked between the removal and the creation reported "stopped" and exited with success while the next server was about to serve. The record is now admitted as a managed file and replaced in one step by a rename, so a reader sees the earlier record or the new one. A first publication, with no record to replace, is unchanged.

**An interrupt nobody was listening for.** `rrflow serve` wrote its announcement, and `rrd-server` its address and readiness file, before the first wait on the interrupt, which is when the handler was installed. An interrupt sent the moment a supervisor read the announcement met the default action and killed the process, with no clean shutdown and its serving record still published. `rrd_server::Interrupt::listen` now installs the handler when called, both servers call it before they announce anything, and the serve loop takes that listener.

Neither cause came from the storage work of 2026-11-20. The test arrived with `rrflow stop` and has been exposed to both since.

## Evidence

Windows 11.

| Command | Result |
|---|---|
| `cargo test -p rrd-engine --lib serving`, with the earlier remove-then-create restored | `a_requester_waiting_on_a_stale_record_never_sees_the_project_unserved` fails: "a look found no serving record during replacement" |
| the same, with the repair | 19 passed |
| `cargo test -p rrflow-cli --test installed_lifecycle instances` | 4 passed |
| `a_registered_instance_is_listed_served_and_stopped_cleanly`, five runs | 5 passed |
| `cargo test -p rrd-server --locked` | 39 passed |
| `cargo clippy -p rrd-engine -p rrd-server -p rrflow-cli --all-targets --locked -- -D warnings` | clean |

## Not proven

- `tests/interrupt_listener.rs` is Unix-only and was not run here: the local Linux environment did not respond. The candidate gate runs it.
- The ARM64 failure is repaired by reasoning from the code. The next nightly matrix on native Linux ARM64 is its proof.
- Five passing runs do not prove a race absent; the two tests above check each cause directly.

## Roadmap and POA&M

No status change.

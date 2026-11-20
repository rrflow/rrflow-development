# The Parse step tests read time from a clock that moves with real time

**Status:** active change-journal evidence; the failing tests' cause is fixed; the same pattern in twelve other test files is recorded, not changed

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/parse-step-test-clock-20261120`

**Owner:** the alarm [Main is red: ci](https://github.com/rrflow/rrflow-development/issues/437); [POAM-091](../../../poam/rrflow-1.0-alpha/poam-091.md)

**Starting revision:** main `aeb76407b`.

## Change brief

- **Outcome and roadmap item.** OBJ-07 attunement; D-02. An open `Main is red` alarm is repaired before new work.
- **Current behaviour.** The push run on `aeb76407b` failed two tests in `engine::tests::attunement::parse_step` during their shared setup, with `attunement transition time must advance`. The same commit's candidate run had passed them.
- **Target behaviour.** The tests give the engine times that advance the way a real caller's clock does, whatever the machine's speed.
- **Scope.** `crates/authority/rrd-engine/src/engine/tests/attunement/parse_step.rs` and POAM-091.
- **Failure oracle.** The nine Parse step tests.
- **Trace decision.** Read from the job log and the code; not reproduced by slowing a machine.
- **Stop conditions.** Stop if the engine's rule would have to be weakened to make the tests pass. It is unchanged.

## Cause

The engine stamps an attunement transition with the time its caller gave plus the real time the step took (`elapsed_time` in `engine/attunement.rs`), and refuses a transition that is not later than the one before. That rule is right: a real caller's clock already includes the time that has passed.

The tests gave fixed times one second apart. A step that took longer than one second in real time was recorded after the next fixed time, and the next step was refused. Parse steps start a worker process; on the push run the whole suite took 397 seconds. The engine was correct and the tests' clock was not a clock.

`tick(offset)` now returns the base, the offset, and the real time since the first call in the process. Every time a test supplies therefore includes the time that has passed, as a host clock does, and the offsets keep the order the tests meant. One test in the file already read its time this way.

## Evidence

Windows 11, one run.

| Command | Result |
|---|---|
| `cargo test -p rrd-engine --lib --locked parse_step` | 9 passed, in 297 seconds |

That run's duration shows steps taking seconds each on this machine too.

## Not proven

- The failure was not reproduced on demand. The repair follows from the code: with the earlier fixed times, any setup step longer than one second fails exactly as the log shows.
- Twelve other files under `engine/tests/attunement/` give fixed times the same way, 172 uses. Their steps do not start a worker process and none has failed, but the assumption is the same. They are recorded as POAM-091 and not changed here, because some assert exact times and each needs reading first.

## Roadmap and POA&M

POAM-091 added, High, Open. No other status change.

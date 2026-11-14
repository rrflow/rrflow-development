# C-07g: bounded Windows characterization launch

**Status:** active launch evidence; characterization and qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-windows-characterization-launch`
**Owner:** OBJ-02 / C-07g / POAM-035 / PR151

## Scope and immutable candidate

This evidence-only package started from development main
`86adada6dab6411cc58cb9aaa931e35472c0f452`, tree
`42f1a022bf06f1a5c0593203f9a553e1c7d837e0`. Empty opening commit
`c4f2cd80257760be457bb8545787bb6defbd7e74` opened
[PR151](https://github.com/rrflow/rrflow-development/pull/151) before this
record. The separate tested [C-07 candidate](https://github.com/rrflow/rrflow-development/pull/142)
is `8721f17adc6fd97a05708feb1b3e332508365058`, tree
`7ba25be80d4023b41ea8dd4e912f97922ce5a129`; its source remains frozen. Its
fresh candidate CI and PR150 dependency remain pending.

The isolated Windows release build passed. Executable SHA-256 is
`eeeff4d579d9e382a3729fc89ecc0b0f1c0c6ddff143026ba2b9eb53acd00248`;
the binary reports `qualification: unqualified` and no build provenance.
The installed harness SHA-256 reported by the launcher is
`cdcc3699cbfc3d2f15eea5fdc040a14525276875f0f0eae02e78ecc3a49e6c89`.
The configuration file SHA-256 is
`fed73738ac4df13564f0cfc4c3f9b753b367aa9bd733823f157a4bb74951669f`.

## Cold-lane failure

The first campaign file SHA-256 was
`0523c29417c970775ae8d811f1125b31ac588679057ba9ae2e99ee9c93e871c0`;
its canonical campaign digest was
`8521a024c8123f74a5ea551b770ba9c8db60e05f80171dbe5b09a6eaf571f92a`.
The installed driver actually started but rejected its requested cold lane:
`native cold-cache control is not available on this platform`. Its result was
failed with zero segments, exit code 101, and cleanup confirmed. It did not
start corpus setup or soak. Evidence is retained under
`target/c07-windows-preflight-8721/`, including campaign, result, progress,
launcher output and cleanup receipt. This is a platform/fixture admission
failure, not proof of an engine durability failure.

## Distinct warm-only run

The second campaign intentionally omits the unsupported cold lane. Its file
SHA-256 is `7b5557ac5288946efdc54cb5d5f2522cc9299f92967ab0dfa5afd92a7143d602`
and canonical digest is
`0d475b7b0c2b16a085042eb6369b176139c19e54eb85c26b1536b13c0e0d98db`.
It is a `characterization`, not a qualifying replacement for the cold lane or
the frozen Desktop profile. The host has 64 GiB physical memory; the runner
reports binding a 2-GiB Windows Job process/job limit and a four-CPU quota on
the 24-logical-CPU host. The corpus specifies 100000 records, 500000 edges,
100000 vectors of dimension 384 and four clients; its setup deadline remains
two hours and its operation deadline 30 seconds. Its ten-minute soak field is
not the required 24-hour soak.

The actual launcher command is the repository's `scripts/ci/
run_sustained_capacity.py` with the separate campaign, configuration and new
evidence directory under `target/c07-windows-warm-only-8721/`. At a read-only
sample on 2026-10-27, the launcher process was alive and the progress file
contained 6848 corpus intents and 6784 acknowledgements. No terminal result,
complete independent readback or measured segment was present at that sample.
The run may later fail or complete; a later record must bind its actual
terminal artifacts and cleanup. Partial acknowledgements do not establish
correctness, sustained throughput or recovery.

## Verification and limits

Both campaign documents passed the existing canonical validation command.
The retained evidence and local process were read without modifying the
running estate. No engine code, deadline, data corpus or gate status changed.
Documentation, generated navigation/inventory and workflow checks passed
locally; candidate CI is still required. The real 8-GiB-host profile, full
native matrix, exact-candidate CI, complete readback and 24-hour soak remain
open under the existing owners.

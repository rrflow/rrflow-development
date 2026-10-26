# C-07g bounded-window native terminal outcome

**Status:** active retained failure evidence; C-07 remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-window-native-outcome`
**Owner:** OBJ-02 / C-07g / POAM-035 / PR139

The [frozen launch](c-07g-window-native-launch.md) at source
`04559542ada72d335cdf21c3b586eec46895f205` ended in failure. Starting repository
revision for this result-only correction is
`eb010a2961c38446d17876bf812d02368187e489`. The PR139 brief described this
correction before edits. Driver: current Codex session.

Observed service `rrflow-c07-desktop-integrated-04559542` is failed, exit 1;
installed harness exit 101. The retained launch reports elapsed 7202384 ms,
cleanup confirmed. All 100000 corpus operations were acknowledged; 64724 had
independent readback before `corpus setup exceeded its frozen deadline`.
No measured load or actual 24-hour soak began. The previous 38960-read run and
this 64724-read run are both incomplete setup attempts; these partial counts
are not accepted throughput or full-engine reliability results.

The final observed process RSS was 838348800 bytes under the 2 GiB engine limit;
physical file handles peaked 128 with zero handle denials, zero failed flushes
and zero failed compactions. Those samples do not establish full-duration
resource acceptance. The failure is incomplete verification, not demonstrated
acknowledged-data loss. No operation, setup, latency, error or resource limit
was relaxed and no replacement run was started by this result correction.

## Retained evidence

Guest base:
`/home/devcontainers/rrflow-c07-native-79663cdd/desktop-integrated-preflight-04559542/`.
All original data and evidence remain in place. A read-only hash capture is
retained locally at `target/c07-window-outcome-identities.json`.

| File | Bytes | SHA-256 |
|---|---:|---|
| `campaign.json` | 9423 | `a0e7fca8eb2c05e603e6aea25574c5ccba6a17883517d09442ade6bceeb97727` |
| `service-launch.json` | 1319 | `68e75ab32c1aeb626fe0fab458a0428011b354f350e791bea6044445afe0bdb4` |
| `evidence/progress.jsonl` | 55526256 | `7e8b92e0ee4e29f1677217f23b70ac46dc6f2c6df2ca77acabd1944203692948` |
| `evidence/result.json` | 427 | `7c8a1903aeef8e1d19b8054084ac88472d4d240effb4ef7b82067be6f0bd5c0b` |
| `evidence/launch.json` | 505 | `0af2d8515746e11550dd3edfcd767e50a732c48c6894f5d4532701fdcc308313` |
| `launcher.stderr.log` | 604 | `9ef5714f3d09e015e46011b6bec1fb914b25573872f97b2f9d1ee83b1b261e06` |

The campaign-file hash differs from the emitted semantic campaign hash
`14aa3e45b07cfce6fd4824e49a00d7913b00f65716403028c1ed1c344d8b6a1a`;
these identify different representations, not a substituted workload.

## Change scope and evidence discipline

Changed paths: this new immutable journal, C-07g's current evidence row,
POAM-035 and generated journal navigation/file inventory. Original journals
remain untouched. Research decision: terminal service/results/progress are
the primary evidence; no external algorithm is introduced. Trace/resource
decision: retain exact identities and limits, actual failure, final resource
sample and missing phases. Complete reads: retained result/launch/stderr and
hash capture; streaming progress inspection extracted counts and last events.
Targeted reads: C-07 owner, POAM-035 and previous launch/implementation records.
No claim of a full engine audit or source optimization follows from this run.

PR130/131 have merged through merge-group 9a441efd; PR134's trace correction
passed its engine checks, but its macOS disk-pressure fixture failed in
candidate 40266752462. The separate shared fixture remedy is PR140. Neither
merge status nor passing engine tests closes this native setup gap.

Acceptance for this correction: inspect exact terminal identities, verify
source/test bytes unchanged, regenerate/check documentation/navigation/inventory,
review full diff and preserve both failure histories. No Cargo rerun is needed
for result-only documentation. No native profile/24-hour, provider, UI or
project-indexing acceptance is claimed. Stop for changed history, altered
thresholds or any success inference from partial readback. C-07/POAM-035 remain
open; the roadmap current evidence is corrected, with no checkbox change.

Result-only validation passed: documentation (619 statuses/617 coordinates),
navigation (17 indexes/618 nodes/2441 edges), inventory (1747 records),
whitespace and exact unchanged source/test comparison. Full changed-document
diff was reviewed. No runtime acceptance command was rerun or claimed.

# C-07 corrected-cadence installed Windows characterization

**Status:** active completed characterization; operating point failed
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-corrected-cadence-4k-outcome`
**Owner:** OBJ-02 / C-07g / POAM-035 / [PR214](https://github.com/rrflow/rrflow-development/pull/214)

The [PR brief](https://github.com/rrflow/rrflow-development/pull/214) froze a
single short Windows NTFS characterization before launch. Its clean source was
`473261ea5f31011c750ee132b2631ce569a68ac1`, tree
`e23fe8b0e6bc21f8ab37c89d7ed40d0d85888744`, primary SHA-256
`5f942908ac8ad6e1f6af46db48bbbc3db9283a3e2b36ba5fcf7c4d594831c568`,
release harness `b993d2dfc4761e05d82c6c15c0af719774413baa89f468e596324cd0d02822e5`,
configuration file `fed73738ac4df13564f0cfc4c3f9b753b367aa9bd733823f157a4bb74951669f`,
campaign file `a7d5c549e2792ee5c69d6c687fd344089940e8ea2304a0ad7f543189bb3f3482`,
canonical campaign `7f8e1d29f884d1517ba62ed8047efa9a28b419c6a45c9a6332abf20cc6cf75ec`,
driver `61dfa9d3e4717e2b5b136a398d5d9ca2372d66e7c8d17a73b3da21cdab5423a1`,
workload `8228ca07b1d86685d4a3b4444b2b6451c3b8211f2d49f688795d77eaeca68816`,
and resource-window helper `fcf4746a8590973a56cc704dc971ddcd3097b48f875332a7611e50871d7c9619`.
The [preceding candidate](c-07-initial-sample-cadence-correction.md) failed
warmup on a sampling gap. No numerical threshold or workload was relaxed here.

The owned Windows Job enforced four CPU equivalents and 2 GiB, its harness
digest matched the frozen one, the launcher returned zero and cleanup was
confirmed. The independent command `cargo run --offline --locked -q -p
rrflow-eval -- storage campaign verify D:/localdev/devlabs/rrflow/target/c07-cadence-4k/evidence`
also returned zero for the retained characterization's consistency. This is
not a successful operating point: `qualification_passed=false`. Its measured
phase had 600 attempts, four rejected mixed writes, zero unresolved outcomes
and two over-budget latencies. Warmup, burst and the 60-second short soak had
zero recorded failures or latency breaches. Twenty-four segments, 12 reader
interference observations and a backup-under-write lane were retained. The
corpus independently verified 4,000 records, 20,000 edges and 4,000 vectors,
including after two forced reopens; final readback followed the short soak.

All four rejections have the fixed `api-conflict-storage-runtime-conflict`
classification and individually read back absent. SHA-256 of the synthetic
logical IDs matches the bounded trace digest: `load-9-337` and `load-9-338`
reported read/expected cursor 28837 and actual 28844; `load-10-356` and
`load-10-357` reported read/expected 28858 and actual 28865. The cursor values
show the returned conflict state; they do not establish which internal conflict
branch fired or which action advanced the cursor. The two measured latencies
over the 500 ms scheduled-to-finish bound were rejected mixed write 356 at
550.904 ms and acknowledged point read 361 at 513.639 ms. A different
post-finish verification exceeded 500 ms but is not the evaluator's latency
metric and is not counted as a third breach. No retry or exclusion occurred.

The PR brief initially said the two digest domains prevented correlation;
hashing the synthetic IDs established the match, and the brief was corrected
before this record. The host monitor recorded no Cargo/rustc process in its
sampled intervals; other shared-host work was not controlled or measured.
Local, ignored evidence under `target/c07-cadence-4k/` is not a remote upload:
result SHA-256 `3ea3853480075722e25ed55a3b5d950d4b53ab013a695dbd41f5ab2050280208`,
progress `8e05f8fe9a72886bb3553d03f314eef6f90b0a2019f66a73e18634c2ba7f7b18`,
launch `a82090c397461a2597965affe79f2366a2ced10517f7fe67686b7d08027e2061`,
host observations `7acc43aa4fd4c5f678621c7d9b3bc88c12ba95574eaaae3d598699527ecc7ebc4`,
operator receipt `6a7cf2082a68e19df567748cf05d6432fa821a114185d51d0528caeb58289029`,
stdout `83bb1955ba38ebcd0d4347c4a34ff62f6675e988e1e2f9eb440c4e6570af022f`,
stderr `9b96a3253e752d02224f6cc669b38024e4681d1f892054a84b1ed8eaea5659c2`.

Checklist: reviewed the campaign, frozen identities, launcher, prior cadence
journal, result, progress, segments 9/10, evaluator latency calculation and
bounded traces; documented the PR brief before launch and evidence edits;
validated source and hash bindings; ran the single workload and independent
verification; preserved failures and uncertainty; updated open POAM-035;
regenerated navigation and inventory. No runtime code, gate, workload,
threshold or roadmap status changed in this evidence follow-up. The cause of
the cursor advancements, causal repair, integrated candidate/protected CI,
100,000-record native matrix and actual 24-hour soak remain open.

The first documentation-policy check raced the navigation generator and
failed because the new index link was not yet present; it must be rerun after
generation. Rerunning `python scripts/knowledge/render_navigation.py --check`
and `python scripts/ci/check_documentation.py` sequentially passed. Two
subsequent patch attempts used mismatched context and made no changes; this
record retains those authoring failures.

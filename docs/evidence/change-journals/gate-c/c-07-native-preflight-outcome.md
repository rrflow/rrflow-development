# C-07 macOS native capacity preflight outcome

**Status:** active native characterization; candidate acceptance and failure diagnosis open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-native-preflight-outcome`
**Owner:** OBJ-02 / C-07 / POAM-035; [PR204](https://github.com/rrflow/rrflow-development/pull/204)

[Manual preflight 40933586936](https://github.com/rrflow/rrflow-development/actions/runs/40933586936) completed successfully on macOS at experimental head `5e8f6ed293158b071f9d4d922ebf70020ce080bf`, tree `40e41e459ea3b05b8983975ce396ab3f4f624532`. It ran from 18:44:05 to 18:56:57 UTC on 2026-10-29, including setup and compilation. The [launch record](c-07-native-preflight-launched.md) records its deliberate temporary workflow and byte-for-byte restoration at `18f8d440cbaed17471dd3ce50c6f3857af69760a`. The preflight is not the full candidate pipeline or the protected merge group.

The `native_capacity_suite` receipt reports `status: passed`, 24 cases: three fresh/existing pairs for each of `wal_before_write`, `segment_before_create`, `manifest_before_create` and `current_before_create`, all on APFS. The log contains 24 `native_capacity_receipt` lines and 24 detached-cleanup reports. The harness SHA-256 is `8b376fa18c6202f74c793f2411c68c366caa2065a01e5efd8297a515c0c89f67`; each receipt identifies primary executable SHA-256 `4ff08b6ee94ff0075c97890f4cbeb794325a4a7803ae4160f6716e243c25f834`. The build is unqualified debug instrumentation, not a signed release. GitHub reports no artifact: no native-create failure occurred, so the conditional failure diagnostic was not produced. The raw run log remains linked above; a local ignored copy is `target/c07-native-preflight-40933586936.log`.

`git diff 5e8f6ed293158b071f9d4d922ebf70020ce080bf 18f8d440cbaed17471dd3ce50c6f3857af69760a -- scripts/ci/run_native_capacity.py scripts/ci/test_native_capacity.py crates/adapters/rrflow-cli crates/persistence/rrd-lsm Cargo.lock` returned no diff; this checks the named harness inputs, not every build input. The first attempt to parse the downloaded log with an inline PowerShell/Python command failed from quoting; a separate ignored Python script parsed the suite JSON successfully, and direct log counts confirmed the 24 receipts and cleanup reports. No product state or authoritative evidence was altered by that parsing.

This real preflight demonstrates one successful run of the repeated APFS case set on this experimental identity. It does not identify the cause of PR194's earlier `hdiutil create: Resource busy`, test the new failure-artifact path on real macOS, establish the full native hardware capacity matrix or 24-hour soak, prove unchanged code will never fail again, or clear PR194's failed candidate. PR194 dependency integration and full candidate and protected merge-group checks remain required. No roadmap or POAM status change.

- [x] Bind the run to exact source/tree and inspect its real suite, receipt and cleanup counts.
- [x] Preserve the earlier dispatch rejection and verify the normal benchmark workflow was restored before PR readiness.
- [ ] Diagnose any actual native-create failure from retained evidence, validate the failure artifact on macOS, integrate dependencies and pass final candidate and merge-group checks.

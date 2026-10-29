# H-04 SDK harness native manifest privacy

**Status:** active implementation evidence; candidate CI and native qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-sdk-native-private-manifest`
**Owner:** H-04 real-process SDK conformance and POAM-011

## Brief and baseline

[PR177](https://github.com/rrflow/rrflow-development/pull/177) carried the brief before code and workflow edits. Starting public development main `9dea4eae5f904ba479a2df1898987b807e276172`; empty bootstrap `db842428946f04feb8dab139d011e778dc8308fd`. Driver: this Codex session. [PR176](https://github.com/rrflow/rrflow-development/pull/176) independently records the related discovery; neither PR changes gate status.

Before, a native Windows probe creating a temporary file and applying `os.chmod(path, 0o600)` reported mode `0o666`, while `scripts/ci/run_sdk_conformance.py::wait_for_manifest` unconditionally required mode `0600`. The Rust example used ordinary file creation on non-Unix and its Windows DACL had not been established by this harness. The initial full conformance command exits before starting the server because `mvn` is missing on this host.

After, the harness creates a random Windows directory with an explicit protected current-user and LocalSystem DACL at directory creation, verifies the actual owner and allowed inherited entries of the directory and manifest before reading, and rejects unsupported ACLs/reparse objects. The child manifest inherits that directory DACL. POSIX keeps owner-only `0700` workspace and `0600` regular file checks; unsupported platforms fail closed. This is a test-harness rendezvous boundary, not product authentication or a bridge. The same focused tests run in existing candidate Linux SDK and Windows/macOS portability jobs. Windows also resolves the fixed toolchain name before launch: the native probe found `shutil.which('pnpm')` returned `pnpm.CMD` but `subprocess.run(['pnpm', '--version'])` raised `FileNotFoundError`; the resolved `.CMD` path exited zero.

## Review and decisions

Read the H-04 roadmap, POAM-011, CI and change-authoring procedures, full `run_sdk_conformance.py`, `sdk_conformance_server.rs` manifest creation, relevant native secret-file boundary and CI jobs. Microsoft [file security](https://learn.microsoft.com/en-us/windows/win32/fileio/file-security-and-access-rights) documents security attributes at creation and inherited file ACLs; [security descriptors for new objects](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors-for-new-objects) documents protected DACLs. No upstream code adapted, no code moved, no product runtime modified. Runtime trace/resource measurement is not needed for this bounded test harness; real process startup was exercised. Creating a protected directory does not claim resistance to administrators or other principals that can rename/delete it through its parent.

## Commands and observations

- Native Windows `python -m unittest discover -s scripts/ci -p test_sdk_private_manifest.py -v`: three tests passed, POSIX test skipped. Real Windows DACL readback accepted the restricted directory and actual child file, and rejected a deliberately Everyone-accessible directory/file; a temporary `.cmd` tool launched by its resolved path. This checks descriptor structure; no separate-user access attempt was run.
- `wsl -d Ubuntu -- python3 -m unittest discover -s /mnt/d/localdev/devlabs/rrflow/target/sdk-native-private-manifest-worktree/scripts/ci -p test_sdk_private_manifest.py -v`: two POSIX tests passed; Windows tests skipped. This is Linux under WSL, not a capacity or clean-machine qualification.
- With `CARGO_TARGET_DIR=D:/localdev/devlabs/rrflow/target`, `cargo build -p rrd-client --example sdk_conformance_server --profile test --locked` passed. A Python probe launched that real Windows executable with the actual corpus and protected workspace; `wait_for_manifest` validated and read format 1 with six retry endpoint identities, then the server exited 0 after shutdown. This does not execute SDK clients.
- With the same real server and manifest, individually invoking the prescribed Rust, Go and .NET commands from `LANGUAGE_COMMANDS` passed the shared corpus on Windows (corpus SHA-256 `674911d36afcd3889019d7f031d874906e8f5e6da72ba40e9c003d70b08427c2`); each server exited 0. The Rust test passed one test, Go printed its corpus success, and the .NET runner passed five tests. These are three language results, not the six-language run.
- The Python command `uv --directory sdks/python run --frozen python tests/sdk_conformance.py` failed because no Python >=3.15 interpreter was available. The TypeScript command `pnpm --dir sdks/typescript install --frozen-lockfile` failed with `ERR_PNPM_BROKEN_LOCKFILE` under local pnpm 10.30.3 and warned Node 24.21.0 was below the declared >=24.22.0; the lockfile declares pnpm 12.14.0. This does not establish a malformed lockfile under the pinned toolchain. Java was not run because Java/Maven were absent. These commands were attempted separately rather than altering the declared corpus or counting absent languages as passes.
- `python scripts/ci/run_sdk_conformance.py` failed at `require_toolchains`: missing `mvn`; Java was also not found by `Get-Command`. No full six-SDK corpus is claimed.
- `python scripts/ci/check_workflow.py` passed; `python scripts/ci/test_candidate_admission.py` passed 11 tests. Navigation and documentation checks passed. The first inventory check after additional edits detected drift; it was regenerated and checked again before commit. Diff whitespace checks passed.

## Checklist and remaining work

- [x] Brief named owner, baseline, scope, smallest oracle, research, failure and stop conditions before edits.
- [x] Native Windows positive and permissive-descriptor tests; real Rust server manifest startup; Linux-under-WSL permission checks.
- [ ] Independent separate-principal effective-access denial, full six-language Windows corpus, clean Linux/Windows/ARM/macOS installed qualification and candidate CI not yet established. Do not describe this as complete SDK, bridge, C-07 or alpha evidence.
- [ ] PR merge requires all candidate checks and the repository merge controls.

Changed `scripts/ci/run_sdk_conformance.py`, `.github/workflows/ci-reusable.yml`, POAM-011 detail and generated navigation/inventory; created `scripts/ci/sdk_private_manifest.py`, `scripts/ci/test_sdk_private_manifest.py`, and this journal. No objective, roadmap or POAM lifecycle status changed.

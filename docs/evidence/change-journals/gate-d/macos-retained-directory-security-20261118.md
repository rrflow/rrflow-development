# Read a retained directory's security on macOS

**Status:** active implementation evidence; macOS proof is the qualification run named below
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/macos-retained-directory-security-20261118`
**Owner:** Gate D, [POAM-006](../../../poam/rrflow-1.0-alpha/poam-006.md)

## Change brief

- **Outcome and package:** installation into an existing project (Gate D, POAM-006).
- **Starting revision:** `cc5ebe0fd` on `main`.
- **Current behaviour:** on macOS, `RrdEngine::plan_installation` refuses a project that already has a `.rrflow` folder with `retained directory ACL inspection is not yet supported on this platform`. `directory_security_sha256` in `crates/authority/rrd-engine/src/engine/token_key/private_file/unix.rs` was built for Linux only; every other Unix returned `Unsupported`. The nightly full matrix of 2026-11-16 (run 43878604248) failed on it. Commit `a44cc6eab` then limited the two retained-install tests and the digest test to Windows and Linux and added a macOS test that asserts the refusal. POAM-006 records that as "an open deficiency, not a decision".
- **Target behaviour:** macOS reads the directory's owner, group, mode, file flags and extended ACL from the retained descriptor and binds them in the plan, as Linux and Windows do. The same three tests run on macOS. The refusal remains only for platforms with no read.
- **Scope:** `private_file/unix.rs` (the macOS read), the `cfg` attributes of three tests in `tests/installation/configuration_and_adoption.rs` and one in `token_key/private_file.rs`, POAM-006, and the new POAM-085.
- **Failure oracle:** on macOS, `a_leftover_rrflow_folder_is_retained_untouched_and_the_engram_installs_beside_it`, `a_changed_leftover_rrflow_folder_refuses_the_stale_plan_without_effects` and `directory_security_digest_is_stable_and_follows_native_permissions` fail before this change (they do not run; run by hand they panic on the refusal) and pass after it.
- **Research decision:** the `libc` crate (0.2.189) binds no `acl(3)` call for Apple targets, checked in its `src/unix/bsd/apple/mod.rs`. The four calls used (`acl_get_fd_np`, `acl_size`, `acl_copy_ext`, `acl_free`) are declared locally; libSystem provides them. `acl_copy_ext` is used rather than `acl_to_text` because the text form resolves principal names through the directory service.
- **Stop condition:** if the macOS qualification run fails any of the three tests, the change does not land.

## Files read

`private_file.rs`, `private_file/unix.rs`, `private_file/windows.rs` (the digest), `engram_layout.rs` (the two callers), `tests/installation/configuration_and_adoption.rs`, `rrd-lsm/src/filesystem.rs` (`pin_directory_checked`) and `filesystem/native.rs` (how macOS handles are opened), POAM-006, and the log of run 43878604248.

## What changed

- macOS digest: `("rrflow-macos-directory-security-v1", uid, gid, mode, st_flags, extended ACL bytes or none)`, hashed with SHA-256. The ACL is read from the descriptor with `acl_get_fd_np(fd, ACL_TYPE_EXTENDED)`; a null result with `ENOENT` means the directory has no ACL. The ACL is bounded to 64 KiB and freed on every path.
- The two retained-install tests, their fixture and the digest test run on Windows, Linux and macOS.
- The refusal test now compiles only where no read exists (neither Windows, Linux nor macOS).

## Commands and results

- `cargo fmt --all`: no changes beyond the edit.
- The macOS function compiled alone against `aarch64-apple-darwin` with warnings denied (`cargo check --target aarch64-apple-darwin` on a crate holding only that function and `libc`): finished without a diagnostic. This checks types and the foreign declarations, not behaviour.
- macOS behaviour: see the qualification run below.

## Not run locally

Nothing here runs macOS. The whole engine was not compiled for the Apple target locally, because its C dependencies need Apple's toolchain.

## Found while reading, not fixed here

Parse, the code-indexing phase, cannot run on macOS: its worker supervisor exists for Windows and Linux only (`attunement/evidence.rs`, `attunement/parse/worker.rs`). Recorded as [POAM-085](../../../poam/rrflow-1.0-alpha/poam-085.md).

## Qualification run

Pending: a manual `ci` dispatch on this branch, which runs the full matrix including macOS.

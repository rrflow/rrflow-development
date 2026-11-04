# Lab definitions as signed, patchable packs

**Status:** active immutable change-journal evidence; Gate J and OBJ-10 remain open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-j/lab-definition-packs`
**Owner:** evidence for shipping the comparison lab's definitions apart from the app

## Evidence record

```text
gate/package and alpha outcome or prerequisite advanced: OBJ-10 reproducible competitive evidence, OBJ-11's versioned extension model, Gate J's evaluation boundary. Owner direction of 2026-11-03: tests "bundled and built as a way that allows us to send and patch like a virus definition".
starting revision/tree and pull request: 92f18c23, stacked on #268 and #260; pull request #272.
change brief (current -> target behavior, owner, exact scope, unchanged behavior, stop conditions): the description of pull request #272. Definitions were loose files in a checkout, with nothing proving who wrote them and nothing stopping an altered or older copy. Now they travel as one signed text file that can be patched, and a store installs it atomically. The lab itself is unchanged and still reads eval/lab; running it from an installed pack is a later package.
files and symbols read in full: lab/catalog.rs, lab/sources.rs, lab/mod.rs, bin/rrflow_lab.rs, tests/lab_catalog.rs, eval/lab/README.md, eval/lab/systems/*.json, eval/lab/tests/*.json, docs/roadmap/rrflow-1.0-execution/change-authoring.md.
implementation/evidence paths created, changed, moved, or deleted: created crates/evaluation/rrflow-eval/src/lab/definitions/mod.rs, pack.rs and content.rs, tests/lab_definitions.rs and this journal. Changed src/lab/mod.rs, src/bin/rrflow_lab.rs, crates/evaluation/rrflow-eval/Cargo.toml, Cargo.lock, eval/lab/README.md and eval/lab/sources.json. Generated the gate-j index.
research decision and primary-source/adaptation record: read ClamAV's cvd.c, cdiff.rs and libfreshclam (release clamav-1.5.4, verified with the GitHub API) and The Update Framework specification (v1.0.37, verified likewise). Taken as ideas, with no code: a versioned signed container, a patch with a full-pack fallback, a test load before replacing, rollback refusal and expiry. Left out on purpose: delegations, thresholds, an online timestamp, mirrors, ClamAV's line-patch commands, and X.509. ed25519-dalek 3.0.0 (BSD-3-Clause, verified with cargo info) adds six crates to the lockfile, and verification uses verify_strict. No precedent for signed evaluation definitions was found, so that design is mine.
trace/resource/debug decision (add/preserve/not applicable, with reason): bounded by MAX_PACK_BYTES (16 MiB), MAX_PACK_FILES (4,096) and MAX_FILE_BYTES (4 MiB). Install reports added, changed and removed files and every changed launch. The store keeps an ordered history of installs and reverts.
first-failure or characterization oracle and result: 1ae17480 committed 22 tests against a stub, and all 22 failed. Each rule in the brief has at least one.
acceptance commands and exact results: cargo test --locked -p rrflow-eval --test lab_definitions, 23 passed, including the real rrflow-lab binary end to end. cargo test --locked -p rrflow-eval --no-fail-fast, 142 passed. cargo clippy --locked -p rrflow-eval --all-targets with -D warnings, clean. cargo fmt --all --check, clean. cargo test --locked -p rrd-engine --test workspace_architecture, 33 passed. rrflow-lab sources --file eval/lab/sources.json, 35 sources valid. render_navigation.py and check_documentation.py passed. A manual round trip with a throwaway key on this repository's definitions: the full pack carried 18 of 18 files in 53,899 bytes; after one test changed, the patch carried 1 of 18 in 5,920 bytes; a pack whose manifest was edited was refused with "the pack signature does not check"; the patch installed and reported tests/vector-small.json changed; the older pack was refused; status reported sequence 2 of 18 files, not expired.
surfaced failures and corrections: ed25519-dalek 3.0.0 has no std feature, which cargo reported, so its default features are used. A test first gave a system the wrong file name, which the content check correctly refused, so it now sets the new system's id. A README edit matched a heading that is not unique and failed its assertion before writing, so the section was inserted by position.
checks not run and reason: candidate CI, because it skips drafts. The rest of the workspace's suites, because only rrflow-eval changed. No production signing key exists, so no pack has been signed with one, and the owner decides where that key lives. Nothing was tested on Linux or macOS in this package, including the private key's 0600 mode, which only applies on Unix.
remaining known errors and owner: one signing key and no threshold, which The Update Framework calls insecure and which is a deliberate limit for one publisher. Replacing a key means shipping new trusted keys; recovery-signed rotation is not built. The private key file has no special protection on Windows beyond the folder it is written to. The trusted keys come from a file named on the command line and are not yet pinned in a binary, so whoever can change that file can change who is trusted. A pack can change which container starts, and install names that change, but no approval step exists yet, and nothing runs from a pack yet either. Those are the next package, with the toggles and reference results.
full-file reread and diff review: complete for pack.rs, content.rs, the command-line changes, the tests and this record.
change checklist: complete. Brief before implementation; failing tests committed first; no engine change, so the engine still does not grade itself; no runtime fetch; no provider state; the envelope holds no RRFlow types, and a test uses it with another product.
roadmap/POA&M status change, or explicit none: none.
pull request, CI run, and merge evidence: recorded on pull request #272.
```

## Links

- [Pull request #272](https://github.com/rrflow/rrflow-development/pull/272)
- [Comparison lab](../../../../eval/lab/README.md#definition-packs)
- [Gate J](../../../roadmap/rrflow-1.0/gate-j.md)

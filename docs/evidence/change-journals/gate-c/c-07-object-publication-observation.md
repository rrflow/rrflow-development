# Windows immutable-object publication collision observation

**Status:** active observed native hazard; owner regression and repair pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-object-publication-observation`
**Owner:** C-03 / C-07 / POAM-035; driver: active SWAG session

## Recorded failure

The controller/Inventory integration's all-feature engine suite failed
`concurrent_builds_with_one_key_publish_one_generation` with
`Vector("object store: Access is denied. (os error 5)")`. Its resolved source
tree was `e32836491a42c377a878b91978f97a9aca341d1f`; published draft integration
is `c5bd4d99fa84a107ce7c46171f335107977d0d2d` in PR206. The full run retains
289 library passes, one failure, four ignored helpers and 63 integration passes.
A focused run passed; it does not explain or clear the failure.

This separate worktree starts at exact public main
`f256c8e1337d85fa01f2a0c61e895a4d213163fa`. Its immutable-object implementation
matches the failed integration's implementation. The opening commit records
actual failure/probe evidence only, so the draft PR can carry the bounded brief
before regression or production edits; it contains no implementation or claimed fix.

## Native characterization and limits

`LocalObjectStore::publish_stage` observes target existence before calling
`publish_object_path`, which uses replacing `AnchoredPath::rename_to`.
A concurrent publisher can create the target between observation and rename.
The existing anchored `rename_new_to` provides non-replacing publication.

A separate Windows native probe creates two equal controlled files and holds the
winner without delete sharing. `MoveFileExW` with replacement/write-through
fails with error 5. The same operation without replacement fails with error 183
(already exists), preserving both files and the winner's SHA-256
`0c57191dbecced93e9730a558be512c1497c050eb96f5ee591392641fb331120`.
This proves a primitive hazard; it does not identify the original engine error's
exact operation. Probe bytes are correctness fixtures, never training data.

The [Windows API reference](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)
defines replacement and access semantics; the
[Rust rename reference](https://doc.rust-lang.org/std/fs/fn.rename.html)
also distinguishes replacement from preserving an existing target. No upstream
implementation is imported. Review covered the object put/stream/verification,
quarantine/publication paths, anchored rename variants, native Windows flags,
engine concurrency oracle, README/AGENTS, Gate C and POAM-035.

## Next owner proof

Require a deterministic late-publisher case to preserve a verified reader-held
winner, reject a different/corrupt target without deleting the valid stage,
and preserve actual I/O errors. Use the existing anchored non-replacing operation
and verify the winning digest before accepting collision reuse. Preserve parent
directory synchronization and existing crash instrumentation; no blind retry,
global serialization, unanchored path operation or relaxed sharing is allowed.

Local raw evidence is under
`D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/`:
`controller-inventory-all-features.log`, `controller-inventory-validation.json`,
`controller-inventory-quantization-repro.log`, `reproduce-object-publication.py`,
`object-publication-native-probe.json` and its retained disposable directory.
The full-suite log hash is
`b4ad363794155d2f63d521eb55bac186a570834d1afc38fd886e6e63b17ac609`.
The original controller worktree and histories remain intact. This opening
record changes documentation/discovery only. Component, joined engine,
candidate/protected and cross-platform qualification remain pending; no gate closes.

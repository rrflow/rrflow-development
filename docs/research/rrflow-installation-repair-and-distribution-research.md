# RRFlow installation, repair, and distribution research

**Status:** active supporting research; not an implementation or release claim
**Coordinate:** `rrflow://rrflow-instance/data/research/installation-repair-distribution`
**Owner:** primary-source evidence for the RRFlow installed-lifecycle target
**Reviewed:** 2026-09-10

This record answers one bounded question: what lifecycle must RRFlow expose so
an acquired release artifact can create, operate, verify, recover, repair, and
remove one project-bound estate without a source checkout or a second engine
authority? The accepted RRFlow target is defined by the
[installed lifecycle](../reference/deployment/installed-lifecycle.md); delivery
order and completion remain owned by the
[roadmap](../roadmap/rrflow-1.0.md).

## Method

The review combined:

1. complete reads of the current CLI command parser, development supervisor,
   server entry point, security bootstrap, installed-instance binding, backup
   and recovery controllers, engine backup operations, rrflowKV open path,
   rrflowKV manifest/WAL recovery, and current release plans;
2. execution of the current release binaries and their help/startup paths; and
3. primary documentation for single-binary database distribution, snapshot
   restore, integrity checking, corruption salvage, last-resort WAL reset,
   native Windows Rust targets, artifact provenance, checksums, and Windows
   executable signing.

Source behavior is evidence, not RRFlow architecture. No upstream command,
file layout, implementation tree, or compatibility surface is adopted
automatically.

## Current checkout findings

The checkout contains useful engine components but no truthful installed
product lifecycle:

- a release build emits separate `rrflow`, `rrd-server`, and `rrflow-mcp`
  executables, while the public `rrflow` command has no `install`, `serve`,
  `verify`, `repair`, `restore`, or `version` operation;
- `rrflow dev up` builds and launches checkout-local companion binaries,
  creates private JSON and marker-file state, and invokes a standalone security
  initializer;
- `rrflow dev doctor` can report readiness from source-tree and supervisor
  checks even when the public binary cannot initialize or serve the project;
- `rrd-server` instructs an operator to run `rrflow init`, but that command does
  not exist;
- `RrflowKvStore::open` creates an absent database and reconciles checkpoints,
  so it cannot be used as a read-only verifier or as proof that cold start is
  installation-only;
- rrflowKV already rejects a torn WAL tail and exposes a narrowly safe
  `repair_torn_tail` primitive; complete-frame corruption fails closed;
- authenticated logical/application backup catalogues and restore-to-new-root
  behavior exist, but standalone controllers can still accept arbitrary roots
  outside the installed engine resolution path; and
- the repository has platform characterization, but no natively built and run
  Windows `rrflow.exe` release candidate.

These are implementation facts, not a verdict on the intended architecture.
They establish the concrete convergence work.

## Primary-source findings and RRFlow adaptations

### One public executable

SurrealDB distributes its server and CLI as one executable and uses one command
tree to start and operate the database. This demonstrates that a serious
database need not expose its internal process/package topology as the default
operator surface. Sources:
[SurrealDB Windows installation](https://surrealdb.com/docs/running/installation/windows),
[SurrealDB installation](https://surrealdb.com/surrealdb/install), and
[`surreal start`](https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/start).

RRFlow adaptation: the default distribution exposes one primary executable,
`rrflow` on Unix-like systems and `rrflow.exe` on Windows. `serve`, MCP
presentation, verification, backup, restore, and project installation are
subcommands backed by libraries. Internal crates remain independent testable
boundaries; they are not separate prerequisites or lifecycle authorities.

### Verify is not repair

SQLite separates read-only integrity inspection from best-effort recovery.
`PRAGMA integrity_check` checks low-level consistency, while `.recover`
extracts whatever uncorrupted content remains and may produce a `lost_and_found`
table rather than silently declaring the result canonical. Sources:
[SQLite integrity check](https://www.sqlite.org/pragma.html#pragma_integrity_check)
and [SQLite recovery](https://www.sqlite.org/cli.html#recover_data_from_a_corrupted_database).

RRFlow adaptation: `rrflow verify` never changes bytes. `rrflow salvage` may
emit a non-authoritative export with omissions and uncertainty recorded, but
cannot publish it as the installed estate. Only a separately authorized
restore/import into an absent target may promote verified recovered state.

### Repair must be bounded and explicit

PostgreSQL documents `pg_resetwal` as an offline last resort, provides a dry
run, warns that forced reconstruction may leave inconsistent data, and advises
immediate dump/reinitialize/restore. It refuses to operate while the server is
running. Source:
[PostgreSQL `pg_resetwal`](https://www.postgresql.org/docs/current/app-pgresetwal.html).

RRFlow adaptation: RRFlow will not invent sequence, manifest, transaction, or
authorization truth. A repair plan is read-only and content-addressed. Applying
it requires the same plan digest, an exclusive offline lease, an authenticated
snapshot, exact preconditions, and an absent reflink/clone or fully accounted
same-filesystem candidate copy. Only the candidate is changed; full
verification precedes one durable atomic locator replacement and the prior
root remains retained or quarantined. Corrupt acknowledged canonical state
fails closed and routes to restore or non-authoritative salvage; there is no
RRFlow equivalent of a general forced WAL reset.

### Restore targets and conflict policy must be explicit

Qdrant snapshots include data and built indexes, restrict compatible versions,
and distinguish recovery priorities. Its startup restore requires an absent
target unless an explicit force option is used. Sources:
[Qdrant snapshots](https://qdrant.tech/documentation/operations/snapshots/)
and [Qdrant migration and recovery options](https://qdrant.tech/documentation/migration-recovery-options/).

RRFlow adaptation: restore creates an absent target or an explicitly named
quarantine replacement; it never overlays the live estate. Canonical semantic
state and authenticated backup inventory are restored first. Rebuildable
graph, BM25, vector, and analytical projections are accepted only when their
source cursor, schema, algorithm, and artifact digests match; otherwise they
are rebuilt through `RrdEngine` before readiness.

### Native platform proof is part of the product proof

Rust classifies supported Windows MSVC targets separately and documents their
native runtime/toolchain requirements. Cross-compilation or a Linux build does
not prove a Windows executable starts and persists data. Source:
[Rust Windows MSVC target support](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html).

RRFlow adaptation: the release matrix builds and runs the exact candidate on a
native runner for each supported target. Windows evidence names
`rrflow.exe`, verifies its archive and executable digest, runs install through
close/reopen/verify, and records required runtime dependencies. Linux-only
ELF output is never reported as Windows progress.

### Distribution integrity needs more than a checksum file

GitHub artifact attestations bind artifacts to build provenance and can bind an
SBOM; cargo-dist can generate release archives and checksums. Microsoft
SignTool verifies and Authenticode-signs Windows binaries, with SHA-256 and
timestamping recommended. Sources:
[GitHub artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations),
[GitHub build provenance and SBOM attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations),
[cargo-dist configuration](https://axodotdev.github.io/cargo-dist/book/reference/config.html),
and [Microsoft SignTool](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool).

RRFlow adaptation: an archive checksum detects transport corruption, the RRFlow
distribution manifest accounts for every shipped byte and embedded capability,
the SBOM accounts for dependencies, provenance binds build inputs to outputs,
and native platform signatures provide the platform trust signal. No one of
these substitutes for runtime qualification.

## Accepted conclusions

1. Binary acquisition and per-project installation are distinct. Acquisition
   yields a verified offline-complete bundle; `rrflow install` creates one
   project estate from only that bundle.
2. The alpha's default operator surface is one executable. Companion
   first-party processes are not default readiness prerequisites.
3. Normal open never creates an estate. Only an exact `install apply` action
   may execute cold start.
4. Plan generation is pure. Every mutating install, repair, restore, and
   uninstall operation consumes a reviewed content digest and records an
   engine-owned receipt.
5. Startup recovery, verification, repair, restore, and salvage are separate:
   startup replays valid acknowledged state; verify is offline and read-only;
   repair makes bounded changes only in an absent candidate before atomic
   publication; restore publishes a verified new target; salvage emits
   explicitly non-authoritative data.
6. Derived index repair rebuilds from canonical committed semantic state and
   source cursors. Physical compaction never decides semantic deletion.
7. `READY` means an authenticated operation passed through the installed
   `RrdEngine` and the required persistence/index/query health checks passed.
   A port, PID, marker file, source tree, or successful compile is insufficient.
8. Linux, Windows, and macOS artifacts are separately built, started, and
   exercised. The Windows deliverable is specifically `rrflow.exe`.

## Rejected shortcuts

- retaining `rrflow dev doctor` as release readiness;
- using create-on-open as implicit installation or repair;
- repairing complete-frame corruption by truncating or guessing counters;
- restoring over the live instance by default;
- declaring derived indexes canonical when their source coordinates differ;
- downloading templates, models, runtimes, or helper binaries during default
  installation;
- requiring SurrealDB, Qdrant, PostgreSQL, Turso, Dragonfly, Connectome, a mesh,
  or a provider for default readiness; and
- calling a compiled Linux binary proof of an installable cross-platform
  product.

## Research limits

This review does not select a release automation product, certificate vendor,
Windows installer format, service manager, or package manager. Those are
implementation choices to measure in Gate J. It also does not claim the current
rrflowKV physical verifier, semantic verifier, install coordinator, repair
planner, or single-executable composition already exists.

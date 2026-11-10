# Local cross-checkout compiler-cache characterization

**Status:** active local diagnostic evidence; cache-policy decision pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/local-compiler-cache-probe-20261104`
**Owner:** H-05/J-04 developer diagnostics; PR249 preflight resource decision

This is a local experiment accompanying the
[LFG/worker composition](lfg-selected-worker-composition-20261104.md); it is
not a CI pass or runtime benchmark. Two clean worktrees of exact source
`0fc8c9759032222fcf9d61777eb493095ab33012` used Rust/Cargo 1.99.1 and
`sccache 0.18.0` on the same Windows x86-64 host. The experiment launched
a separate loopback cache server with an empty local cache/configuration,
separate target directories, both checkout/target base directories, two
Cargo jobs and `CARGO_INCREMENTAL=0`. It did not reset or change the existing
shared sccache server, system settings or CI. Its own server was stopped.

Each ran `cargo build --offline --locked -j 2 -p rrd-contract --bin
rrd-contract-export`. Cargo exited successfully for both. The first took
65.109 seconds and the second 46.403 seconds; in the second build sccache
recorded 21 Rust hits, six Rust misses and 14 non-cacheable requests.
The exported OpenAPI documents had the same SHA-256 and 79 paths. The
[sanitized receipt](local-compiler-cache-probe-20261104.json) retains the
counts; raw logs and isolated cache remain under ignored `target/`.

These are one cold/warm component pair, with other host load and operating
system caches uncontrolled. They do not establish whole-CI speedup, safe
cross-platform reuse, release provenance or runtime behavior. The cache
counted one failed compiler invocation in each build despite successful
Cargo exits; its cause remains unclassified and needs investigation before
a policy decision. Equal OpenAPI output also does not verify debug paths or
all embedded paths. [sccache's documentation](https://github.com/mozilla/sccache/blob/main/README.md#normalizing-paths-with-sccache_basedirs)
warns that cross-checkout hits can preserve original debug and `env!` paths;
RRFlow test code uses `CARGO_MANIFEST_DIR`. Any wider adoption therefore
requires path-sensitive and build-provenance checks, representative repeated
trials, and review of memory, storage and cache trust.

Only this evidence record, its receipt and navigation index are changed; no
compiler-cache configuration is adopted and no product, roadmap or POAM
status changes. Required candidate and merge-group CI remain independent.

# Compiler-cache failure-counter diagnosis

**Status:** active local diagnostic evidence; cache-policy decision pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/local-compiler-cache-probe-diagnosis-20261104`
**Owner:** H-05/J-04 developer diagnostics; PR249 preflight resource decision

This follows the [isolated cross-checkout trial](local-compiler-cache-probe-20261104.md),
which recorded one compiler failure in each otherwise successful Cargo build
without retaining detailed compiler traces. To investigate, a separate empty
local sccache server and target directory built the same source
`0fc8c9759032222fcf9d61777eb493095ab33012` with the same locked contract
exporter command and two Cargo jobs. Cargo exited zero; sccache recorded one
failed compiler invocation. The debug trace identified the invocation as
the locked `proc-macro2 1.0.107` `proc_macro_span` unstable-feature build
probe: stable Rust returned E0554. The dependency's `build.rs` calls this
probe to test availability and converts its exit status to a boolean, so
the observed probe failure in this rerun is handled by the dependency's
normal fallback. A [sanitized receipt](local-compiler-cache-probe-diagnosis-20261104.json)
retains the classification; raw debug and Cargo logs remain under ignored
`target/`. The isolated server was stopped and the global cache was not reset.

The original two builds lacked debug traces; this reproduction supports but
does not independently prove that their counters were from the same probe.
No global or CI cache settings changed, and neither this diagnosis nor the
one-pair timing establishes whole-CI speed or path/provenance safety. No
product, roadmap or POAM status changes; required protected CI remains open.

# C-07 diagnostic candidate architecture correction

**Status:** active correction evidence; candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07g-diagnostic-boundary-correction`
**Owner:** OBJ-02, C-07g and the existing architecture guard

The [prior attribution record](c-07g-diagnostic-lock-attribution.md) describes candidate `d8023d56fc59383ebc3cc1ad925b025772eb13c8`; it is not rewritten. [Candidate run 40737647596](https://github.com/rrflow/rrflow-development/actions/runs/40737647596/job/122990414798) failed `cargo test -p rrd-engine --test workspace_architecture --locked`: `outward_product_sources_do_not_import_physical_components` identified `crates/transport/rrd-server/src/http/handlers/diagnostic.rs`. The first candidate placed an `rrd-store` import in a `#[cfg(test)]` module inside outward product source. Local package tests passed but had not exercised that workspace guard. This was a real candidate failure, not a runner skip or product completion.

[PR178](https://github.com/rrflow/rrflow-development/pull/178) was returned to draft and its correction scope was recorded before this change. Removed the misplaced unit module and kept `crates/transport/rrd-server/tests/http_process.rs`'s actual authenticated HTTP-to-rrflowKV trace/redaction test. The architecture rule was not weakened, the product trace was not removed, and no physical implementation moved. This test remains a process-local HTTP integration test; it is not installed capacity evidence.

In the isolated Windows target described in the earlier journal, `cargo test -p rrd-engine --test workspace_architecture outward_product_sources_do_not_import_physical_components --locked -- --nocapture` passed after removal; the full `cargo test -p rrd-engine --test workspace_architecture --locked` then passed all 31 tests. `cargo test -p rrd-server --all-targets --locked` passed, including all 19 HTTP tests and four controller-driver tests with the already built CI-required controller fixture. The first `cargo fmt --all -- --check` found an extra trailing blank line after removing the unit module; `cargo fmt --all` corrected it. Navigation, documentation, inventory, format and diff checks run before this correction commit. Other jobs on the failed candidate do not make its required gate pass; the exact corrected head needs a fresh full candidate run and merge queue. C-07, sample correlation and native/soak qualification remain open.

- [x] Failure and scope recorded before edits; retained unchanged guard and actual cross-boundary test.
- [x] Narrow architecture guard and server package suite passed.
- [ ] Full corrected-head candidate CI, integration with the sampler, loaded causal evidence and C-07 qualification remain pending.

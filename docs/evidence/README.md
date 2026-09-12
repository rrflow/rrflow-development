# RRFlow evidence index

**Status:** active evidence index; test plans are not execution evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence-index/rrflow-evidence`
**Owner:** reproducible proof and acceptance-test-plan discovery; linked from `docs/README.md`

Evidence is generated or measured output bound to a source revision, inputs,
environment, command, and owning roadmap gate. A design note, passing compile,
unchecked scenario, or copied benchmark claim is not evidence.

| Subject | Durable warp | Checkout record | State |
|---|---|---|---|
| Acceptance test plans | [`rrflow://rrflow-instance/data/evidence-index/test-plans`](rrflow://rrflow-instance/data/evidence-index/test-plans) | [`test-plans/`](test-plans/) | scenario definitions only; individual cells become evidence only through their owning gate |
| C-06i rrflowKV persisted-filter integration | n/a | [`c06i-rrflowkv-persisted-filter-linux-x86_64.json`](c06i-rrflowkv-persisted-filter-linux-x86_64.json) | clean-revision segment-v5 filter bytes, canonicality, false-negative/false-positive, metadata-only reopen, and miss-path page-I/O evidence; not C-06, release, or competitor proof |
| C-06i rrflowKV physical-policy candidate screen | n/a | [`c06i-rrflowkv-physical-policy-linux-x86_64.json`](c06i-rrflowkv-physical-policy-linux-x86_64.json) | clean revision `f7257fa` fixed-machine candidate selection; not an integrated production policy, release proof, or competitor claim |
| Historical mixed-storage soak | n/a | [`m4-storage-mixed-soak.json`](m4-storage-mixed-soak.json) | retained raw artifact; not RRFlow 1.0 qualification |
| Historical vector measurement | n/a | [`m5-vector-local-10000x128.json`](m5-vector-local-10000x128.json) | retained raw artifact; not RRFlow 1.0 qualification |
| Historical edge measurement | n/a | [`m6-edge-local-10000x128.json`](m6-edge-local-10000x128.json) | retained raw artifact; not RRFlow 1.0 qualification |
| Historical quantization measurement | n/a | [`g04-w04-quantization-local-512x64.json`](g04-w04-quantization-local-512x64.json) | retained raw artifact; not RRFlow 1.0 qualification |

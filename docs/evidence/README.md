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
| Historical mixed-storage soak | n/a | [`m4-storage-mixed-soak.json`](m4-storage-mixed-soak.json) | retained raw artifact; not RRFlow 1.0 qualification |
| Historical vector measurement | n/a | [`m5-vector-local-10000x128.json`](m5-vector-local-10000x128.json) | retained raw artifact; not RRFlow 1.0 qualification |
| Historical edge measurement | n/a | [`m6-edge-local-10000x128.json`](m6-edge-local-10000x128.json) | retained raw artifact; not RRFlow 1.0 qualification |
| Historical quantization measurement | n/a | [`g04-w04-quantization-local-512x64.json`](g04-w04-quantization-local-512x64.json) | retained raw artifact; not RRFlow 1.0 qualification |

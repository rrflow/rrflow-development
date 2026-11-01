# Correction: build-output probe executable

**Status:** active evidence correction; verified gap remains open
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-d/d-03-build-output-probe-correction`
**Owner:** OBJ-14 / D-01 / D-03 / POAM-038 / [PR208](https://github.com/rrflow/rrflow-development/pull/208)

The [build-output precondition record](d-03-build-output-precondition-gap.md) calls the tested `rrflow.exe` an ?installed? binary. That is imprecise: the probe ran the staged release executable directly from `target/c07-semantic-4k-fa945bc7/rrflow.exe` and attempted installation in a disposable project. It did not first install the executable into that project. The recorded binary SHA-256, exact source, synthetic Git ignore, changed precondition, nonzero apply result and `ProjectBindingMismatch` remain the actual observations. This correction does not establish a successful installation, a fix, D-03 or alpha acceptance.

PR208's correction brief preceded this evidence-only record. The earlier immutable record is preserved. Generated journal navigation is refreshed and checked with documentation and execution inventory; no product source or roadmap status changes.

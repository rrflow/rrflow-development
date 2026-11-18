# Route the rrd-store optional feature through the nightly matrix

**Status:** package journal, 2026-11-18
**Owner:** repository

## Change brief

- **Current behaviour:** the nightly full matrix fails in `pipeline / verify` with `CI policy violation: optional-feature coverage mismatch: expected=[... rrd-store ...] routed=[rrd-cluster, rrd-engine, rrd-inference, rrd-lsm, rrd-vector]`. Issue 406 recorded it. The cause is PR 405, which gave `rrd-store` the optional `fjall-baseline` feature without adding `rrd-store` to the optional-feature matrix in `.github/workflows/ci-reusable.yml`.
- **Target:** every package with an optional feature runs `cargo test --all-features` in the matrix, so the Fjall comparison test runs in CI.
- **Scope:** one matrix entry, `rrd-store`, in the optional-feature job.
- **Failure oracle:** `python scripts/ci/check_workflow.py` fails without the entry and passes with it.

## Result

`python scripts/ci/check_workflow.py`: `CI policy OK ... 7 optional-feature packages`.

## Not run

The optional-feature job itself, which runs on the pull request's candidate CI.

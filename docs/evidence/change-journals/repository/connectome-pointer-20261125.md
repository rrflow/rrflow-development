# The Connectome pointer moves to its merged main

**Status:** active landing evidence
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/connectome-pointer-20261125`
**Owner:** [Connectome client reference](../../../reference/client/connectome.md), repository checkout.

## Change brief

Baseline `3564b7054`. The `connectome/` gitlink pinned `a54f2f92`, and this checkout had carried an uncommitted pointer to an unmerged Connectome branch since 2026-11-23. Connectome pull request 121 merged on 2026-11-25 as `c39d8888f63ce6c0a39a6becc8a90d80273e788d`; that commit pins the engine to `3564b7054`, which contains the prompt flight readback change of pull request 502. Target: the gitlink names `c39d8888`. Scope: the gitlink and this journal. Stop if the pinned commit is not on Connectome's `main`.

## Verification

- `c39d8888` is the head of `rrflow/connectome-development` `main`.
- At the commit it merges, on Windows: `pnpm check`, `pnpm build`, 38 smoke tests, `cargo clippy` and 88 unit tests passed; the 34 real-engine tests passed in four of five full runs. One run failed `an_instance_started_elsewhere_is_found_attached_and_left_running`; its message was not captured and the cause is not known.
- Connectome's push run on `c39d8888` (45672191968) ended in failure: Linux, Windows, the frontend and the SDK integration jobs passed, and the macOS job was cancelled because no hosted runner accepted it. Connectome issue 122 is open for it. Corrected later on 2026-11-25: a macOS result does exist for this commit. The merge-queue run 45668600633 ran on the same commit `c39d8888` twenty minutes earlier and all six jobs passed, macOS included; only the repeat run after the merge was left without a runner.
- `cargo test -p rrd-engine --locked --test workspace_architecture`: 34 passed.

No objective, roadmap or POA&M status changes.

//! RRFlow Lab's tool. `settings` holds a machine's choices of which systems and
//! tests run, how RRFlow is tuned for a run and what has been approved, and
//! turns them into a run plan. The `rrflow-lab` command composes every lab
//! domain: systems, datasets, benchmarks, evaluations and definition packs.
pub mod settings;

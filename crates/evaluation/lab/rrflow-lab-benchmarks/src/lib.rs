//! RRFlow Lab's benchmarks. `manifest` is the frozen definition of one vector
//! comparison, `catalog` composes it from per-system and per-test files,
//! `driver` runs it against each system and grades every answer with the
//! dataset's exact oracle, `import` restates results from outside benchmarks,
//! and `references` holds numbers others published, shown beside our own.
pub mod catalog;
pub mod driver;
pub mod import;
pub mod manifest;
pub mod references;

//! Canonical semantic repositories over the one physical storage transaction.
//!
//! Repositories own key selection, decoding, validation, and semantic mutation
//! planning. They borrow a [`StorageEngine`](crate::StorageEngine), but the
//! engine exposes no semantic read or write methods of its own. rrflowMX and
//! rrflowKV therefore execute the same repository code and differ only in
//! volatility and physical durability.

mod claims;
mod control;
mod function;
mod invocation;
mod projection;
mod runtime;

/// Bounds retries when another physical transaction wins publication.
const PUBLICATION_ATTEMPTS: usize = 16;

pub use claims::ClaimRepository;
pub(crate) use control::prepare_control_batch;
pub use control::ControlRepository;
pub use function::FunctionCatalogueRepository;
pub use invocation::InvocationRepository;
pub use projection::ProjectionRepository;
pub use runtime::{RuntimeDataSnapshotRead, RuntimeRepository};

use crate::StorageEngine;

/// The semantic repositories over a storage port.
///
/// The port ([`StorageEngine`]) opens physical transactions and knows nothing
/// above them. This trait is implemented once for every port, so each backend
/// runs the same repository code and none can supply its own.
pub trait Repositories {
    fn claims(&self) -> ClaimRepository<'_>;
    fn control(&self) -> ControlRepository<'_>;
    fn function_catalogue(&self) -> FunctionCatalogueRepository<'_>;
    fn projections(&self) -> ProjectionRepository<'_>;
    fn runtime(&self) -> RuntimeRepository<'_>;
    fn invocations(&self) -> InvocationRepository<'_>;
}

macro_rules! repositories_over {
    () => {
        fn claims(&self) -> ClaimRepository<'_> {
            ClaimRepository::new(self)
        }

        fn control(&self) -> ControlRepository<'_> {
            ControlRepository::new(self)
        }

        fn function_catalogue(&self) -> FunctionCatalogueRepository<'_> {
            FunctionCatalogueRepository::new(self)
        }

        fn projections(&self) -> ProjectionRepository<'_> {
            ProjectionRepository::new(self)
        }

        fn runtime(&self) -> RuntimeRepository<'_> {
            RuntimeRepository::new(self)
        }

        fn invocations(&self) -> InvocationRepository<'_> {
            InvocationRepository::new(self)
        }
    };
}

impl<T: StorageEngine> Repositories for T {
    repositories_over!();
}

impl Repositories for dyn StorageEngine + '_ {
    repositories_over!();
}

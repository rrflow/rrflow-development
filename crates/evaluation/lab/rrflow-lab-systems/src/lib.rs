//! The systems RRFlow Lab runs tests against: RRFlow itself and the systems it
//! learns from. Each system has a typed description (`binding`), an adapter that
//! speaks only that system's public API (`adapters`), and one entry in the
//! `registry`, which declares what it can do. A new system is a new adapter and
//! one registry entry; no runner changes.
//!
//! Every adapter speaks only the system's public API, so no comparator code
//! enters this repository.
pub mod adapter;
pub mod adapters;
pub mod binding;
pub mod registry;
pub mod sources;

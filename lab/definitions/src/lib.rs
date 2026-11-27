//! RRFlow Lab's definition packs. Tests, systems and references ship as signed,
//! versioned bundles that can be sent and patched apart from the app, the way
//! antivirus definitions are. `pack` is the product-neutral envelope and store,
//! `home` installs and publishes packs, and `content` is RRFlow Lab's own pack
//! content and its trusted keys.
pub mod content;
pub mod home;
pub mod pack;

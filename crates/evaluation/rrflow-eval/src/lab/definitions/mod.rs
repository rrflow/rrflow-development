//! Definitions that travel like an antivirus update: signed, versioned and patchable, apart from
//! the app. `pack` is the envelope and the store, and it knows nothing about what it carries, so
//! another product can use it. `content` is what RRFlow's lab carries.
pub mod content;
pub mod pack;

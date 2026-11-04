//! Definitions that travel like an antivirus update: signed, versioned and patchable, apart from
//! the app. `pack` is the envelope and the store, and it knows nothing about what it carries, so
//! another product can use it. `home` does the steps a person would otherwise repeat, and it knows
//! no more than `pack` does. `content` is what RRFlow's lab carries.
pub mod content;
pub mod home;
pub mod pack;

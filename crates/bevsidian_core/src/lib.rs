//! Shared state of the Bevsidian app: the note vault, navigation (the [`Nav`] resource
//! and the systems that drive it).

pub mod docs;
pub mod icons;
pub mod map;
pub mod nav;
pub mod vault;

pub use nav::{Nav, View};
pub use vault::{NoteId, Vault, VaultData};

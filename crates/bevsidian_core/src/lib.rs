//! Shared state of the Bevsidian app: the note vault, navigation (the reactive [`Nav`]
//! signals and the systems that drive them).

pub mod icons;
pub mod nav;
pub mod vault;

pub use nav::{Nav, View};
pub use vault::{NoteId, Vault, VaultData};

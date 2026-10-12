//! Shared state of the Bevsidian app: the note vault, navigation (the reactive [`Nav`]
//! signals and the systems that drive them) and the theme.

pub mod nav;
pub mod theme;
pub mod vault;

pub use nav::{Nav, View};
pub use vault::{NoteId, Vault, VaultData};

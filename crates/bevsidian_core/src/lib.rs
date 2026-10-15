//! Shared state of the Bevsidian app: the note vault, navigation (the reactive [`Nav`]
//! signals and the systems that drive them).

pub mod docs;
pub mod icons;
pub mod map;
pub mod nav;
pub mod signal;
pub mod vault;

pub use nav::{Nav, View};
pub use signal::{Memo, Signal, memo, signal};
pub use vault::{NoteId, Vault, VaultData};

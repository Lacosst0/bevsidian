//! Bevsidian's UI kit: design [`tokens`] and components modelled on
//! [shadcn/ui](https://ui.shadcn.com), as BSN scene functions.
//!
//! **Need a new component? Look at shadcn/ui first** (<https://ui.shadcn.com/docs/components>,
//! source in `apps/v4/registry/new-york-v4/ui/` of github.com/shadcn-ui/ui): copy its anatomy,
//! sizes and states, map its Tailwind classes onto [`tokens`], and add it as one file under
//! `components/`. See `README.md` in this crate for the conventions.
//!
//! Every component is a function returning a `bsn!` scene (`view!` when it reads signals).
//! Use it like any scene and patch it: add `on(...)` handlers, components, or `Node` fields.
//!
//! ```ignore
//! bsn! {
//!     @{text_button(Variant::Primary, Size::Md, "Done")}
//!     on(|_: On<Activate>| info!("done"))
//! }
//! ```
//!
//! Interactive parts carry a [`Surface`]: one system paints them idle, hovered,
//! [`Selected`] or [`Disabled`], together with the [`Tint`] of the labels inside, so
//! reactive code only flips `Selected($(...))`.

pub mod components;
pub mod surface;
pub mod tokens;

use bevy::prelude::*;

pub use components::*;
pub use surface::{Disabled, Selected, Surface, SurfaceBorder, Tint};
pub use tokens::{
    color,
    font::{font, icon_font, insert, mono, size, ui_font},
    layer, radius, shadow, space,
};

/// Everything: `use bevsidian_ui::prelude::*;`.
pub mod prelude {
    pub use crate::{
        components::*,
        surface::{Disabled, Selected, Surface, SurfaceBorder, Tint},
        tokens::{
            color,
            font::{font, icon_font, insert, mono, size, ui_font},
            layer, radius, shadow, space,
        },
    };
}

/// Paints surfaces and keeps text inputs in sync.
pub fn plugin(app: &mut App) {
    app.add_plugins((
        surface::plugin,
        components::input::plugin,
        components::tooltip::plugin,
        components::slider::plugin,
    ));
}

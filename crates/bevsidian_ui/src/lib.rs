//! Bevsidian's UI kit: design tokens ([`theme`]) and components modelled on
//! [shadcn/ui](https://ui.shadcn.com), as plain scene functions.
//!
//! Every component is a function returning a `bsn!` (or reactive `view!`) scene. Use it like
//! any scene and patch it as usual: add an `on(...)` handler, extra components, or `Node`
//! fields to override.
//!
//! ```ignore
//! bsn! {
//!     @{text_button(Variant::Secondary, Size::Sm, "Done")}
//!     on(|_: On<Activate>| info!("done"))
//! }
//! ```
//!
//! Interactive surfaces (buttons, list items, toggles, badges) carry a [`Surface`]: one system
//! paints them in their idle, hovered or [`Selected`] colour, so reactive code only sets
//! `Selected($(...))`.

pub mod theme;

use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::{cursor::EntityCursor, events::PointerClick, hover::Hovered},
    prelude::*,
    text::FontWeight,
    ui_widgets::Activate,
    window::SystemCursorIcon,
};
use bevy_radioactive::view::{Signal, view};

use bevsidian_core::icons;
pub use theme::{color, font, icon_font, insert, layer, mono, radius, size, ui_font};

/// Everything: `use bevsidian_ui::prelude::*;`.
pub mod prelude {
    pub use crate::{
        Selected, Size, Surface, Variant, badge, badge_button, button, caption, card, color,
        dialog, empty, field, glyph, icon, icon_button, icon_font, inset, item, kbd, label, layer,
        muted, note_box, popover, progress, radius, scrim, section_label, separator, shortcuts,
        size, stepper, switch, text_button, tile, title, toast, toggle, toggle_group, ui_font,
    };
}

/// Paints [`Surface`]s.
pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        paint_surfaces.before(bevy::ui::UiSystems::Prepare),
    );
}

// ---------------------------------------------------------------------------
// Surfaces: hover and selection
// ---------------------------------------------------------------------------

/// Background colours of an interactive element: idle, hovered and selected.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
#[require(Hovered, BackgroundColor)]
pub struct Surface {
    pub idle: Color,
    pub hover: Color,
    pub selected: Color,
}

impl Surface {
    /// Transparent until hovered (ghost buttons, list items).
    pub const GHOST: Self = Self {
        idle: Color::NONE,
        hover: color::SECONDARY,
        selected: color::SELECTED,
    };
    /// Grey (secondary buttons).
    pub const SECONDARY: Self = Self {
        idle: color::SECONDARY,
        hover: color::SECONDARY_HOVER,
        selected: color::SELECTED,
    };
    /// Card-coloured with a border (outline buttons).
    pub const OUTLINE: Self = Self {
        idle: color::CARD,
        hover: color::SECONDARY,
        selected: color::SELECTED,
    };
    /// The accented call to action (primary buttons).
    pub const PRIMARY: Self = Self {
        idle: color::SELECTED,
        hover: color::SELECTED_HOVER,
        selected: color::SELECTED_HOVER,
    };
    /// Tinted with the primary colour (badges).
    pub const SOFT: Self = Self {
        idle: color::PRIMARY_SOFT,
        hover: color::PRIMARY_SOFT_HOVER,
        selected: color::PRIMARY_STRONG,
    };
    /// Inside a [`toggle_group`]: transparent on the group's grey.
    pub const TOGGLE: Self = Self {
        idle: Color::NONE,
        hover: color::SECONDARY_HOVER,
        selected: color::SELECTED,
    };
}

/// Whether a [`Surface`] shows its selected colour (the open note, the active tab, …).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selected(pub bool);

fn paint_surfaces(
    mut surfaces: Query<
        (&Surface, &Hovered, Option<&Selected>, &mut BackgroundColor),
        Or<(Changed<Surface>, Changed<Hovered>, Changed<Selected>)>,
    >,
) {
    for (surface, hovered, selected, mut background) in &mut surfaces {
        let color = if selected.is_some_and(|s| s.0) {
            surface.selected
        } else if hovered.get() {
            surface.hover
        } else {
            surface.idle
        };
        background.set_if_neq(BackgroundColor(color));
    }
}

// ---------------------------------------------------------------------------
// Typography and icons
// ---------------------------------------------------------------------------

/// A Lucide icon's character by name (`"pencil"`), or the default note icon.
pub fn glyph(name: &str) -> String {
    icons::find(name).unwrap_or(icons::DEFAULT).to_string()
}

/// Non-interactive text.
pub fn label(text: &str, size: f32, weight: FontWeight, color: Color) -> impl Scene + use<> {
    let text = Text::new(text);
    let font = font(size, weight);
    bsn! { text @{insert(font)} TextColor(color) Pickable::IGNORE }
}

/// A Lucide icon by name.
pub fn icon(name: &str, size: f32, color: Color) -> impl Scene + use<> {
    let text = Text::new(glyph(name));
    bsn! { text @{icon_font(size)} TextColor(color) Pickable::IGNORE }
}

/// A dialog or card title.
pub fn title(text: &str) -> impl Scene + use<> {
    label(text, size::TITLE, FontWeight::SEMIBOLD, color::FOREGROUND)
}

/// Secondary text.
pub fn muted(text: &str) -> impl Scene + use<> {
    label(text, size::SM, FontWeight::NORMAL, color::MUTED_FOREGROUND)
}

/// Hints and metadata.
pub fn caption(text: &str) -> impl Scene + use<> {
    label(text, size::XS, FontWeight::NORMAL, color::SUBTLE_FOREGROUND)
}

/// An uppercase label over a group of items (sidebar sections).
pub fn section_label(text: &str) -> impl Scene + use<> {
    let text = Text::new(text.to_uppercase());
    let font = font(11.0, FontWeight::SEMIBOLD);
    bsn! {
        text @{insert(font)} TextColor(color::SUBTLE_FOREGROUND) Pickable::IGNORE
        Node { margin: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(14.0), Val::Px(6.0)) }
    }
}

/// What a list shows when it has nothing to show.
pub fn empty(text: &str) -> impl Scene + use<> {
    let text = Text::new(text);
    let font = font(size::SM, FontWeight::NORMAL);
    bsn! {
        text @{insert(font)} TextColor(color::SUBTLE_FOREGROUND) Pickable::IGNORE
        Node { margin: UiRect::axes(Val::Px(12.0), Val::Px(6.0)) }
    }
}

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

/// Button style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// Accented call to action.
    Primary,
    /// Grey.
    Secondary,
    /// Bordered.
    Outline,
    /// Transparent until hovered.
    #[default]
    Ghost,
}

impl Variant {
    fn surface(self) -> Surface {
        match self {
            Self::Primary => Surface::PRIMARY,
            Self::Secondary => Surface::SECONDARY,
            Self::Outline => Surface::OUTLINE,
            Self::Ghost => Surface::GHOST,
        }
    }
}

/// Button size. Icon sizes are square.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// 28 px tall.
    Sm,
    /// 32 px tall.
    #[default]
    Md,
    /// 40 px tall.
    Lg,
    /// 30 px square.
    IconSm,
    /// 36 px square.
    Icon,
    /// 40 px square.
    IconLg,
}

impl Size {
    fn node(self) -> Node {
        let (width, height, padding) = match self {
            Self::Sm => (Val::Auto, 28.0, 10.0),
            Self::Md => (Val::Auto, 32.0, 12.0),
            Self::Lg => (Val::Auto, 40.0, 16.0),
            Self::IconSm => (Val::Px(30.0), 30.0, 0.0),
            Self::Icon => (Val::Px(36.0), 36.0, 0.0),
            Self::IconLg => (Val::Px(40.0), 40.0, 0.0),
        };
        Node {
            width,
            height: Val::Px(height),
            padding: UiRect::horizontal(Val::Px(padding)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            flex_shrink: 0.0,
            border_radius: BorderRadius::all(Val::Px(if self == Self::IconLg {
                radius::LG
            } else {
                radius::MD
            })),
            ..default()
        }
    }

    /// Font size of a text label.
    pub fn text(self) -> f32 {
        match self {
            Self::Sm | Self::IconSm => size::SM,
            Self::Md | Self::Icon => 13.5,
            Self::Lg | Self::IconLg => size::LG,
        }
    }

    /// Size of an icon.
    pub fn icon(self) -> f32 {
        match self {
            Self::Sm | Self::IconSm => 16.0,
            Self::Md | Self::Icon => 18.0,
            Self::Lg | Self::IconLg => 20.0,
        }
    }
}

/// A button with no content: add children (or use [`text_button`] / [`icon_button`]) and an
/// `on(|_: On<Activate>| ...)` handler.
pub fn button(variant: Variant, size: Size) -> impl Scene {
    let mut node = size.node();
    if variant == Variant::Outline {
        node.border = UiRect::all(Val::Px(1.0));
    }
    let surface = variant.surface();
    let border = BorderColor::all(if variant == Variant::Outline {
        color::INPUT
    } else {
        Color::NONE
    });
    bsn! {
        node surface border
        bevy::ui_widgets::Button
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
    }
}

/// A button with a text label.
pub fn text_button(variant: Variant, size: Size, text: &str) -> impl Scene + use<> {
    let label = label(text, size.text(), FontWeight::MEDIUM, color::FOREGROUND);
    bsn! { @{button(variant, size)} Children [ @{label} ] }
}

/// A button showing a Lucide icon.
pub fn icon_button(variant: Variant, size: Size, name: &str) -> impl Scene + use<> {
    let icon = icon(name, size.icon(), color::FOREGROUND);
    bsn! { @{button(variant, size)} Children [ @{icon} ] }
}

/// A row of [`toggle`]s on a grey track, one of them [`Selected`] (shadcn's tabs list).
pub fn toggle_group() -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::all(Val::Px(3.0)),
            column_gap: Val::Px(2.0),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(radius::LG)),
            flex_shrink: 0.0,
        }
        BackgroundColor(color::SECONDARY)
    }
}

/// An item of a [`toggle_group`]; set `Selected($(...))` on the active one.
pub fn toggle(text: &str) -> impl Scene + use<> {
    let label = label(text, size::SM, FontWeight::MEDIUM, color::FOREGROUND);
    bsn! {
        @{button(Variant::Ghost, Size::Sm)}
        Surface::TOGGLE
        Node { height: Val::Px(26.0) }
        Children [ @{label} ]
    }
}

// ---------------------------------------------------------------------------
// Badges and keys
// ---------------------------------------------------------------------------

/// A pill. For a clickable one, use [`badge_button`].
pub fn badge() -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
            column_gap: Val::Px(5.0),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(radius::FULL)),
            flex_shrink: 0.0,
        }
        BackgroundColor(color::PRIMARY_SOFT)
    }
}

/// A clickable primary-tinted pill (tags); `Selected` shows it as active.
pub fn badge_button() -> impl Scene {
    bsn! {
        @{badge()}
        Surface::SOFT
        bevy::ui_widgets::Button
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
    }
}

/// A keyboard key.
pub fn kbd(key: &str) -> impl Scene + use<> {
    let key = label(key, 11.0, FontWeight::MEDIUM, color::MUTED_FOREGROUND);
    bsn! {
        Node {
            min_width: Val::Px(20.0),
            height: Val::Px(20.0),
            padding: UiRect::horizontal(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(radius::SM)),
            border: UiRect::bottom(Val::Px(1.0)),
        }
        BackgroundColor(color::SECONDARY)
        BorderColor::all(color::INPUT)
        Pickable::IGNORE
        Children [ @{key} ]
    }
}

/// Keyboard shortcuts with what they do: `[("Ctrl O", "switch")]`. Keys are separated by
/// spaces.
pub fn shortcuts(items: &[(&str, &str)]) -> impl Scene + use<> {
    let items: Vec<Box<dyn Scene>> = items
        .iter()
        .map(|(keys, action)| {
            let keys: Vec<Box<dyn Scene>> = keys
                .split_whitespace()
                .map(|key| Box::new(kbd(key)) as Box<dyn Scene>)
                .collect();
            let action = caption(action);
            Box::new(bsn! {
                Node { column_gap: Val::Px(3.0), align_items: AlignItems::Center }
                Children [ {keys} -- Node { margin: UiRect::left(Val::Px(3.0)) } @{action} ]
            }) as Box<dyn Scene>
        })
        .collect();
    bsn! {
        Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(12.0), row_gap: Val::Px(6.0), align_items: AlignItems::Center }
        Pickable::IGNORE
        Children [ {items} ]
    }
}

// ---------------------------------------------------------------------------
// Containers
// ---------------------------------------------------------------------------

/// A bordered panel.
pub fn card() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(radius::LG)),
            min_width: Val::Px(0.0),
            overflow: Overflow::clip(),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
    }
}

/// A sunken [`card`], for embedded content such as a live preview.
pub fn inset() -> impl Scene {
    bsn! { @{card()} BackgroundColor(color::INSET) }
}

/// A small titled tile inside a card.
pub fn tile(title: &str) -> impl Scene + use<> {
    let title = label(title, 11.0, FontWeight::BOLD, color::MUTED_FOREGROUND);
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
            border_radius: BorderRadius::all(Val::Px(radius::MD)),
            min_width: Val::Px(140.0),
        }
        BackgroundColor(color::CARD)
        Children [ @{title} ]
    }
}

/// A muted note box (shadcn's alert).
pub fn note_box(text: &str) -> impl Scene + use<> {
    let text = muted(text);
    bsn! {
        Node {
            padding: UiRect::all(Val::Px(10.0)),
            border_radius: BorderRadius::all(Val::Px(radius::MD)),
        }
        BackgroundColor(color::BACKGROUND)
        Children [ @{text} ]
    }
}

/// A horizontal rule.
pub fn separator() -> impl Scene {
    bsn! {
        Node { height: Val::Px(1.0), width: Val::Percent(100.0), flex_shrink: 0.0 }
        BackgroundColor(color::BORDER)
        Pickable::IGNORE
    }
}

/// A list row: transparent until hovered, [`Selected`] when active. Patch `Node` for
/// padding or direction.
pub fn item() -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)),
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(radius::SM)),
        }
        Surface::GHOST
        bevy::ui_widgets::Button
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
    }
}

/// A labelled row of a settings form: the label, then `control`.
pub fn field(text: &str, control: impl Scene) -> impl Scene {
    let text = label(text, size::BASE, FontWeight::NORMAL, color::FOREGROUND);
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0), min_height: Val::Px(30.0) }
        Children [
            @{text} Node { flex_grow: 1.0 }
            --
            @{control}
        ]
    }
}

/// A completion bar, `fraction` from 0 to 1.
pub fn progress(fraction: f32) -> impl Scene {
    let fill = Node {
        width: Val::Percent(fraction.clamp(0.0, 1.0) * 100.0),
        ..default()
    };
    bsn! {
        Node {
            height: Val::Px(6.0),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            overflow: Overflow::clip(),
        }
        BackgroundColor(color::BORDER)
        Children [ fill BackgroundColor(color::PRIMARY) ]
    }
}

// ---------------------------------------------------------------------------
// Form controls
// ---------------------------------------------------------------------------

/// An on/off switch bound to `value`.
pub fn switch(value: Signal<bool>) -> impl Scene {
    let toggle = value.clone();
    view! {
        Node {
            width: Val::Px(40.0),
            height: Val::Px(22.0),
            padding: UiRect::all(Val::Px(3.0)),
            border_radius: BorderRadius::all(Val::Px(radius::FULL)),
            flex_shrink: 0.0,
            justify_content: $(if $value { JustifyContent::End } else { JustifyContent::Start }),
        }
        BackgroundColor($(if $value { color::PRIMARY } else { color::SECONDARY_HOVER }))
        bevy::ui_widgets::Button
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
        on(move |_: On<Activate>| toggle.update(|on| *on = !*on))
        Children [
            Node {
                width: Val::Px(16.0),
                height: Val::Px(16.0),
                border_radius: BorderRadius::all(Val::Px(radius::FULL)),
            }
            BackgroundColor(color::FOREGROUND)
            Pickable::IGNORE
        ]
    }
}

/// `−` value `+`: `value` is re-read when the signals it reads change.
pub fn stepper<V, D, I>(value: V, decrease: D, increase: I) -> impl Scene
where
    V: Fn() -> String + Clone + Send + Sync + 'static,
    D: Fn() + Clone + Send + Sync + 'static,
    I: Fn() + Clone + Send + Sync + 'static,
{
    let font = font(13.0, FontWeight::NORMAL);
    view! {
        Node { align_items: AlignItems::Center, column_gap: Val::Px(4.0) }
        Children [
            @{icon_button(Variant::Secondary, Size::IconSm, "minus")}
            on(move |_: On<Activate>| decrease())
            --
            Node { width: Val::Px(56.0), justify_content: JustifyContent::Center }
            Pickable::IGNORE
            Children [ Text($(value())) @{insert(font)} TextColor(color::MUTED_FOREGROUND) Pickable::IGNORE ]
            --
            @{icon_button(Variant::Secondary, Size::IconSm, "plus")}
            on(move |_: On<Activate>| increase())
        ]
    }
}

// ---------------------------------------------------------------------------
// Overlays
// ---------------------------------------------------------------------------

/// A full-screen dimmed backdrop; a click on it (not on its content) calls `close`. Patch
/// `Node` to place the content it holds.
pub fn scrim<F>(close: F) -> impl Scene
where
    F: Fn() + Clone + Send + Sync + 'static,
{
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
        }
        BackgroundColor(color::SCRIM)
        on(move |click: On<PointerClick>| {
            if click.entity == click.original_event_target() {
                close();
            }
        })
    }
}

/// A modal panel `width` px wide (narrower on small screens).
pub fn dialog(width: f32) -> impl Scene {
    let node = Node {
        flex_direction: FlexDirection::Column,
        width: Val::Px(width),
        max_width: Val::Percent(92.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(14.0),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(radius::XL)),
        ..default()
    };
    let shadow = theme::shadow_lg();
    bsn! { node BackgroundColor(color::CARD) BorderColor::all(color::BORDER) shadow }
}

/// A floating surface: menus and toolbars.
pub fn popover() -> impl Scene {
    let shadow = theme::shadow_md();
    bsn! {
        Node {
            padding: UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(radius::XL)),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        shadow
    }
}

/// A short notice pill.
pub fn toast(text: &str) -> impl Scene + use<> {
    let text = label(text, size::BASE, FontWeight::MEDIUM, color::FOREGROUND);
    bsn! {
        Node {
            padding: UiRect::axes(Val::Px(14.0), Val::Px(7.0)),
            border_radius: BorderRadius::all(Val::Px(radius::LG)),
        }
        BackgroundColor(color::ELEVATED)
        Pickable::IGNORE
        Children [ @{text} ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_used_by_components_exist() {
        for name in [
            "minus",
            "plus",
            "chevron-left",
            "chevron-right",
            "panel-left",
            "panel-right",
            "sliders-horizontal",
            "x",
            "maximize",
            "search",
        ] {
            assert!(icons::find(name).is_some(), "missing icon {name}");
        }
    }

    #[test]
    fn surfaces_paint_their_state() {
        let mut app = App::new();
        app.add_systems(Update, paint_surfaces);
        let id = app
            .world_mut()
            .spawn((Surface::GHOST, Selected(false)))
            .id();
        app.update();
        let bg = |app: &App| app.world().get::<BackgroundColor>(id).unwrap().0;
        assert_eq!(bg(&app), Color::NONE);
        app.world_mut().get_mut::<Selected>(id).unwrap().0 = true;
        app.update();
        assert_eq!(bg(&app), color::SELECTED);
        app.world_mut().get_mut::<Selected>(id).unwrap().0 = false;
        app.world_mut().entity_mut(id).insert(Hovered(true));
        app.update();
        assert_eq!(bg(&app), color::SECONDARY);
    }
}

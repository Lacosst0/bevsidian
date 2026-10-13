//! Interaction states. A [`Surface`] is anything you can hover or select (buttons, rows, tabs);
//! one system paints its background, and the [`Tint`] of the labels and icons inside it, for
//! its idle, hovered, [`Selected`] or [`Disabled`] state. Reactive code only flips
//! `Selected($(...))` or `Disabled($(...))`.

use bevy::{picking::hover::Hovered, prelude::*, ui::InteractionDisabled};

use crate::tokens::color;

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (sync_disabled, paint_surfaces, paint_new_tints)
            .chain()
            .before(bevy::ui::UiSystems::Prepare),
    );
}

/// Background colours of an interactive element.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
#[require(Hovered, BackgroundColor, Selected, Disabled)]
pub struct Surface {
    pub idle: Color,
    pub hover: Color,
    pub selected: Color,
}

impl Surface {
    /// Transparent until hovered: ghost buttons, list rows, menu items.
    pub const GHOST: Self = Self {
        idle: Color::NONE,
        hover: color::HOVER,
        selected: color::SELECTED,
    };
    /// Grey: secondary buttons.
    pub const SECONDARY: Self = Self {
        idle: color::ACCENT,
        hover: color::ACCENT_HOVER,
        selected: color::ACCENT_HOVER,
    };
    /// Faint fill inside a border: outline buttons.
    pub const OUTLINE: Self = Self {
        idle: color::INPUT_BG,
        hover: color::INPUT_HOVER,
        selected: color::SELECTED,
    };
    /// White: the one call to action of a dialog.
    pub const PRIMARY: Self = Self {
        idle: color::PRIMARY,
        hover: color::PRIMARY_HOVER,
        selected: color::PRIMARY,
    };
    /// A tab on its grey track; the selected one takes the page colour.
    pub const TAB: Self = Self {
        idle: Color::NONE,
        hover: color::HOVER,
        selected: color::BACKGROUND,
    };
    /// A segment of a [`segmented`](crate::segmented) control; the selected one is white.
    pub const SEGMENT: Self = Self {
        idle: Color::NONE,
        hover: color::HOVER,
        selected: color::PRIMARY,
    };
}

/// Border colours of a [`Surface`] that has a border, idle and selected (a selected tab is
/// outlined). Without it the border keeps its colour.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfaceBorder {
    pub idle: Color,
    pub selected: Color,
}

/// Whether a [`Surface`] shows its selected state (the open note, the active tab, …).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selected(pub bool);

/// Whether a [`Surface`] ignores input (a Back button with no history). Mirrors Bevy's
/// [`InteractionDisabled`], so a disabled button sends no `Activate`.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Disabled(pub bool);

/// The colour of a label or icon inside a [`Surface`], per state of the nearest surface above
/// it. Disabled surfaces show [`color::DISABLED_FOREGROUND`].
#[derive(Component, Clone, Copy, Debug, PartialEq)]
#[require(TextColor)]
pub struct Tint {
    pub idle: Color,
    pub hover: Color,
    pub selected: Color,
}

impl Default for Tint {
    fn default() -> Self {
        Self::TEXT
    }
}

impl Tint {
    /// The same colour in every state (still dimmed when disabled).
    pub const fn fixed(color: Color) -> Self {
        Self {
            idle: color,
            hover: color,
            selected: color,
        }
    }
    /// Row labels: body text that brightens when hovered or selected.
    pub const TEXT: Self = Self {
        idle: color::BODY,
        hover: color::FOREGROUND,
        selected: color::FOREGROUND,
    };
    /// Row icons, ghost buttons, idle tabs: muted until hovered or selected.
    pub const MUTED: Self = Self {
        idle: color::MUTED_FOREGROUND,
        hover: color::FOREGROUND,
        selected: color::FOREGROUND,
    };
    /// Text on a [`Surface::SEGMENT`]: dark once its segment turns white.
    pub const SEGMENT: Self = Self {
        idle: color::BODY,
        hover: color::FOREGROUND,
        selected: color::PRIMARY_FOREGROUND,
    };

    fn pick(self, state: State) -> Color {
        match state {
            State::Idle => self.idle,
            State::Hover => self.hover,
            State::Selected => self.selected,
            State::Disabled => color::DISABLED_FOREGROUND,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Hover,
    Selected,
    Disabled,
}

impl State {
    fn of(hovered: Hovered, selected: Selected, disabled: Disabled) -> Self {
        if disabled.0 {
            Self::Disabled
        } else if selected.0 {
            Self::Selected
        } else if hovered.get() {
            Self::Hover
        } else {
            Self::Idle
        }
    }

    fn background(self, surface: &Surface) -> Color {
        match self {
            Self::Idle | Self::Disabled => surface.idle,
            Self::Hover => surface.hover,
            Self::Selected => surface.selected,
        }
    }
}

fn sync_disabled(surfaces: Query<(Entity, &Disabled), Changed<Disabled>>, mut commands: Commands) {
    for (entity, disabled) in &surfaces {
        // `try_`: a reactive region may despawn the surface before the commands apply.
        if disabled.0 {
            commands.entity(entity).try_insert(InteractionDisabled);
        } else {
            commands.entity(entity).try_remove::<InteractionDisabled>();
        }
    }
}

type SurfaceState<'a> = (
    Entity,
    &'a Surface,
    &'a Hovered,
    &'a Selected,
    &'a Disabled,
    &'a mut BackgroundColor,
    Option<(&'a SurfaceBorder, &'a mut BorderColor)>,
);

fn paint_surfaces(
    mut surfaces: Query<
        SurfaceState,
        Or<(
            Changed<Surface>,
            Changed<Hovered>,
            Changed<Selected>,
            Changed<Disabled>,
        )>,
    >,
    nested: Query<(), With<Surface>>,
    children: Query<&Children>,
    mut tints: Query<(&Tint, &mut TextColor)>,
) {
    for (entity, surface, hovered, selected, disabled, mut background, border) in &mut surfaces {
        let state = State::of(*hovered, *selected, *disabled);
        background.set_if_neq(BackgroundColor(state.background(surface)));
        if let Some((colors, mut border)) = border {
            let color = if state == State::Selected {
                colors.selected
            } else {
                colors.idle
            };
            border.set_if_neq(BorderColor::all(color));
        }
        paint_tints(entity, state, &nested, &children, &mut tints);
    }
}

/// Sets the tints below `root`, stopping at nested surfaces (they paint their own).
fn paint_tints(
    root: Entity,
    state: State,
    nested: &Query<(), With<Surface>>,
    children: &Query<&Children>,
    tints: &mut Query<(&Tint, &mut TextColor)>,
) {
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if let Ok((tint, mut text)) = tints.get_mut(entity) {
            text.set_if_neq(TextColor(tint.pick(state)));
        }
        for &child in children.get(entity).into_iter().flatten() {
            if !nested.contains(child) {
                stack.push(child);
            }
        }
    }
}

/// Labels spawned later inside an existing surface (a reactive region re-rendering) take
/// the surface's current state.
fn paint_new_tints(
    mut tints: Query<(Entity, &Tint, &mut TextColor), Added<Tint>>,
    parents: Query<&ChildOf>,
    surfaces: Query<(&Hovered, &Selected, &Disabled), With<Surface>>,
) {
    for (entity, tint, mut text) in &mut tints {
        let state = std::iter::once(entity)
            .chain(parents.iter_ancestors(entity))
            .find_map(|e| surfaces.get(e).ok())
            .map_or(State::Idle, |(h, s, d)| State::of(*h, *s, *d));
        text.set_if_neq(TextColor(tint.pick(state)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (sync_disabled, paint_surfaces, paint_new_tints).chain(),
        );
        app
    }

    #[test]
    fn surfaces_paint_their_state_and_tints() {
        let mut app = app();
        let surface = app.world_mut().spawn(Surface::GHOST).id();
        let label = app.world_mut().spawn((Tint::MUTED, ChildOf(surface))).id();
        app.update();
        let bg = |app: &App| app.world().get::<BackgroundColor>(surface).unwrap().0;
        let fg = |app: &App| app.world().get::<TextColor>(label).unwrap().0;
        assert_eq!(bg(&app), Color::NONE);
        assert_eq!(fg(&app), color::MUTED_FOREGROUND);

        app.world_mut().get_mut::<Selected>(surface).unwrap().0 = true;
        app.update();
        assert_eq!(bg(&app), color::SELECTED);
        assert_eq!(fg(&app), color::FOREGROUND);

        app.world_mut().get_mut::<Selected>(surface).unwrap().0 = false;
        app.world_mut().entity_mut(surface).insert(Hovered(true));
        app.update();
        assert_eq!(bg(&app), color::HOVER);
    }

    #[test]
    fn disabled_wins_and_blocks_interaction() {
        let mut app = app();
        let surface = app
            .world_mut()
            .spawn((
                Surface::GHOST,
                Selected(true),
                Disabled(true),
                Hovered(true),
            ))
            .id();
        let label = app.world_mut().spawn((Tint::TEXT, ChildOf(surface))).id();
        app.update();
        assert!(app.world().get::<InteractionDisabled>(surface).is_some());
        assert_eq!(
            app.world().get::<BackgroundColor>(surface).unwrap().0,
            Color::NONE
        );
        assert_eq!(
            app.world().get::<TextColor>(label).unwrap().0,
            color::DISABLED_FOREGROUND
        );
        app.world_mut().get_mut::<Disabled>(surface).unwrap().0 = false;
        app.update();
        assert!(app.world().get::<InteractionDisabled>(surface).is_none());
    }

    #[test]
    fn nested_surfaces_keep_their_own_tints() {
        let mut app = app();
        let outer = app.world_mut().spawn((Surface::GHOST, Selected(true))).id();
        let inner = app.world_mut().spawn((Surface::GHOST, ChildOf(outer))).id();
        let label = app.world_mut().spawn((Tint::MUTED, ChildOf(inner))).id();
        app.update();
        assert_eq!(
            app.world().get::<TextColor>(label).unwrap().0,
            color::MUTED_FOREGROUND
        );
    }

    #[test]
    fn late_tints_take_the_current_state() {
        let mut app = app();
        let surface = app.world_mut().spawn((Surface::GHOST, Selected(true))).id();
        app.update();
        let label = app.world_mut().spawn((Tint::MUTED, ChildOf(surface))).id();
        app.update();
        assert_eq!(
            app.world().get::<TextColor>(label).unwrap().0,
            color::FOREGROUND
        );
    }
}

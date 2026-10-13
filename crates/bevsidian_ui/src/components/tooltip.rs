//! Tooltips (shadcn/ui `Tooltip`): a short label, and optionally its shortcut, shown under an
//! element while a mouse rests on it. Add a [`Tooltip`] to any element, usually an icon
//! button: `@{icon_button(..)} Tooltip::new("Back")`.
//!
//! The first tooltip waits a moment; moving on to the next element shows its tooltip at once.
//! Touch never shows them (there is no hover), and pressing the element hides its tooltip.

use bevy::{
    picking::{hover::HoverMap, pointer::PointerId},
    prelude::*,
    text::FontWeight,
    ui::UiGlobalTransform,
    window::{PrimaryWindow, RequestRedraw},
};

use crate::{
    components::typography::label,
    tokens::{color, font::size, layer, radius},
};

/// The label shown while the pointer rests on this element.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Tooltip {
    pub text: &'static str,
    /// A shortcut shown after the text, e.g. `"Ctrl O"`.
    pub keys: Option<&'static str>,
}

impl Tooltip {
    pub const fn new(text: &'static str) -> Self {
        Self { text, keys: None }
    }

    pub const fn with_keys(text: &'static str, keys: &'static str) -> Self {
        Self {
            text,
            keys: Some(keys),
        }
    }
}

/// How long the pointer rests before the first tooltip shows (seconds).
const DELAY: f64 = 0.5;
/// After a tooltip hides, the next shows at once for this long (seconds).
const GRACE: f64 = 0.4;
/// Space between an element and its tooltip.
const GAP: f32 = 6.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<TooltipState>()
        .add_systems(Update, track_tooltips)
        .add_systems(
            PostUpdate,
            place_tooltips.after(bevy::ui::UiSystems::Layout),
        );
}

#[derive(Resource, Default)]
struct TooltipState {
    /// The element under the mouse that has a tooltip, and since when.
    anchor: Option<(Entity, f64)>,
    /// The tooltip on screen.
    shown: Option<Entity>,
    /// When the last tooltip hid.
    hidden_at: f64,
    /// The anchor was pressed: no tooltip until the mouse leaves it.
    pressed: bool,
}

/// The tooltip on screen.
#[derive(Component)]
struct TooltipPopup {
    anchor: Entity,
}

fn track_tooltips(
    hover: Res<HoverMap>,
    time: Res<Time<Real>>,
    mouse: Res<ButtonInput<MouseButton>>,
    tooltips: Query<&Tooltip>,
    parents: Query<&ChildOf>,
    mut state: ResMut<TooltipState>,
    mut commands: Commands,
    mut redraw: MessageWriter<RequestRedraw>,
) {
    let now = time.elapsed_secs_f64();
    // The innermost element with a tooltip under the mouse (touch has no hover).
    let anchor = hover
        .iter()
        .filter(|(pointer, _)| !matches!(pointer, PointerId::Touch(_)))
        .flat_map(|(_, hits)| hits.keys())
        .find_map(|&hit| {
            std::iter::once(hit)
                .chain(parents.iter_ancestors(hit))
                .find(|&e| tooltips.contains(e))
        });
    if anchor != state.anchor.map(|(e, _)| e) {
        state.anchor = anchor.map(|e| (e, now));
        state.pressed = false;
        hide(&mut state, &mut commands, now);
    }
    if mouse.get_just_pressed().next().is_some() {
        state.pressed = true;
        hide(&mut state, &mut commands, now);
    }
    let Some((anchor, since)) = state.anchor else {
        return;
    };
    if state.shown.is_some() || state.pressed {
        return;
    }
    let wait = if now - state.hidden_at < GRACE {
        0.0
    } else {
        DELAY
    };
    if now - since < wait {
        // The app only renders on input: keep frames coming until the tooltip is due.
        redraw.write(RequestRedraw);
        return;
    }
    let Ok(tooltip) = tooltips.get(anchor) else {
        return;
    };
    let popup = commands
        .spawn_scene(popup(*tooltip))
        .insert((TooltipPopup { anchor }, Visibility::Hidden))
        .id();
    state.shown = Some(popup);
}

fn hide(state: &mut TooltipState, commands: &mut Commands, now: f64) {
    if let Some(popup) = state.shown.take() {
        commands.entity(popup).try_despawn();
        state.hidden_at = now;
    }
}

fn popup(tooltip: Tooltip) -> impl Scene {
    let keys: Vec<Box<dyn Scene>> = tooltip
        .keys
        .map(|keys| {
            Box::new(label(
                keys,
                size::XS,
                FontWeight::NORMAL,
                color::MUTED_FOREGROUND,
            )) as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            column_gap: px(8),
            padding: UiRect::axes(px(12), px(6)),
            border_radius: BorderRadius::all(px(radius::MD)),
        }
        BackgroundColor(color::PRIMARY)
        GlobalZIndex(layer::TOOLTIP)
        Pickable::IGNORE
        Children [
            @{label(tooltip.text, size::XS, FontWeight::MEDIUM, color::PRIMARY_FOREGROUND)}
            --
            {keys}
        ]
    }
}

/// Puts the tooltip under its element, centred and inside the window (above it when there
/// is no room below), and shows it once placed.
fn place_tooltips(
    anchors: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut popups: Query<(&TooltipPopup, &ComputedNode, &mut Node, &mut Visibility)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    for (popup, computed, mut node, mut visibility) in &mut popups {
        let Ok((anchor, at)) = anchors.get(popup.anchor) else {
            continue;
        };
        let scale = anchor.inverse_scale_factor;
        let (center, size) = (at.translation * scale, anchor.size * scale);
        let own = computed.size * computed.inverse_scale_factor;
        if own.x <= 0.0 {
            continue;
        }
        let left = (center.x - own.x / 2.0)
            .min(window.width() - own.x - 8.0)
            .max(8.0);
        let below = center.y + size.y / 2.0 + GAP;
        let top = if below + own.y > window.height() - 8.0 {
            center.y - size.y / 2.0 - GAP - own.y
        } else {
            below
        };
        if node.left != px(left) || node.top != px(top) {
            node.left = px(left);
            node.top = px(top);
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}

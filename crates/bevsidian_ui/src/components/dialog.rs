//! Modal dialogs (shadcn/ui `Dialog`): a dimmed backdrop, a centred panel, its header and
//! footer.

use bevy::{picking::events::PointerClick, prelude::*, ui_widgets::Activate};
use bevy_ui_touchscroll::TouchScroll;

use crate::{
    components::{
        button::{Size, Variant, icon_button},
        typography::title,
    },
    tokens::{color, radius, shadow},
};

/// A full-screen dimmed backdrop; a tap on it (not on what it holds, and not a touch that
/// scrolled) runs `close` on the world, say to clear the resource field that keeps it open.
/// Patch `Node` to place its content.
pub fn scrim<F>(close: F) -> impl Scene
where
    F: Fn(&mut World) + Copy + Send + Sync + 'static,
{
    bsn! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        BackgroundColor(color::SCRIM)
        on(move |click: On<PointerClick>, touch: Option<Res<TouchScroll>>, mut commands: Commands| {
            let scrolled = touch.is_some_and(|touch| touch.gesture_scrolled());
            if click.entity == click.original_event_target() && !scrolled {
                commands.queue(move |world: &mut World| close(world));
            }
        })
    }
}

/// The panel, `width` px wide and never wider than the screen allows.
pub fn dialog(width: f32) -> impl Scene {
    let shadow = shadow::dialog();
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            width: px(width),
            max_width: Val::Percent(94.0),
            padding: UiRect::all(px(24)),
            row_gap: px(20),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::XXL)),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        shadow
    }
}

/// The title of a dialog with its close button, which runs `close` on the world.
pub fn dialog_header<F>(text: &str, close: F) -> impl Scene + use<F>
where
    F: Fn(&mut World) + Copy + Send + Sync + 'static,
{
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: px(12) }
        Children [
            @{title(text)} Node { flex_grow: 1.0 }
            --
            @{icon_button(Variant::Ghost, Size::IconXs, "x")}
            Node { margin: UiRect::right(px(-8)) }
            on(move |_: On<Activate>, mut commands: Commands| commands.queue(move |world: &mut World| close(world)))
        ]
    }
}

/// The row of buttons at the bottom of a dialog, aligned to the end.
pub fn dialog_footer() -> impl Scene {
    bsn! { Node { align_items: AlignItems::Center, column_gap: px(8), justify_content: JustifyContent::End } }
}

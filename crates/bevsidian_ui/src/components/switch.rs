//! A switch (shadcn/ui `Switch`) bound to a boolean signal.

use bevy::{
    input_focus::tab_navigation::TabIndex, picking::cursor::EntityCursor, prelude::*,
    ui_widgets::Activate, window::SystemCursorIcon,
};
use bevy_radioactive::view::{Signal, view};

use crate::tokens::{color, radius};

/// On (white track, dark thumb at the end) or off (grey track, white thumb at the start).
pub fn switch(value: Signal<bool>) -> impl Scene {
    let toggle = value.clone();
    view! {
        Node {
            width: px(32),
            height: px(18),
            padding: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(radius::FULL)),
            flex_shrink: 0.0,
            justify_content: $(if $value { JustifyContent::End } else { JustifyContent::Start }),
        }
        BackgroundColor($(if $value { color::PRIMARY } else { color::INPUT }))
        bevy::ui_widgets::Button
        TabIndex(0)
        EntityCursor::System(SystemCursorIcon::Pointer)
        on(move |_: On<Activate>| toggle.update(|on| *on = !*on))
        Children [
            Node { width: px(14), height: px(14), border_radius: BorderRadius::all(px(radius::FULL)) }
            BackgroundColor($(if $value { color::BACKGROUND } else { color::FOREGROUND }))
            Pickable::IGNORE
        ]
    }
}

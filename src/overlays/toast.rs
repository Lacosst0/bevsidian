//! "Copied to clipboard", shown briefly at the bottom after copying a selection from the menu
//! or the touch toolbar. Copying again while it shows keeps it longer instead of stacking.

use bevy::prelude::*;
use bevy_ui_commonmark::CopiedBubble;
use bevy_ui_textselect::{CopySelection, TextSelection};

use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(on_copy);
}

/// How long it stays (seconds).
const SECONDS: f64 = 1.4;

/// A toast centred at the bottom of the window.
#[derive(SceneComponent, Default, Clone)]
#[scene(ToastProps)]
pub struct Toast;

#[derive(Default)]
pub struct ToastProps {
    pub text: String,
}

impl Toast {
    fn scene(props: ToastProps) -> impl Scene {
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                bottom: px(84),
                justify_content: JustifyContent::Center,
            }
            GlobalZIndex(layer::TOAST)
            Pickable::IGNORE
            Children [ @{toast(&props.text)} ]
        }
    }
}

fn on_copy(
    _: On<CopySelection>,
    selection: Res<TextSelection>,
    time: Res<Time<Real>>,
    toasts: Query<Entity, With<Toast>>,
    mut commands: Commands,
) {
    if selection.is_collapsed() {
        return;
    }
    // Removed by the markdown plugin at `until`, like a code block's "Copied!" bubble.
    let until = time.elapsed_secs_f64() + SECONDS;
    if let Some(toast) = toasts.iter().next() {
        commands.entity(toast).try_insert(CopiedBubble { until });
        return;
    }
    commands
        .spawn_scene(bsn! { @Toast { @text: "Copied to clipboard" } })
        .insert(CopiedBubble { until });
}

//! The on-screen keyboard. A WebGL canvas cannot summon a phone's keyboard, so focusing a text
//! field by touch docks a Feathers [`VirtualKeyboard`] at the bottom; its keys type into the
//! field that had focus.

use bevy::{
    feathers::controls::{VirtualKeyPressed, VirtualKeyboard},
    input_focus::InputFocus,
    prelude::*,
    text::{EditableText, TextEdit},
};
use bevy_radioactive::view::view;

use bevsidian_core::Nav;
use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<KeyboardTarget>()
        .add_systems(Update, track_keyboard_target);
}

type Keys = VirtualKeyboard<&'static str>;

/// The text field the on-screen keyboard types into.
#[derive(Resource, Default)]
struct KeyboardTarget(Option<Entity>);

/// Root of the on-screen keyboard.
#[derive(Component, Clone, Copy, Debug, Default)]
struct KeyboardPanel;

fn track_keyboard_target(
    focus: Res<InputFocus>,
    editable: Query<(), With<EditableText>>,
    panel: Query<(), With<KeyboardPanel>>,
    parents: Query<&ChildOf>,
    mut target: ResMut<KeyboardTarget>,
    nav: Res<Nav>,
) {
    if !focus.is_changed() {
        return;
    }
    match focus.get() {
        Some(entity) if editable.contains(entity) => target.0 = Some(entity),
        // Pressing a key focuses the key's button: keep typing into the field.
        Some(entity) if parents.iter_ancestors(entity).any(|e| panel.contains(e)) => {}
        _ => target.0 = None,
    }
    nav.keyboard.set_if_neq(target.0.is_some());
}

const ROWS: [&[&str]; 5] = [
    &["#", "*", "-", "`", "[", "]", "(", ")", ".", ","],
    &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    &["a", "s", "d", "f", "g", "h", "j", "k", "l"],
    &["z", "x", "c", "v", "b", "n", "m", "del"],
    &["space", "enter", "Done"],
];

fn on_virtual_key(
    key: On<VirtualKeyPressed<&'static str>>,
    mut target: ResMut<KeyboardTarget>,
    mut fields: Query<&mut EditableText>,
    mut focus: ResMut<InputFocus>,
    nav: Res<Nav>,
) {
    let Some(entity) = target.0 else {
        return;
    };
    match key.key {
        "Done" => {
            target.0 = None;
            focus.clear();
            nav.keyboard.set(false);
            return;
        }
        "del" => {
            if let Ok(mut field) = fields.get_mut(entity) {
                field.queue_edit(TextEdit::Backspace);
            }
        }
        k => {
            if let Ok(mut field) = fields.get_mut(entity) {
                let text = match k {
                    "space" => " ",
                    // New line in the note editor; single-line fields ignore it.
                    "enter" if field.allow_newlines => "\n",
                    "enter" => "",
                    k => k,
                };
                if !text.is_empty() {
                    field.queue_edit(TextEdit::Insert(text.into()));
                }
            }
        }
    }
    focus.set(entity, bevy::input_focus::FocusCause::Navigated);
}

/// The on-screen keyboard, docked at the bottom while a text field is focused by touch.
pub fn keyboard(nav: Nav) -> impl Scene {
    let (keyboard, touch) = (nav.keyboard.clone(), nav.touch.clone());
    view! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::End,
            flex_direction: FlexDirection::Column,
        }
        GlobalZIndex(layer::KEYBOARD)
        Pickable::IGNORE
        Children [
            if $keyboard && $touch {
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(8.0), Val::Px(14.0)),
                    border: UiRect::top(Val::Px(1.0)),
                }
                BackgroundColor(color::CARD)
                BorderColor::all(color::BORDER)
                KeyboardPanel
                Children [
                    @Keys {
                        @keys: {ROWS.iter().map(|row| row.to_vec()).collect::<Vec<_>>()}
                    }
                    Node { width: Val::Percent(100.0) }
                    on(on_virtual_key)
                ]
            }
        ]
    }
}

//! A one-line text field (shadcn/ui `Input`) mirrored into a signal, with a leading icon, a
//! placeholder and a clear button.

use bevsidian_core::signal::{Signal, signal};
use bevy::{
    ecs::template::FromTemplate,
    input_focus::{AutoFocus, InputFocus, tab_navigation::TabIndex},
    picking::cursor::EntityCursor,
    prelude::*,
    text::{EditableText, FontWeight, TextCursorStyle},
    ui_widgets::{Activate, TextInput},
    window::SystemCursorIcon,
};
use bevy_radioactive::view::view;

use crate::{
    components::{
        button::{Size, Variant, icon_button},
        icon::icon,
        typography::label,
    },
    tokens::{color, font::size, radius},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (mirror_values, show_focus, track_focus_within));
}

/// Keeps the signal true while keyboard focus is on this entity or inside it: a search field
/// and the suggestions under it, say, so the suggestions stay open while one is clicked.
#[derive(Component, Clone)]
pub struct FocusWithin(pub Signal<bool>);

/// Mirrors the field's text into the signal whenever it is edited.
#[derive(Component, Clone)]
pub struct InputValue(pub Signal<String>);

/// The bordered box around a field; its border brightens while the field has focus.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct InputFrame;

/// A button that empties the named field.
#[derive(Component, FromTemplate)]
pub struct ClearsInput(pub Entity);

/// A one-line text field. Include it with props, all optional:
///
/// ```ignore
/// @InputField { @icon: "search", @placeholder: "Search notes…", @value: Some(query), @autofocus: true }
/// ```
///
/// The placeholder shows while the field is empty and a clear button while it is not. `value`
/// follows what is typed; it does not write back into the field, except that clearing
/// empties both.
#[derive(SceneComponent, Default, Clone)]
#[scene(InputFieldProps)]
pub struct InputField;

/// Props of an [`InputField`].
#[derive(Default)]
pub struct InputFieldProps {
    /// A Lucide icon before the text (none when empty).
    pub icon: &'static str,
    pub placeholder: String,
    /// Mirrors the text.
    pub value: Option<Signal<String>>,
    /// Focus the field when it appears.
    pub autofocus: bool,
}

impl InputField {
    fn scene(props: InputFieldProps) -> impl Scene {
        let value = props.value.unwrap_or_else(|| signal(String::new));
        let leading: Vec<Box<dyn Scene>> = (!props.icon.is_empty())
            .then(|| Box::new(icon(props.icon, 16.0, color::MUTED_FOREGROUND)) as Box<dyn Scene>)
            .into_iter()
            .collect();
        let focus: Box<dyn Scene> = if props.autofocus {
            Box::new(bsn! { AutoFocus })
        } else {
            Box::new(())
        };
        let hint = label(
            &props.placeholder,
            size::BASE,
            FontWeight::NORMAL,
            color::MUTED_FOREGROUND,
        );
        let mirror = InputValue(value.clone());
        view! {
            Node {
                height: px(36),
                padding: UiRect::new(px(10), px(4), px(0), px(0)),
                column_gap: px(8),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(radius::MD)),
                min_width: px(0),
                flex_shrink: 0.0,
            }
            InputFrame
            BackgroundColor(color::INPUT_BG)
            BorderColor::all(color::INPUT)
            Children [
                {leading}
                --
                Node { flex_grow: 1.0, min_width: px(0), height: px(20), overflow: Overflow::clip() }
                Children [
                    @{hint}
                    Node { position_type: PositionType::Absolute, display: $(if $value.is_empty() { Display::Flex } else { Display::None }) }
                    TextLayout { linebreak: LineBreak::NoWrap }
                    --
                    #Field
                    Node { flex_grow: 1.0 }
                    mirror
                    TextInput
                    EditableText { cursor_width: 0.1 }
                    TextLayout { linebreak: LineBreak::NoWrap }
                    @{crate::ui_font(size::BASE, FontWeight::NORMAL)}
                    TextColor(color::FOREGROUND)
                    TextCursorStyle {
                        color: color::FOREGROUND,
                        selection_color: color::LINK_RING,
                        unfocused_selection_color: color::SELECTED,
                    }
                    TabIndex(0)
                    EntityCursor::System(SystemCursorIcon::Text)
                    @{focus}
                ]
                --
                @{icon_button(Variant::Ghost, Size::IconXs, "circle-x")}
                Node { display: $(if $value.is_empty() { Display::None } else { Display::Flex }) }
                ClearsInput(#Field)
                on(clear)
            ]
        }
    }
}

fn clear(
    activate: On<Activate>,
    buttons: Query<&ClearsInput>,
    mut fields: Query<(&mut EditableText, Option<&InputValue>)>,
    mut focus: ResMut<InputFocus>,
) {
    let Ok(ClearsInput(field)) = buttons.get(activate.entity) else {
        return;
    };
    if let Ok((mut text, value)) = fields.get_mut(*field) {
        text.clear();
        if let Some(value) = value {
            value.0.set_if_neq(String::new());
        }
        focus.set(*field, bevy::input_focus::FocusCause::Navigated);
    }
}

fn mirror_values(fields: Query<(&EditableText, &InputValue), Changed<EditableText>>) {
    for (text, value) in &fields {
        value.0.set_if_neq(text.value().to_string());
    }
}

fn track_focus_within(
    focus: Res<InputFocus>,
    parents: Query<&ChildOf>,
    areas: Query<(Entity, &FocusWithin)>,
) {
    if !focus.is_changed() {
        return;
    }
    let focused = focus.get();
    for (area, within) in &areas {
        let inside =
            focused.is_some_and(|f| f == area || parents.iter_ancestors(f).any(|a| a == area));
        within.0.set_if_neq(inside);
    }
}

fn show_focus(
    focus: Res<InputFocus>,
    parents: Query<&ChildOf>,
    mut frames: Query<(Entity, &mut BorderColor), With<InputFrame>>,
) {
    if !focus.is_changed() {
        return;
    }
    let focused = focus.get();
    for (frame, mut border) in &mut frames {
        let inside =
            focused.is_some_and(|f| f == frame || parents.iter_ancestors(f).any(|a| a == frame));
        border.set_if_neq(BorderColor::all(if inside {
            color::MUTED_FOREGROUND
        } else {
            color::INPUT
        }));
    }
}

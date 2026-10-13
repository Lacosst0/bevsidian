---
icon: keyboard
tags: [ui, intermediate]
---
# Text input

A field where the player types: a name, a chat message, a search box. #ui #intermediate

## The two parts

Bevy 0.20 splits text input in two:

- [`EditableText`] (in `bevy::text`) holds the text, cursor and selection.
- [`TextInput`] (in `bevy::ui_widgets`) turns key presses into edits, handles read-only and copy/paste.

Put both on a node, and give the window **focus** on it:

```rust
use bevy::{
    input_focus::{tab_navigation::{TabGroup, TabNavigationPlugin}, AutoFocus},
    prelude::*,
    text::EditableText,
    ui_widgets::TextInput,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TabNavigationPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node { width: px(300), padding: UiRect::all(px(8)), border: UiRect::all(px(2)), ..default() },
        BorderColor::all(Color::srgb(0.6, 0.6, 0.7)),
        EditableText::default(),
        TextInput,
        AutoFocus,                    // start with the keyboard here
        TabGroup::new(0),
    ));
}
```

Read what the player typed from `EditableText` (`value()`), usually when they press Enter, see Bevy's `text_input` example for submission.

## Several fields

`InputFocus` (resource) says which entity gets the keyboard. Clicking an input focuses it, Tab moves between inputs ([[Buttons and widgets]]).

## Numbers

For numeric fields with drag-to-change, the Feathers widgets include a number input ([[Theming]]).

## Libraries

[[bevy_simple_text_input]] is a small, popular alternative; [[bevy_egui]] has full-featured text widgets for tools and debug menus.

## Next

[[Theming]].

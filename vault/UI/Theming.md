---
icon: palette
tags: [ui, intermediate]
---
# Theming

**Feathers** is Bevy's own set of styled, themeable widgets: buttons, sliders, checkboxes, number inputs, menus, tabs. It is the quickest way to a decent-looking UI, and the basis for Bevy's editor. #ui #intermediate

## Add the plugin and a theme

Feathers is behind a Cargo feature: `bevy = { version = "0.20", features = ["bevy_feathers"] }`.

```rust
use bevy::{
    feathers::{
        controls::FeathersButton,
        dark_theme::create_dark_theme,
        display::caption,
        theme::{ThemeBackgroundColor, UiTheme},
        tokens, FeathersPlugins,
    },
    prelude::*,
    ui_widgets::Activate,
};

#[derive(Resource)]
struct Counter(i32);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(Counter(0))
        .add_systems(Startup, scene.spawn())
        .run();
}

fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @root()
    }
}

fn root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            @FeathersButton
            on(|_: On<Activate>, mut counter: ResMut<Counter>| counter.0 += 1)
            Children [ @caption("+1") ]
        ]
    }
}
```

- `FeathersPlugins` adds the widgets and their behaviour.
- `UiTheme` is a resource that maps **tokens** (`WINDOW_BG`, `BUTTON_BG`...) to colours. `create_dark_theme()` is the built-in dark one.
- `@FeathersButton` is a scene (see [[Scenes and BSN]]) you compose with your own components.

## Tokens, not colours

Components say *what* they are (`ThemeBackgroundColor(tokens::WINDOW_BG)`), not which colour. Swap the `UiTheme` resource and every widget changes at once. It also lets themes change **per subtree** (contextual theming) so a warning panel can use a different palette.

## Make your own theme

```rust
use bevy::prelude::*;
use bevy::feathers::{dark_theme::create_dark_theme, theme::UiTheme};

fn main() {
    let mut theme = create_dark_theme();
    // ...override tokens here, e.g. the accent colour...
    App::new().insert_resource(UiTheme(theme));
}
```

Read the `feathers_gallery` example for every control and token.

## Your own design system

This repo's UI kit ([[UI kit]]) is a worked example of a custom design system on Bevy: **design tokens** for colour, spacing and type, and one file per component that follows shadcn/ui. Its README maps CSS-like classes to tokens. Copy the idea, not the code.

## Next

Interaction beyond buttons: [[Picking]].

---
icon: panel-left
tags: [rendering, beginner]
---
# Windows

The window is an entity too: it has a [`Window`] component you can read and change at any time. #rendering #beginner

## Set it up front

```rust
use bevy::{prelude::*, window::PresentMode};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "My game".into(),
                resolution: (1280, 720).into(),
                present_mode: PresentMode::AutoVsync,   // AutoNoVsync for uncapped
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .run();
}
```

## Change it while running

```rust
use bevy::{prelude::*, window::WindowMode};

fn toggle_fullscreen(keys: Res<ButtonInput<KeyCode>>, mut window: Single<&mut Window>) {
    if keys.just_pressed(KeyCode::F11) {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            _ => WindowMode::Windowed,
        };
    }
}
```

## The cursor

```rust
use bevy::{prelude::*, window::CursorOptions};

fn hide_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = false;
}
```

Grab mode (`CursorGrabMode::Locked`) is how first-person games capture the mouse. Where the mouse is: `window.cursor_position()` ([[Cameras]] has the conversion to world coordinates).

## Resizing and scale

`window.width()` and `window.height()` are in logical pixels; `window.scale_factor()` is the HiDPI factor. UI scales with `UiScale` ([[UI layout]]).

## Quit

```rust
use bevy::prelude::*;

fn quit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
```

## Several windows, no window

Spawn more `Window` entities for more windows (each camera picks one with `RenderTarget`). For servers and tools run without rendering: `MinimalPlugins` or `DefaultPlugins.set(WindowPlugin { primary_window: None, ..default() })` ([[Testing]]).

## On the web

The "window" is a `<canvas>`: `Window { canvas: Some("#game".into()), fit_canvas_to_parent: true, ..default() }`. See [[Web build]].

## Next

Pick your dimension: [[2D]] or [[3D]].

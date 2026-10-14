---
lib: true
icon: package
tags: [ui]
crates: bevy_hui={ version = "0.8", features = ["picking"] }
---
# UI crates

Two other ways to describe a UI: HTML-like files, and a layout engine for world-space interfaces. See [[UI]].

## bevy_hui

Build Bevy UI in **pseudo-HTML files**, reload them while the game runs, and bind logic from Rust by function name. Templates work like web components. #ui

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_hui 0.8` supports **Bevy 0.20**.

```toml
bevy_hui = { version = "0.8", features = ["picking"] }   # "picking" enables hover/press styles and events
```

### A template

`assets/my_button.html`:

```html
<template>
    <property name="text">Press me</property>
    <property name="action">greet</property>

    <node padding="10px">
        <button
            width="210px" height="80px"
            background="#123"
            border_radius="30px"
            hover:background="#503"
            hover:height="100px"
            ease="cubic_in" delay="0.2s"
            on_press="{action}">
            <text font_size="32">{text}</text>
        </button>
    </node>
</template>
```

- Values in `{braces}` are **properties** you can override where the template is used.
- `hover:` / `pressed:` prefixes give state styles and animated transitions.
- `on_press="greet"` calls a Rust function you registered under that name.

### Use it from Rust

Register the template as a component named after its tag, bind your functions by name, and spawn a root template:

```rust
use bevy::prelude::*;
use bevy_hui::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Play,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HuiPlugin))
        .init_state::<GameState>()
        .add_systems(Startup, (register, setup).chain())
        .run();
}

fn register(
    server: Res<AssetServer>,
    mut components: HtmlComponents,
    mut functions: HtmlFunctions,
) {
    // `<my_button>` can now be used inside other templates.
    components.register("my_button", server.load("my_button.html"));

    // on_press="start_game" calls this one-shot system on the clicked node.
    functions.register("start_game", |In(_node): In<Entity>, mut next: ResMut<NextState<GameState>>| {
        next.set(GameState::Play);
    });
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(HtmlNode(server.load("menu.html")));
}
```

`menu.html` then uses it like a tag: `<my_button text="Start" action="start_game"/>`.

### Why it is attractive

- **Designers can edit UI** without a rebuild (hot reload, see [[Hot reloading]]).
- Reusable **templates** with typed properties, recursively.
- It is plain Bevy UI underneath: [[UI layout]], [[Buttons and widgets]], [[Picking]] all still apply.
- No built-in look: you bring the design.

### Alternatives

| Crate | Approach |
|:--|:--|
| [[UI crates#bevy_lunex\|bevy_lunex]] | layout system for responsive, retained UI |
| `bsn!` ([[Scenes and BSN]]) | Rust macro scenes built into Bevy 0.20 |
| [[bevy_egui]] | immediate mode, best for tools |

### Links

[crates.io](https://crates.io/crates/bevy_hui) · [docs.rs](https://docs.rs/bevy_hui) · [repo](https://github.com/Lommix/bevy_hui)

## bevy_lunex

A **retained layout engine** for UI built from regular entities: any aspect ratio without distortion, a flexbox-like `flow` layout, **world-space UI** in 2D and 3D, interactivity through normal systems and observers. #ui

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): the `main` branch targets **Bevy 0.20**; the last crates.io release `0.7` targets 0.19. Depend on the git repository for 0.20.

```toml
bevy_lunex = { git = "https://github.com/bytestring-net/bevy_lunex" }
```

### When to use it

- Menus that must look right on any window shape (ultrawide, phone, 4:3).
- UI placed **in the 3D world** (a screen on a wall, a health bar over an enemy).
- You prefer a layout engine to CSS-like flex.

Bevy's own UI ([[UI layout]]) already does flex and grid and is the default; Lunex is the choice when its specific strengths matter. The README's `Bevypunk` demo shows a full game menu.

### Links

[crates.io](https://crates.io/crates/bevy_lunex) · [docs.rs](https://docs.rs/bevy_lunex) · [repo](https://github.com/bytestring-net/bevy_lunex)

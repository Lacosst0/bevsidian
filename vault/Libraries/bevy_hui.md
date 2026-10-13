---
icon: package
tags: [lib, ui]
crates: bevy_hui={ version = "0.8", features = ["picking"] }
---
# bevy_hui

Build Bevy UI in **pseudo-HTML files**, reload them while the game runs, and bind logic from Rust by function name. Templates work like web components. #lib #ui

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_hui 0.8` supports **Bevy 0.20**.

```toml
bevy_hui = { version = "0.8", features = ["picking"] }   # "picking" enables hover/press styles and events
```

## A template

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

## Use it from Rust

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

## Why it is attractive

- **Designers can edit UI** without a rebuild (hot reload, see [[Hot reloading]]).
- Reusable **templates** with typed properties, recursively.
- It is plain Bevy UI underneath: [[UI layout]], [[Buttons and widgets]], [[Picking]] all still apply.
- No built-in look: you bring the design.

## Alternatives

| Crate | Approach |
|:--|:--|
| [[bevy_lunex]] | layout system for responsive, retained UI |
| `bsn!` ([[Scenes and BSN]]) | Rust macro scenes built into Bevy 0.20 |
| [[bevy_egui]] | immediate mode, best for tools |

## Links

[crates.io](https://crates.io/crates/bevy_hui) · [docs.rs](https://docs.rs/bevy_hui) · [repo](https://github.com/Lommix/bevy_hui)

## Next

[[UI]], [[Libraries]].

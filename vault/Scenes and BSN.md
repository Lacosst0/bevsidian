---
icon: layout-template
---
# Scenes and BSN

**BSN**[^1] (*Bevy Scene Notation*) describes an entity, its components and its children in one place. Bevy 0.20 ships the `bsn!` macro for code-defined scenes. #scenes #ui

## The `bsn!` macro

Components are separated by whitespace, and children are separated by `--`:

```rust
use bevy::prelude::*;

fn menu() -> impl Scene {
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(8) }
        Children [
            Text("Play")
            --
            Text("Quit")
        ]
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(menu());
}
```

## Patching

Writing `Node { width: px(200) }` only sets `width`; every other field keeps the value it had from an earlier patch or the type's default. This makes scenes composable:

```rust
fn wide_menu() -> impl Scene {
    bsn! {
        @menu()
        Node { width: px(400) }
    }
}
```

## Names and references

`#Name` names an entity and lets other entities in the same scene refer to it, which is handy for relationships and [[ECS#Observers and events|observers]].

## Reactive scenes

Libraries such as `bevy_radioactive` add signals on top of BSN. Its `coat!` macro accepts reactive `if`, `for` and `match`, and Bevsidian's interface is built with it.

> [!IMPORTANT]
> The `.bsn` asset file format is still in progress; the macro form is stable for code-defined scenes.

See also [[Rendering#UI]] and [[ECS]].

[^1]: BSN is pronounced "bison".

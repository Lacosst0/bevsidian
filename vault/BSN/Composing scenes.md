---
parent: BSN syntax
icon: blocks
tags: [bsn]
---
# Composing scenes

BSN scenes are plain Rust values, so you compose them with plain Rust: functions, parameters, loops. This note is the toolbox. #bsn

## Patching, precisely

When you write `Button { width: 200 }` you do **not** insert a whole `Button`. You make a *patch* that sets one field. Unmentioned fields keep the value they had from an earlier patch, an included scene, or the type's default.

```rust
use bevy::prelude::*;

#[derive(Component, Clone)]
struct Button { width: u32, height: u32 }

impl Default for Button {
    fn default() -> Self { Button { width: 100, height: 300 } }
}

// All three end up as a 200 x 300 button:
fn a() -> impl Scene { bsn! { Button { width: 200, height: 300 } } }
fn b() -> impl Scene { bsn! { Button { width: 200 } } }
fn c() -> impl Scene {
    bsn! {
        Button                  // defaults: 100 x 300
        Button { width: 200 }   // width changes
        Button { height: 300 }  // height set again
    }
}
```

Plain Bevy replaces the whole component on `insert`. Patching merges instead, in the order written. The **last** patch wins on a field.

## Include a scene: `@`

`@name()` includes a scene function. Its patches apply *first*, at that position, then yours:

```rust preview=bsn-buttons
use bevy::prelude::*;

fn button() -> impl Scene {
    bsn! {
        Node { padding: UiRect::all(px(8)), border_radius: BorderRadius::all(px(6)) }
        BackgroundColor(Color::srgb(0.2, 0.2, 0.3))
    }
}

fn danger_button() -> impl Scene {
    bsn! {
        @button()
        BackgroundColor(Color::srgb(0.7, 0.1, 0.1))     // only the colour changes
    }
}

fn dialog() -> impl Scene {
    bsn! {
        Node { column_gap: px(8) }
        Children [
            @button() Children [ Text("Save") ]
            --
            @danger_button() Children [ Text("Delete") ]
        ]
    }
}
```

Includes nest: `danger_button` can be included by a `confirm_dialog`, which can be patched by a test.

## Parameters

A scene function can take anything: numbers, strings, entities, other scenes.

```rust
use bevy::prelude::*;

#[derive(Component, FromTemplate)]
struct Health { current: u32, max: u32 }

fn enemy(hp: u32, name: &str) -> impl Scene {
    let name = name.to_string();
    bsn! {
        Name({name})
        Health { current: {hp / 2}, max: hp }       // starts hurt
    }
}

fn pack() -> impl SceneList {
    bsn_list! {
        @enemy(30, "rat")
        --
        @enemy(200, "troll")
    }
}
```

`{ ... }` embeds any Rust expression where a value goes. Variables that are already the right type need no braces.

## Scenes as values

Because a scene is a value, it can be an argument. `@{ expr }` includes it:

```rust
use bevy::prelude::*;

/// A framed panel around whatever you give it.
fn panel(content: impl Scene) -> impl Scene {
    bsn! {
        Node { padding: UiRect::all(px(12)), border: UiRect::all(px(1)) }
        BorderColor::all(Color::WHITE)
        Children [ @{content} ]
    }
}

fn settings() -> impl Scene {
    bsn! { @{panel(bsn! { Text("Volume") })} }
}
```

A whole **list** can be passed the same way, and dropped into `Children [ ]` with braces:

```rust
use bevy::prelude::*;

fn column(items: impl SceneList) -> impl Scene {
    bsn! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            Text("Title")
            --
            {items}                 // zero or more entities here
            --
            Text("Footer")
        ]
    }
}

fn menu() -> impl Scene {
    bsn! { @{column(bsn_list! { Text("Play") -- Text("Quit") })} }
}
```

## Lists from data

Collect scenes into a `Vec`; a `Vec` of scenes is a `SceneList`.

```rust
use bevy::prelude::*;

fn scoreboard(scores: &[(&str, u32)]) -> impl Scene {
    let rows: Vec<_> = scores
        .iter()
        .map(|(name, score)| {
            let line = format!("{name}: {score}");
            bsn! { Text({line}) }
        })
        .collect();
    bsn! {
        Node { flex_direction: FlexDirection::Column }
        Children [ {rows} ]
    }
}
```

This is spawn-time. If the list must change while the app runs, use a `for` in a `view!` ([[bevy_radioactive]]).

## Conditions

There is **no `if` inside** `bsn!`. Decide before, then use the result:

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
struct Boss;

#[derive(Component, Default, Clone)]
struct Grunt;

// 1. choose a value
fn hp(is_boss: bool) -> u32 { if is_boss { 500 } else { 100 } }

// 2. choose a whole scene: a Box<dyn Scene> holds any
fn unit(is_boss: bool) -> impl Scene {
    let kind: Box<dyn Scene> = if is_boss {
        Box::new(bsn! { Boss })
    } else {
        Box::new(bsn! { Grunt })
    };
    bsn! { @{kind} }
}

// 3. maybe a scene: Option works too
fn maybe_shield(has_shield: bool) -> impl Scene {
    let shield = has_shield.then(|| Box::new(bsn! { Name("Shield") }) as Box<dyn Scene>);
    bsn! { @{shield} }
}
```

## Reusing a scene, cheaply

Resolving a scene costs work each time. For scene **assets** you opt in to caching with `:`; the cached version is patched copy-on-write:

```rust ignore
bsn! {
    :"enemy.bsn"        // cached; needs a loader for the format
    Health { max: 200 }
}
```

Caching is not available for scene *functions* yet, and the `.bsn` format does not ship yet.

## Keep the shape

A few habits keep scenes pleasant:

- One scene function = one *idea* (`button`, `health_bar`, `enemy`).
- Put the **defaults** in the base function, and **variants** as small patches on top.
- Pass things that vary as parameters, not options. Callers who want a different look just patch.
- Name roots with `#Name`: they show up in the inspector ([[Diagnostics and logging#Dev tools|Dev tools]]) and in `Query<&Name>`.

## Next

[[Templates and assets]]: why some components need `FromTemplate`.

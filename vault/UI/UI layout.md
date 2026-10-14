---
icon: panel-left
tags: [ui]
---
# UI layout

UI layout is **flexbox** and **grid**, like CSS. A [`Node`] describes the box; Bevy computes where everything goes. #ui

## Try it

Change the container's `flex-direction`, `justify-content` and `align-items` below. *Justify* works along the main axis (the direction of travel); *align* works across it.

```rust preview=flex
use bevy::prelude::*;

fn container() -> impl Bundle {
    (
        Node {
            height: px(150),
            padding: UiRect::all(px(8)),
            column_gap: px(8),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,   // main axis
            align_items: AlignItems::Start,           // cross axis
            ..default()
        },
        children![
            (Node { width: px(40), height: px(30), ..default() }, BackgroundColor(Color::srgb(0.95, 0.45, 0.4))),
            (Node { width: px(50), height: px(50), ..default() }, BackgroundColor(Color::srgb(0.98, 0.75, 0.3))),
            (Node { width: px(30), height: px(40), ..default() }, BackgroundColor(Color::srgb(0.35, 0.75, 0.9))),
        ],
    )
}
```

## Sizes: Val

| Write | Means |
|:--|:--|
| `px(200)` | 200 logical pixels |
| `percent(50)` | half of the parent |
| `auto()` | decide from content (or flex) |
| `vw(10)`, `vh(10)` | percent of the window width or height |
| `em(2)`, `rem(1.5)` | relative to font size |

```rust
use bevy::prelude::*;

fn sizes() -> impl Bundle {
    Node {
        width: percent(80),
        max_width: px(600),          // never wider than this
        min_height: px(100),
        aspect_ratio: Some(16.0 / 9.0),
        ..default()
    }
}
```

## Space around things

```rust
use bevy::prelude::*;

fn spacing() -> impl Bundle {
    Node {
        margin: UiRect::all(px(10)),               // outside the border
        padding: UiRect::axes(px(16), px(8)),      // inside, x then y
        border: UiRect::all(px(2)),
        column_gap: px(12),                        // between children
        row_gap: px(12),
        ..default()
    }
}
```

## A typical screen

Fill the window, stack things, centre the middle:

```rust
use bevy::prelude::*;

fn screen() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,   // top bar / content / footer
            ..default()
        },
        children![
            (Node { height: px(48), ..default() }, BackgroundColor(Color::srgb(0.15, 0.15, 0.2))),
            (Node { flex_grow: 1.0, ..default() }, BackgroundColor(Color::srgb(0.1, 0.1, 0.12))),   // takes the rest
            (Node { height: px(32), ..default() }, BackgroundColor(Color::srgb(0.15, 0.15, 0.2))),
        ],
    )
}
```

`flex_grow` is the trick for "fill what is left". `flex_shrink` and `flex_wrap: FlexWrap::Wrap` handle the other cases.

## Grow, shrink, wrap

Three numbers decide what happens when the children do not match the container. `flex_grow` says how much of the **spare** room a child takes (0 = none, 2 = twice the share of a 1). `flex_shrink` says how much it gives up when there is **too little** room. Wrapping starts a new line instead.

Set the middle box's `flex_grow`, then try `wrap`. With the container narrower than the boxes, the row either squeezes or spills onto a second line.

```rust preview=grow
use bevy::prelude::*;

fn row(grow: f32, wrap: FlexWrap) -> impl Bundle {
    (
        Node {
            column_gap: px(8),
            row_gap: px(8),
            flex_wrap: wrap,                 // Wrap or NoWrap
            ..default()
        },
        children![
            (Node { width: px(110), flex_shrink: 0.0, ..default() }, BackgroundColor(Color::srgb(0.95, 0.45, 0.4))),
            (Node { width: px(110), flex_grow: grow, ..default() }, BackgroundColor(Color::srgb(0.98, 0.75, 0.3))),
            (Node { width: px(110), flex_shrink: 0.0, ..default() }, BackgroundColor(Color::srgb(0.55, 0.85, 0.45))),
            (Node { width: px(110), flex_shrink: 0.0, ..default() }, BackgroundColor(Color::srgb(0.35, 0.75, 0.9))),
        ],
    )
}
```

The common pairs: `flex_grow: 1.0` on one child to push the rest to the edges (a toolbar with a spacer), and `flex_wrap: Wrap` for tag lists and inventories that must reflow.

## Grid

When things line up in **rows and columns** (an inventory, a gallery, a form), use a grid. Give it tracks, and children fill the cells in order. `RepeatedGridTrack::flex(4, 1.0)` means "four columns that share the width equally".

```rust preview=grid
use bevy::prelude::*;

fn inventory(columns: u16, gap: f32) -> impl Bundle {
    Node {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::flex(columns, 1.0),   // equal columns
        grid_auto_rows: vec![GridTrack::px(64.0)],                      // every new row is 64 tall
        column_gap: px(gap),
        row_gap: px(gap),
        ..default()
    }
}
```

Change the number of columns: the same eight cells reflow. A child can span several cells with `GridPlacement::span(2)` on its `grid_column`.

## Recipes

The layouts you will write over and over:

```rust
use bevy::prelude::*;

/// Centre one thing on the screen (a dialog, a loading spinner).
fn centred() -> impl Bundle {
    Node {
        width: percent(100),
        height: percent(100),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// A sidebar of fixed width, and content that takes the rest.
fn sidebar_layout() -> impl Bundle {
    (
        Node { width: percent(100), height: percent(100), ..default() },       // a row by default
        children![
            Node { width: px(240), flex_shrink: 0.0, ..default() },            // never squeezed
            Node { flex_grow: 1.0, ..default() },                              // the rest
        ],
    )
}

/// A full-screen dimmer that sits on top of everything and blocks clicks below it.
fn overlay() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        GlobalZIndex(100),
    )
}

/// A square of any width: the height follows.
fn square() -> impl Bundle {
    Node { width: percent(25), aspect_ratio: Some(1.0), ..default() }
}
```

`GlobalZIndex` sorts against the whole UI; the plain `ZIndex` only against siblings.

## Scrolling

A scroll area is a node with a fixed size, `Overflow::scroll_y()` and a `ScrollPosition`. The children simply overflow:

```rust
use bevy::prelude::*;

fn list(rows: usize) -> impl Scene {
    let rows: Vec<_> = (0..rows).map(|i| bsn! { Text({format!("Row {i}")}) }).collect();
    bsn! {
        Node {
            height: px(200),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),     // clip, and allow scrolling
        }
        ScrollPosition
        Children [ {rows} ]
    }
}
```

Mouse-wheel scrolling is yours to wire (a small observer on `PointerScroll` that changes `ScrollPosition`); this app's note view does exactly that.

## Layout in BSN

All of the above is the same in a [[Scenes and BSN|BSN]] scene, which reads better once the tree is deep, and lets you make each piece a function:

```rust
use bevy::prelude::*;

fn toolbar(title: &str) -> impl Scene {
    let title = title.to_string();
    bsn! {
        Node { height: px(48), padding: UiRect::horizontal(px(12)), align_items: AlignItems::Center, column_gap: px(8) }
        BackgroundColor(Color::srgb(0.15, 0.15, 0.2))
        Children [
            Text({title})
            --
            Node { flex_grow: 1.0 }                 // a spacer pushes the next one to the right
            --
            Text("Settings")
        ]
    }
}

fn screen() -> impl Scene {
    bsn! {
        Node { width: percent(100), height: percent(100), flex_direction: FlexDirection::Column }
        Children [
            @toolbar("My game")
            --
            Node { flex_grow: 1.0 }
        ]
    }
}
```

## Debugging layouts

When a box is not where you think, turn on Bevy's UI debug overlay: it outlines every node's border, padding and content boxes. Toggle it with a key:

```rust
use bevy::{prelude::*, ui_render::GlobalUiDebugOptions};

fn toggle_ui_debug(keys: Res<ButtonInput<KeyCode>>, mut options: ResMut<GlobalUiDebugOptions>) {
    if keys.just_pressed(KeyCode::F2) {
        options.toggle();
    }
}
```

Nine times in ten the cause is a missing `flex_shrink: 0.0`, a `percent` inside a parent with no height, or a forgotten `align_items`. Put a `UiDebugOptions` component on one node to override the global setting for just that node.

## Next

[[Text and images]].

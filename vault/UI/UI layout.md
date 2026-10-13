---
icon: panel-left
tags: [ui, beginner]
---
# UI layout

UI layout is **flexbox** and **grid**, like CSS. A [`Node`] describes the box; Bevy computes where everything goes. #ui #beginner

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

## Grid

```rust
use bevy::prelude::*;

fn inventory() -> impl Bundle {
    Node {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::flex(4, 1.0),   // 4 equal columns
        grid_auto_rows: vec![GridTrack::px(64.0)],
        column_gap: px(4),
        row_gap: px(4),
        ..default()
    }
}
```

## Put it anywhere: absolute

```rust
use bevy::prelude::*;

fn corner_badge() -> impl Bundle {
    Node {
        position_type: PositionType::Absolute,   // out of the flow
        top: px(12),
        right: px(12),
        ..default()
    }
}
```

## Show, hide, order

- `Display::None` removes a node from layout; `Visibility::Hidden` keeps its space.
- `ZIndex(5)` draws a node above its siblings.
- `Overflow::clip()` cuts off children; `Overflow::scroll_y()` with `ScrollPosition` makes a scroll area.

## Scale for every screen

`UiScale` multiplies all sizes; Bevy also scales by the window's DPI automatically. Use `percent`, `vw` and `rem` where you can, and `px` for fixed details like borders.

```rust
use bevy::prelude::*;

fn bigger(mut scale: ResMut<UiScale>) {
    scale.0 = 1.5;
}
```

## Next

[[Text and images]].

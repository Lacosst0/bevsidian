---
icon: layout-template
tags: [ui]
---
# UI

Menus, health bars, inventories, dialogs. Bevy's UI is made of ordinary entities: boxes laid out with flexbox or grid, with text and images inside. #ui

## Hello, UI

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);                       // UI needs a camera too
    commands.spawn((
        Node {                                      // a box
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Text::new("Hello, UI!"),                // text inside it
            TextFont { font_size: FontSize::Px(48.0), ..default() },
        )],
    ));
}
```

That is the whole model: a root [`Node`] filling the screen, children inside it.

## The notes

| Note | You learn |
|:--|:--|
| [[UI layout]] | size, flex, grid, margins, absolute position, scrolling |
| [[Text and images]] | fonts, spans, colours, images, borders |
| [[Buttons and widgets]] | buttons, checkboxes, sliders, hover and press |
| [[Text and images#Text input\|Text input]] | typing into a field |
| [[Theming]] | consistent colours with Feathers |
| [[Scenes and BSN]] | build UI trees compactly |
| [[Picking]] | clicks, hovers, dragging on anything |

## Mental model

- **Everything is an entity** with a [`Node`]. Parent/child gives the layout tree ([[Relationships and hierarchy]]).
- **Layout is CSS.** If you know flexbox, you know most of it.
- **Behaviour is observers.** A button's click is an `Activate` event ([[Events and observers]]).
- **State is components and resources.** Update a `Text` from a system when the data changes, or use a reactive layer ([[Building UI]]).

## Pick a style

- Small and simple: spawn nodes in code, as above.
- Bigger interfaces: write them with `bsn!` ([[Scenes and BSN]]).
- A ready look: the Feathers widgets ([[Theming]]) or [[bevy_egui]] for tools.
- Reactivity: `bevy_radioactive` (used by this app) lets nodes read **signals**.

## Libraries

Status for **Bevy 0.20**, checked 2026-10-13: **released** (a crates.io release uses 0.20) · **git** (the `main` branch is on 0.20; use a git dependency) · **pending** (the latest release targets an older Bevy) · **built in** (Bevy has it now). Check crates.io before you start.

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_egui]] | immediate-mode UI for tools | released `0.43` |
| [[UI crates#bevy_hui\|bevy_hui]] | UI in HTML-like files | released `0.8` |
| [[UI crates#bevy_lunex\|bevy_lunex]] | retained layout engine, world-space UI | git |
| [[bevy_radioactive]] | reactive BSN: `watch`, `subscribe`, `hooks`, `view!` | git |

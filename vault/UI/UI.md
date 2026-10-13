---
icon: layout-template
tags: [ui, beginner]
---
# UI

Menus, health bars, inventories, dialogs. Bevy's UI is made of ordinary entities: boxes laid out with flexbox or grid, with text and images inside. #ui #beginner

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
| [[Text input]] | typing into a field |
| [[Theming]] | consistent colours with Feathers |
| [[Scenes and BSN]] | build UI trees compactly |
| [[UI kit]] | the components of **this** app, as a worked example |
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

[[bevy_egui]] (immediate-mode tools, inspectors), [[bevy_lunex]] (layouts), [[bevy_cobweb_ui]], [[bevy_hui]] (UI from HTML-like files), [[bevy_simple_text_input]].

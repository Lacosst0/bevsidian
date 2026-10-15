---
icon: mouse-pointer
tags: [ui]
---
# Buttons and widgets

Bevy's widgets are **headless**: they give you behaviour (clicks, values, keyboard focus) and you draw them however you like. #ui

## A button

Three parts:

1. [`Button`] (from `bevy::ui_widgets`) marks the node as a button and handles press, release and keyboard.
2. [`Hovered`] tracks the pointer being over it.
3. An observer on [`Activate`] runs when the player really clicked (pressed *and* released on it, or pressed Enter/Space while focused).

```rust
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::{observe, Activate, Button},
};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node { width: percent(100), height: percent(100), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        children![(
            Button,
            Hovered::default(),
            Node { width: px(150), height: px(65), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::MAX, ..default() },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            children![Text::new("Play")],
            observe(|_: On<Activate>| info!("clicked!")),
        )],
    ));
}

// The look is yours: react to the state components.
fn colour_buttons(
    mut buttons: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<Button>>,
) {
    for (hovered, pressed, mut background) in &mut buttons {
        background.0 = match (pressed, hovered.get()) {
            (true, _) => Color::srgb(0.35, 0.75, 0.35),
            (false, true) => Color::srgb(0.25, 0.25, 0.25),
            (false, false) => Color::srgb(0.15, 0.15, 0.15),
        };
    }
}
```

> [!NOTE]
> Old code used `Interaction` and `Changed<Interaction>`, now deprecated: use `Hovered` and the `Pressed` marker as above.

## Handle clicks with an observer

`Activate` is an [[Events and observers|entity event]]: it fires on the button. Attach an observer **to the button** (`observe(...)` above, `on(...)` in BSN) or listen globally and check `event.entity`:

```rust
use bevy::{prelude::*, ui_widgets::Activate};

#[derive(Component)] struct Quit;

fn on_click(activate: On<Activate>, quit: Query<(), With<Quit>>, mut exit: MessageWriter<AppExit>) {
    if quit.contains(activate.entity) {
        exit.write(AppExit::Success);
    }
}
```

## Press events and raw pointer events

For custom behaviour, every node also receives picking events (`PointerPress`, `PointerOver`, `PointerClick`, `PointerDrag`...), see [[Picking]].

## More headless widgets

| Widget | Components | You get |
|:--|:--|:--|
| Checkbox | `Checkbox`, `Checked` | `ValueChange<bool>` |
| Radio group | `RadioGroup`, `RadioButton` | `ValueChange<Entity>` |
| Slider | `Slider`, `SliderValue`, `SliderRange` + a thumb | `ValueChange<f32>` |
| Scrollbar | `Scrollbar`, `ScrollbarThumb` | scrolling |
| Menu, tabs | `MenuPopup`, headless tabs | selection |
| Text input | `TextInput` | see [[Text and images#Text input\|Text input]] |

```rust
use bevy::{
    prelude::*,
    ui_widgets::{checkbox_self_update, observe, Checkbox, Slider, SliderRange, SliderValue, ValueChange},
};

fn controls() -> impl Bundle {
    (
        Node { flex_direction: FlexDirection::Column, row_gap: px(10), ..default() },
        children![
            (
                Checkbox,                                    // add `Checked` to start checked
                Node { width: px(24), height: px(24), border: UiRect::all(px(2)), ..default() },
                observe(checkbox_self_update),               // toggles itself when clicked
            ),
            (
                Slider::default(),
                SliderValue(50.0),
                SliderRange::new(0.0, 100.0),
                Node { width: px(200), height: px(12), ..default() },
                observe(|change: On<ValueChange<f32>>| info!("volume {}", change.value)),
            ),
        ],
    )
}
```

Checkbox and slider need a `SliderThumb` child (or checked-state styling) to look like anything; see Bevy's `standard_widgets_observers` example for the styling observers.

## Disabling, focus, keyboard

- `InteractionDisabled` on a widget turns it off.
- `TabNavigationPlugin` + `TabIndex(n)` + `TabGroup` give Tab/Shift-Tab focus; `InputFocus` stores which entity has it.
- Directional (gamepad/arrow) navigation: the `directional_navigation` example.

## Ready-made looks

Wiring the colours of every state yourself is tedious. Two ways out:

- **Feathers**, Bevy's own styled widget set: [[Theming]].
- **Your own** small set of component functions that wrap these widgets, with colours from one place.

## Next

[[Text and images#Text input|Text input]].

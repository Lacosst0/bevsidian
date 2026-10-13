---
icon: package
tags: [lib, ui]
check: skip
---
# bevy_simple_text_input

A small single-line text field for Bevy UI: placeholder, password masking, max length, submit event. #lib #ui

> [!WARNING]
> **Deprecated.** Its author recommends Bevy's built-in `EditableText` + `TextInput` ([[Text input]]) instead, which is better and has more features, and plans to archive the crate with Bevy 0.20. The latest release `0.15` targets Bevy 0.19.

## What to use now

```rust ignore
// Bevy 0.20: no dependency needed.
use bevy::{prelude::*, text::EditableText, ui_widgets::TextInput};

fn field() -> impl Bundle {
    (
        Node { width: px(300), padding: UiRect::all(px(8)), ..default() },
        EditableText::default(),
        TextInput,
    )
}
```

See [[Text input]] for focus, tabbing and reading the value.

## Links

[crates.io](https://crates.io/crates/bevy_simple_text_input) · [repo](https://github.com/rparrett/bevy_simple_text_input)

## Next

[[Text input]], [[Libraries]].

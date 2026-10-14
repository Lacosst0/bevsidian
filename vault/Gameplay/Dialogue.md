---
icon: message-circle
tags: [gameplay, ui, intermediate]
crates: serde={ version = "1", features = ["derive"] }
---
# Dialogue

Conversations are a **graph of lines**. A line says something, then either leads to the next line or offers choices. Model that as data, write a small runner, and the UI is just a view of it. #gameplay #ui

## The data

```rust
use std::collections::HashMap;

use bevy::prelude::*;

/// One screen of talk.
struct Line {
    speaker: &'static str,
    text: &'static str,
    next: Next,
}

enum Next {
    /// Continue to this line.
    Go(&'static str),
    /// Ask the player: (button text, line to go to).
    Choose(Vec<(&'static str, &'static str)>),
    /// The end.
    End,
}

#[derive(Resource)]
struct Script(HashMap<&'static str, Line>);

fn innkeeper() -> Script {
    Script(HashMap::from([
        ("hello", Line {
            speaker: "Innkeeper",
            text: "Welcome, traveller. Looking for a room?",
            next: Next::Choose(vec![("Yes please", "room"), ("Just passing", "bye")]),
        }),
        ("room", Line {
            speaker: "Innkeeper",
            text: "Ten coins a night. Breakfast is free.",
            next: Next::Go("bye"),
        }),
        ("bye", Line { speaker: "Innkeeper", text: "Safe travels!", next: Next::End }),
    ]))
}
```

In a real game this table is a file (RON, JSON, or Yarn, below) so writers can edit it without Rust.

## The runner

State is one thing: **which line we are on**. Advancing is a function of the player's input:

```rust
use bevy::prelude::*;

#[derive(Default, Resource)]
struct Conversation {
    /// The id of the current line, if a conversation is running.
    at: Option<&'static str>,
    shown: usize,        // characters revealed so far
    timer: Timer,
}

#[derive(Message)]
struct Advance(Option<usize>);        // Some(i) = chose option i

/// Reveals the text one letter at a time.
fn typewriter(time: Res<Time>, mut talk: ResMut<Conversation>) {
    if talk.at.is_none() {
        return;
    }
    if talk.timer.tick(time.delta()).just_finished() {
        talk.shown += 1;
    }
}
```

Set `timer` to `Timer::from_seconds(0.03, TimerMode::Repeating)` when a line starts. The player pressing a key while text is still revealing should **show the whole line first**, and only advance on the next press. It feels respectful of reading speed.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct DialogueText;

#[derive(Default, Resource)]
struct Conversation { shown: usize }

fn show_line(talk: Res<Conversation>, mut text: Single<&mut Text, With<DialogueText>>) {
    const LINE: &str = "Welcome, traveller. Looking for a room?";
    // char_indices keeps multi-byte letters whole
    let end = LINE.char_indices().nth(talk.shown).map_or(LINE.len(), |(i, _)| i);
    text.0 = LINE[..end].to_string();
}
```

## Choices are buttons

When a line is `Next::Choose`, spawn one button per option; each button carries the *id of its target*. Clicking sends the message the runner listens for:

```rust
use bevy::{prelude::*, ui_widgets::Activate};

#[derive(Message)]
struct Goto(&'static str);

fn choice(label: &'static str, target: &'static str) -> impl Scene {
    bsn! {
        bevy::ui_widgets::Button
        Node { padding: UiRect::all(px(8)) }
        on(move |_: On<Activate>, mut goto: MessageWriter<Goto>| { goto.write(Goto(target)); })
        Children [ Text({label}) ]
    }
}
```

## Make it matter

Talking is only worth it if it **changes the game**. Add conditions and effects to lines:

- A line has an optional `if` (checks a flag or an item: [[Inventory and items]]) and `then` (sets a flag, gives gold, starts a quest).
- Keep the flags in one `Resource` of `HashSet<String>`; [[Saving and loading]] stores them.
- Fire a **message** from the dialogue, never reach into the world from it. The shop opens because a `OpenShop` message arrived, not because the dialogue knows what a shop is.

## Writing tools

For a big script, use a language made for it. **Yarn Spinner** is a dialogue language with a Bevy plugin, `bevy_yarnspinner` 0.9 (pending on Bevy 0.19 as of 2026-10-13). Others: Ink (via a runtime), or your own RON.

```
title: hello
---
Innkeeper: Welcome, traveller. Looking for a room?
-> Yes please
    <<jump room>>
-> Just passing
    <<jump bye>>
===
```

## Next

[[Game AI]] for what the innkeeper does when not talking, [[Text and images]] for a speech bubble.

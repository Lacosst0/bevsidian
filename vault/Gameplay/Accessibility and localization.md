---
icon: accessibility
tags: [gameplay, ui]
---
# Accessibility and localization

Two ways to let more people play: make the game usable by people with different abilities, and make it readable in their language. Both are far cheaper if you plan them before the first menu is built. #ui

## Accessibility settings that cost little

Most of it is **choices the player can change** and **not relying on one sense**.

| Do | Why |
|:--|:--|
| text size setting | `UiScale` scales the whole UI |
| never colour alone | add a shape, icon or label to every colour signal (red/green is the common clash) |
| high-contrast text | body text 4.5:1 against its background or better |
| remappable keys | [[Input mapping]]; some players cannot use two hands |
| hold *or* toggle | for sprint, aim and other held buttons |
| reduce motion | turn off screen shake and flashes ([[Game feel]]) |
| subtitles | for every spoken line, and for important sounds |
| pausing in any menu | and no timers a player cannot stop |

Keep them in one resource, loaded and saved with the other settings ([[Saving and loading]]):

```rust
use bevy::prelude::*;

#[derive(Resource, Clone)]
struct Accessibility {
    ui_scale: f32,
    reduce_motion: bool,
    subtitles: bool,
}

impl Default for Accessibility {
    fn default() -> Self {
        Self { ui_scale: 1.0, reduce_motion: false, subtitles: true }
    }
}

fn apply(settings: Res<Accessibility>, mut scale: ResMut<UiScale>) {
    if settings.is_changed() {
        scale.0 = settings.ui_scale;
    }
}

/// Anything that shakes or flashes asks first.
fn shake_power(settings: &Accessibility, wanted: f32) -> f32 {
    if settings.reduce_motion { 0.0 } else { wanted }
}
```

## Screen readers and the keyboard

Bevy talks to the operating system's accessibility layer through **AccessKit**. It is on by default: UI `Text`, buttons and the standard widgets publish an `AccessibilityNode`, so a screen reader can find and read them. Give it better information by labelling things:

- Mark text that names something with `Label`, so it is read as that thing's name.
- Make every interactive element reachable by keyboard with `TabIndex` (and `TabGroup` for a menu). Test your menus with only Tab and Enter.
- Gamepads and arrow keys can move focus with directional navigation.

```rust
use bevy::{input_focus::tab_navigation::{TabGroup, TabIndex}, prelude::*, ui_widgets::Button};

fn menu() -> impl Scene {
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(8) }
        TabGroup
        Children [
            Button TabIndex(0) Children [ Text("Play") ]
            --
            Button TabIndex(1) Children [ Text("Options") ]
            --
            Button TabIndex(2) Children [ Text("Quit") ]
        ]
    }
}
```

## Localization

A localized game never has a string in its code. Every visible text has a **key** and the real text lives in a file per language.

```text
assets/lang/en.ron      ("menu.play": "Play", "hello": "Hello, {name}!")
assets/lang/pl.ron      ("menu.play": "Graj", "hello": "Cześć, {name}!")
```

The smallest working design: a `Locale` resource with the tables, a `t` lookup, and a component that marks text to translate.

```rust
use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Resource)]
struct Locale {
    lang: String,
    tables: HashMap<String, HashMap<String, String>>,
}

impl Locale {
    /// The text for `key`, in the current language, then English, then the key itself.
    fn t(&self, key: &str) -> String {
        self.tables
            .get(&self.lang)
            .and_then(|table| table.get(key))
            .or_else(|| self.tables.get("en").and_then(|table| table.get(key)))
            .cloned()
            .unwrap_or_else(|| key.to_owned())
    }

    /// `{name}` placeholders filled in.
    fn format(&self, key: &str, args: &[(&str, &str)]) -> String {
        args.iter()
            .fold(self.t(key), |text, (name, value)| text.replace(&format!("{{{name}}}"), value))
    }
}

/// Put on a `Text`: its content is looked up by this key.
#[derive(Component)]
struct Translate(&'static str);

fn translate(locale: Res<Locale>, mut texts: Query<(&Translate, &mut Text)>) {
    if locale.is_changed() {
        for (key, mut text) in &mut texts {
            text.0 = locale.t(key.0);
        }
    }
}
```

Switching language is `locale.lang = "pl".into()`; every `Translate` text updates in the same frame. In BSN:

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
struct Translate(String);

fn play_button() -> impl Scene {
    bsn! {
        bevy::ui_widgets::Button
        Children [ Text("") Translate("menu.play") ]
    }
}
```

### What goes wrong

- **Plurals and grammar.** "1 coin, 2 coins" is not `n + " coins"`: Polish and Arabic have more than two forms. A real library handles this.
- **Text grows.** German is often 30% longer than English. Never hard-code a button width; let [[UI layout]] size it.
- **Fonts.** Your font must contain the letters. Chinese, Japanese, Korean, Arabic and Thai need extra fonts; list fallbacks in `FontSource::families("Inter, Noto Sans CJK, sans-serif")`.
- **Right-to-left** languages flip layout as well as text. Plan for it if you ship Arabic or Hebrew.
- **Numbers, dates, currency.** Format with the locale, not `to_string`.
- **Text in images.** Impossible to translate. Draw the picture, put the words in text.

### Libraries

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| `bevy_fluent` 0.15 | Mozilla's Fluent files (plurals, genders) as assets | pending on 0.19 |
| `fluent-templates` | Fluent without a Bevy plugin | works today |

Status checked 2026-10-13.

## Next

[[Game feel]] has the reduce-motion switch's other half, [[Theming]] covers high-contrast palettes.

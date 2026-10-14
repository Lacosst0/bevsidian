---
icon: atom
tags: [ui, bsn]
crates: bevy_radioactive={ git = "https://github.com/Lacosst0/bevy_radioactive", default-features = false, features = ["watch", "subscribe", "hooks", "view", "ui"] }
---
# bevy_radioactive

Reactive UI for [[Scenes and BSN|BSN]]. A node says what it depends on **where it is spawned**, instead of a global system finding it by a marker. This app's own interface is written with it. #ui #bsn

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_radioactive 0.5` targets **Bevy 0.20** and is used from git (`Lacosst0/bevy_radioactive`). Needs Rust 1.97.1 or newer.

## Why

Plain Bevy keeps UI in sync with data using *systems*. That works, but each piece of UI needs a marker component and a system far away from the scene that spawns it:

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
struct Count(u32);

#[derive(Component)]
struct CountLabel;

// Far from the scene that spawns the label:
fn show_count(count: Res<Count>, mut labels: Query<&mut Text, With<CountLabel>>) {
    if count.is_changed() {
        for mut text in &mut labels {
            text.0 = count.0.to_string();
        }
    }
}
```

`bevy_radioactive` moves that wiring *into* the scene. There are four styles. They share no code, so use one, or several side by side. Each is a Cargo feature.

| Style | Feature | Reacts to | Reach for it when |
|:--|:--|:--|:--|
| `watch` | `watch` | a component or resource changing | a label should follow a value |
| `subscribe` | `subscribe` | an entity event fired elsewhere | one node reacts to another's click |
| `hooks` | `hooks` | local state, like React | a widget owns its own state |
| `view` | `view` | signals, `if`, `for`, `match` | UI that appears, disappears and repeats |

## Try them

Below is a plain Bevy app: a resource, a button that adds one. **Pick** the styles to add. The new lines are highlighted, and each choice is one compiled program.

```rust pick
use bevy::{prelude::*, ui_widgets::Activate};
//@ anchor imports

#[derive(Resource, Clone, Default)]
struct Count(u32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Count>()
        .add_systems(Startup, setup)
        //@ anchor plugins
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(screen());
}

fn screen() -> impl Scene {
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(8), padding: UiRect::all(px(16)) }
        Children [
            #Plus bevy::ui_widgets::Button
            Node { padding: UiRect::all(px(8)) }
            on(|_: On<Activate>, mut count: ResMut<Count>| count.0 += 1)
            Children [ Text("+1") ]
            //@ anchor children
        ]
    }
}
//@ anchor items

//@ option watch watch: a label follows the resource
//@ at imports
use bevy_radioactive::watch::*;
//@ at plugins
.add_plugins(WatchPlugin)
//@ at children
--
Text("0")
bind(Singleton, |count: &Count, text: &mut Text| text.0 = count.0.to_string())

//@ option subscribe subscribe: a label hears the button
//@ at imports
use bevy_radioactive::subscribe::*;
//@ at children
--
Text("Not pressed yet")
subscribe_to(#Plus, |click: Subscription<Activate>, mut texts: Query<&mut Text>| {
    if let Ok(mut text) = texts.get_mut(click.target()) {
        text.0 = "Pressed!".into();
    }
})

//@ option hooks hooks: a counter that owns its state
//@ at imports
use bevy_radioactive::hooks::*;
//@ at plugins
.add_plugins(HooksPlugin)
//@ at children
--
@counter(10)
//@ at items
#[component]
fn counter(cx: &Cx, start: i32) -> impl Scene {
    let (count, set_count) = cx.use_state(|| start);
    bsn! {
        bevy::ui_widgets::Button
        on(move |_: On<Activate>| set_count(count + 1))
        Children [ Text({format!("clicked {count}")}) ]
    }
}

//@ option view view: a section that opens and closes
//@ at imports
use bevy_radioactive::view::*;
//@ at plugins
.add_plugins(ViewPlugin)
//@ at children
--
@details()
//@ at items
fn details() -> impl Scene {
    let open = signal(|| false);
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            bevy::ui_widgets::Button
            on(move |_: On<Activate>| open.set(!$open))
            Children [ Text($(if $open { "Hide" } else { "Show" })) ]
            --
            if $open {
                Text("Only here while open")
            }
        ]
    }
}
```

Everything above runs in one window. Read the highlighted lines: each style adds **one import, one plugin and a few lines inside the scene**. Nothing else in the app changes.

## `watch`

`bind` writes a component of *this* entity whenever a value on a source changes. A source is a `#Name`, an `Entity`, a list like `[#A, #B]`, or `Singleton` for a resource (resources are components in 0.20).

| Function | Runs when | Does |
|:--|:--|:--|
| `bind(source, \|&T, &mut C\| ..)` | `T` is inserted or changed | writes `C` here |
| `bind_or(source, write, remove)` | same, plus removal or despawn | writes `C` both ways |
| `watch(source, handler)` | `T` is inserted or changed | runs an observer taking `Reaction<T>` |
| `watch_removed(source, handler)` | `T` is removed | observer taking `Removal<T>` |

```rust
use bevy::prelude::*;
use bevy_radioactive::watch::*;

#[derive(Component, Clone, Default)]
struct Health(u32);

fn hud() -> impl Scene {
    bsn! {
        Node
        Children [
            #Player Health(100)
            --
            Text("")
            bind_or(
                #Player,
                |hp: &Health, text: &mut Text| text.0 = format!("HP {}", hp.0),
                |text: &mut Text| text.0 = "DEAD".into(),
            )
        ]
    }
}
```

Good to know:

- Several writes in one frame arrive **once**, with the latest value.
- The *current* value is not replayed when a scene spawns. Values the same scene inserts are, so they show on spawn.
- Plug in `WatchPlugin`. For global observers use `app.track::<T>()` and observe `ValueChanged<T>`.

## `subscribe`

BSN's `on(...)` hears events on the entity that declares it. `subscribe_to(#Other, ..)` hears events fired on **another** entity; `bind_event` writes a component of the declaring entity in response. No plugin needed.

```rust
use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::subscribe::*;

#[derive(Component, Clone, Default)]
struct Clicks(u32);

fn toolbar() -> impl Scene {
    bsn! {
        Node
        Children [
            #Save bevy::ui_widgets::Button
            --
            Clicks(0)
            bind_event(#Save, |_: &Activate, clicks: &mut Clicks| clicks.0 += 1)
        ]
    }
}
```

`Subscription<E>` dereferences to the event; `target()` is the node that subscribed and `source()` the one the event fired on. A subscription goes away with the entity that made it.

## `hooks`

If you know React: a function marked `#[component]` is a component, `cx.use_state` is `useState`. The scene re-renders when its state or the ECS data it read changes. Use it like a scene function: `@counter(5)`.

| React | `hooks` |
|:--|:--|
| `<Counter start={5} />` | `@counter(5)` |
| `useState` / `useReducer` | `cx.use_state(init)` / `cx.use_reducer(init, f)` |
| `useMemo` / `useEffect` / `useRef` | `cx.use_memo` / `cx.use_effect` / `cx.use_ref` |
| context | `cx.provide_context(v)` / `cx.use_context::<T>()` |
| store selectors | `cx.use_resource::<R>()` / `cx.use_component::<C>(entity)` |

```rust
use bevy::prelude::*;
use bevy_radioactive::hooks::*;

#[derive(Component, Clone)]
struct Health(u32);

#[component]
fn health_bar(cx: &Cx, player: Entity) -> impl Scene {
    let hp = cx.use_component::<Health>(player).map_or(0, |hp| hp.0);
    bsn! { Text({format!("HP {hp}")}) }
}
```

> [!WARNING]
> A render **respawns the subtree**. Keep components small, call hooks in the same order every render, and give list rows a key.

## `view`

`view!` is `bsn!` with signals. `$name` reads one. Anything that reads a signal re-runs by itself: a field is patched in place, an `if` or `for` rebuilds only its own region.

| In a list | Meaning |
|:--|:--|
| `if $c { .. } else { .. }` | entities that come and go |
| `#[key(expr)] for x in $items { .. }` | a row per item; unchanged rows keep their entities |
| `match $v { .. }` | the arm that matches |
| `let x = expr;` | a binding for what follows |

Signals have `get`, `set`, `update`, `set_if_neq` and `with`. `memo(|| ..)` caches something derived, and `effect(|commands| ..)` re-runs a side effect. Signals are `Send + Sync`, so a system can set one that lives in a component or resource.

```rust
use bevy::prelude::*;
use bevy_radioactive::view::*;

#[derive(Clone, PartialEq)]
struct Todo { id: u32, title: &'static str }

fn todo_list() -> impl Scene {
    let todos = signal(|| vec![Todo { id: 1, title: "Write docs" }]);
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            Text($(format!("{} todos", $todos.len())))
            --
            #[key(todo.id)]
            for todo in $todos {
                Text({todo.title})
            }
        ]
    }
}
```

## Which one?

- A label follows a **resource or another entity's component**: `watch`.
- Something reacts to **another node's event**: `subscribe`.
- A widget has **private state** and you like React: `hooks`.
- UI **appears, disappears or repeats**, or you want the smallest updates: `view`.

They mix freely. This app uses `view!` for its sidebar, dialogs and settings.

## Next

[[Scenes and BSN]] for the syntax it builds on, [[Buttons and widgets]] for what to put inside, or the rest of [[UI]].

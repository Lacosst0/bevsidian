---
lib: true
icon: atom
tags: [ui, bsn]
crates: bevy_radioactive={ git = "https://github.com/Lacosst0/bevy_radioactive", branch = "master", default-features = false, features = ["watch", "subscribe", "hooks", "view", "ui"] }
---
# bevy_radioactive

Reactive UI for [[Scenes and BSN|BSN]]. A node says what it depends on **where it is spawned**, instead of a global system finding it by a marker. This app's own interface is written with it. #ui #bsn

#0.20 · version 0.6, from git (`Lacosst0/bevy_radioactive`, branch `master`) · Rust 1.97.1 or newer

```toml
bevy_radioactive = { git = "https://github.com/Lacosst0/bevy_radioactive", branch = "master" }
```

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
| `watch` | `watch` | whatever an ordinary system reads | a label should follow a value |
| `subscribe` | `subscribe` | an entity event fired elsewhere | one node reacts to another's click |
| `hooks` | `hooks` | local state, like React | a widget owns its own state |
| `view` | `view` | typed `Watch` parameters, with `if`, `for`, `match` | UI that appears, disappears and repeats |

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
watch(|count: Res<Count>, mut text: This<&mut Text>| {
    if let Some(mut text) = text.get_mut() {
        text.0 = count.0.to_string();
    }
})

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
use bevy_radioactive::hooks::{Cx, HooksPlugin};
//@ at plugins
.add_plugins(HooksPlugin)
//@ at children
--
@counter(10)
//@ at items
#[bevy_radioactive::hooks::component]
fn counter(cx: &Cx, start: i32) -> impl Scene {
    let (count, set_count) = cx.use_state(|| start);
    bsn! {
        bevy::ui_widgets::Button
        on(move |_: On<Activate>| set_count(count + 1))
        Children [ Text({format!("clicked {count}")}) ]
    }
}

//@ option view view: a section that opens and follows the count
//@ at imports
use bevy_radioactive::view::{ViewPlugin, Watch, view};
//@ at plugins
.add_plugins(ViewPlugin)
//@ at children
--
@details()
//@ at items
#[bevy_radioactive::view::component]
fn details(count: Watch<Res<Count>>, open: Watch<Local<bool>>) -> impl Scene {
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            bevy::ui_widgets::Button
            on(move |_: On<Activate>| open.set(!$open))
            Children [ Text($(format!("{}", if $open { "Hide" } else { "Show" }))) ]
            --
            if $open {
                Text($(format!("The count is {}", $count.0)))
            }
        ]
    }
}
```

Everything above runs in one window. Each style adds **one import, one plugin and a few lines inside the scene**; nothing else in the app changes.

## `watch`

`watch` takes an **ordinary Bevy system** and attaches it to the entity that declares it. The system runs once when the scene spawns, then again whenever something it reads changes. There are no sources to name: the dependencies come from its parameters. `This<D>` queries the entity that declared the watch.

| The system reads | It re-runs when |
|:--|:--|
| `Res<T>` | `T` is inserted or changed |
| `Option<Res<T>>` | same, or `T` is removed |
| `Query<&T>`, `Option<&T>`, `Has<T>` | `T` is inserted, changed or removed on any entity |
| `With<T>`, `Without<T>`, `Changed<T>` | same, for the filtered `T` |
| `ResMut<T>`, `&mut T`, `Commands`, `This<..>` | never: what a watch writes isn't a dependency |

```rust
use bevy::prelude::*;
use bevy_radioactive::watch::*;

#[derive(Component, Clone, Default)]
struct Player;

#[derive(Component, Clone, Default)]
struct Health(u32);

fn hud() -> impl Scene {
    bsn! {
        Node
        Children [
            Player Health(100)
            --
            Text("")
            watch(|player: Query<&Health, With<Player>>, mut text: This<&mut Text>| {
                if let Some(mut text) = text.get_mut() {
                    text.0 = match player.single() {
                        Ok(hp) => format!("HP {}", hp.0),
                        Err(_) => "DEAD".into(),
                    };
                }
            })
        ]
    }
}
```

> [!TIP]
> Several writes in one frame re-run a watch once. Use `set_if_neq` to skip writes that change nothing, and `Option<Res<T>>` to draw before a resource exists. A watch that reads `Res<Time>` runs every frame: keep those for plain systems.

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

`Subscription<E>` dereferences to the event; `target()` is the node that subscribed and `source()` the one the event fired on. A source is a `#Name`, an `Entity` or a list like `[#A, #B]`. A subscription goes away with the entity that made it.

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

`view!` is `bsn!` plus reactive `if`, `for` and `match`. A `#[component]` function declares what its template reacts to **in the types of its parameters**, and the runtime supplies them:

| Parameter | Value |
|:--|:--|
| `Watch<Local<T>>` | state of this instance, from `T::default()`; `set`, `update`, `set_if_neq` |
| `#[init(expr)] x: Watch<Local<T>>` | the same, starting at `expr` |
| `Watch<Res<T>>` | resource `T`, read-only; follows its changes |
| `Watch<Option<Res<T>>>` | resource `T`, or `None` while it doesn't exist |
| anything else | a prop: `@row(todo)` passes `todo` |

`$name` reads a watch. A field that reads one is patched in place; an `if`, `for` or `match` that reads one rebuilds only its own region.

```rust
use bevy::prelude::*;
use bevy_radioactive::view::*;

#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: &'static str,
}

#[derive(Resource, Clone, Default)]
struct Todos(Vec<Todo>);

#[component]
fn todo_list(todos: Watch<Res<Todos>>, #[init(true)] open: Watch<Local<bool>>) -> impl Scene {
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            bevy::ui_widgets::Button
            on(move |_: On<PointerClick>| open.set(!$open))
            --
            Text($(format!("{} todos", $todos.0.len())))
            --
            if $open {
                #[key(todo.id)]
                for todo in $todos.0 { Text({todo.title}) }
            }
        ]
    }
}
```

| In a list | Meaning |
|:--|:--|
| `if $c { .. } else { .. }` | entities that come and go |
| `#[key(expr)] for x in $items { .. }` | a row per item; unchanged rows keep their entities |
| `match $v { .. }` | the arm that matches |
| `let x = expr;` | a binding for what follows |

> [!NOTE]
> Only `Local` state is written from the view. Change resources the usual way (`ResMut`, in systems or handlers) and every view watching them follows. Read a resource watch with `$` in fields, regions and handlers, not in a `{ .. }` value: those run while the scene is built, before the watch is bound.

## Which one?

- A label follows a **resource or another entity's component**: `watch`.
- Something reacts to **another node's event**: `subscribe`.
- A widget has **private state** and you like React: `hooks`.
- UI **appears, disappears or repeats**, or you want the smallest updates: `view`.

They mix freely. This app uses `view!` for its sidebar, dialogs and settings.

## Upgrading from 0.5

| 0.5 | 0.6 |
|:--|:--|
| `bind(Singleton, \|t: &T, c: &mut C\| ..)` | `watch(\|t: Res<T>, mut c: This<&mut C>\| ..)` |
| `bind(#Source, ..)`, `watch(#Source, ..)` | `watch` with a `Query` and a marker component |
| `watch_removed`, `bind_or` | one `watch`; check whether the query still matches |
| `app.track::<T>()`, `ValueChanged<T>` | nothing to register; a system with `Changed<T>` |
| `let x = signal(\|\| v)` in a scene function | `#[init(v)] x: Watch<Local<T>>` on a `#[component]` |
| a signal set from systems | a resource, read with `Watch<Res<T>>` |
| `memo(\|\| ..)` | compute inline: `$(format!(..))` |

The crate's `MIGRATION.md` has the details.

## Links

[repo](https://github.com/Lacosst0/bevy_radioactive) · [migration guide](https://github.com/Lacosst0/bevy_radioactive/blob/master/MIGRATION.md)

## Next

[[Scenes and BSN]] for the syntax it builds on, [[Buttons and widgets]] for what to put inside, or the rest of [[UI]].

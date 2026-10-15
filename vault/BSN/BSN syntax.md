---
icon: file-code
tags: [bsn]
---
# BSN syntax

Every piece of `bsn!` and `bsn_list!` notation, each with an example. Skim it once, come back when you need it. #bsn

A scene is a list of **entries** separated by whitespace: components, includes (`@`), names (`#`), relationships (`Children [..]`) and observers (`on`). Values are Rust expressions with a few shortcuts.

## Components

| Write | Means |
|:--|:--|
| `Player` | the component, built with its default |
| `Health(10)` | tuple component, first field set |
| `Health(10, 20)` | tuple component, two fields |
| `Stats { hp: 10 }` | named fields; the rest keep their defaults (**patching**) |
| `game::Stats { hp: 10 }` | same, with a module path |
| `Stats { hp }` | field shorthand; means `hp: hp.into()` |
| `Mode::Fast` | an enum component |
| `transform` | a variable that holds a component value |
| `Transform::from_xyz(0., 1., 0.)` | any expression that returns a component |

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
struct Player;

#[derive(Component, Default, Clone)]
struct Health(u32);

#[derive(Component, Default, Clone)]
struct Stats { hp: u32, speed: f32 }

#[derive(Component, Default, Clone)]
enum Mode { #[default] Idle, Fast }

fn examples(hp: u32) -> impl Scene {
    let spawn_point = Transform::from_xyz(0.0, 1.0, 0.0);
    bsn! {
        Player                      // default
        Health(10)                  // tuple
        Stats { hp, speed: 2.5 }    // named; `hp` is shorthand for `hp: hp.into()`
        Mode::Fast                  // enum
        spawn_point                 // a variable
        Visibility::Hidden          // an enum from Bevy
    }
}
```

> [!WARNING]
> Enums must be written out in full: Rust has no "variant defaults", so `Mode::Fast` is the whole value. Structs and tuple structs may name fewer fields.

## Values

Inside a component, a value can be:

| Write | Is |
|:--|:--|
| `1`, `-1`, `1.5`, `true` | number or bool |
| `"text"` | `String` or `&str` |
| `"player.png"` | in a `Handle<T>` field: **load that asset path** ([[Templates and assets]]) |
| `px(8)`, `Color::WHITE` | function call or path |
| `GREEN` | a constant that is in scope |
| `variable` | a variable, without braces |
| `{ a + b }` | any Rust expression; braces required if it is not a plain value |
| `{ vec![1, 2] }` | a collection |
| `#Name` | a reference to a named entity ([below](#names-and-references)) |

```rust
use bevy::prelude::*;

fn values(width: f32, label: &str) -> impl Scene {
    bsn! {
        Node {
            width: px(width),                // a call, using a variable
            height: px(width * 0.5),         // an expression
            padding: UiRect::all(px(4)),
        }
        Text({label.to_string()})            // braces: a method call
        BackgroundColor(Color::srgb(0.1, 0.1, 0.2))
    }
}
```

Variables of the right type are usable as-is (`px(width)`); anything that is not a literal, a path or a plain call goes in braces.

## Names and references

`#Name` gives the entity `Name("Name")` and lets anything in the same `bsn!` refer to it, **even before it exists**.

| Write | Means |
|:--|:--|
| `#Hero` | `Name("Hero")` on this entity, and a handle for others |
| `Follows(#Hero)` | a component field that receives that entity (the component derives `FromTemplate`) |
| `ChildOf(#Hero)` | this entity is a child of `#Hero` |
| `#Hero Name("Sir Hero")` | keep the reference, but show a different name |

```rust
use bevy::prelude::*;

#[derive(Component, FromTemplate)]
struct Follows(Entity);

fn pair() -> impl SceneList {
    bsn_list! {
        #Leader
        --
        #Follower Follows(#Leader)
    }
}
```

Scope rules:

- A name is visible to the root, its children and deeper descendants of the **same** `bsn!`, and parents can name their descendants too.
- Every `bsn!` has its own scope. An included scene may reuse `#X` without clashing with yours.
- A `bsn_list!` shares one scope across its roots, so siblings can point at each other.
- Names are literal. To also give a **dynamic** name, put it after: `#Slot Name({format!("slot {i}")})`.

## Children and relationships

| Write | Means |
|:--|:--|
| `Children [ A -- B ]` | two children |
| `Children [ A B ]` | one child with `A` and `B` |
| `Children [ {list} ]` | every scene of a `SceneList` expression |
| `Children [ #First -- {list} -- #Last ]` | mix |
| `Rel [ ... ]` | any relationship target, e.g. your own (see [[Relationships and hierarchy]]) |
| `ChildOf(parent)` | make *this* entity a child of an `Entity` or `#Name` |

```rust
use bevy::prelude::*;

fn board(cells: usize) -> impl Scene {
    let row = (0..cells).map(|_| bsn! { Text("•") }).collect::<Vec<_>>();
    bsn! {
        Node { column_gap: px(4) }
        Children [
            Text("start")
            --
            {row}
            --
            Text("end")
        ]
    }
}
```

## Includes

`@` brings in another scene; its entries apply **first**, then yours (see [[Composing scenes]]).

| Write | Includes |
|:--|:--|
| `@card()` / `@card("title")` | the result of a scene function |
| `@scene` | a variable that holds a scene |
| `@{ expr }` | any expression returning a scene |
| `@Widget` | a [[Scene components]] with default fields |
| `@Widget { @prop: 1, field: 2 }` | a scene component, with a prop (`@`) and a normal field |
| `:"enemy.bsn"` | **cached** include of a scene asset (the `.bsn` format is not shipped yet) |
| `:card()` / `:Widget` | cached include (reserved; not wired up for functions yet) |

## Observers

| Write | Means |
|:--|:--|
| `on(\|e: On<Ev>\| { .. })` | an observer on this entity for the `EntityEvent` `Ev` |
| `on(handler)` | same, with a function |

The first parameter decides the event. After it, the closure takes any system parameters ([[System params]]), and `move` closures work:

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
struct Health(u32);

#[derive(EntityEvent)]
struct Damage { entity: Entity, amount: u32 }

fn target() -> impl Scene {
    bsn! {
        Health(100)
        on(|hit: On<Damage>, mut health: Query<&mut Health>| {
            if let Ok(mut health) = health.get_mut(hit.entity) {
                health.0 = health.0.saturating_sub(hit.amount);
            }
        })
        on(log_damage)
    }
}

fn log_damage(hit: On<Damage>) {
    info!("{} damage", hit.amount);
}
```

## Templates and code

| Write | Means |
|:--|:--|
| `~{ expr }` | a [`Template`] or component value from an expression, when the plain form is misread |
| `template(\|ctx\| Ok(Comp(..)))` | build a component with world access, when the scene spawns |
| `~MyTemplate { .. }` | a type that implements `Template` itself (not a component's own template) |

See [[Templates and assets]].

## Scene lists

`bsn_list!` is the same notation, but each `--`-separated entry is a **root** entity, and the result is a `SceneList`:

```rust
use bevy::prelude::*;

fn level() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        #Floor Sprite { color: Color::WHITE }
        --
        #Player Transform::from_xyz(0., 20., 0.)
    }
}
```

## Comments, expressions and Rust inside

Rust comments work. There is **no `if` or `match` inside the notation**: compute the value first, or use `{ }`:

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
struct Boss;

fn unit(is_boss: bool) -> impl Scene {
    let hp = if is_boss { 500 } else { 100 };     // decide outside
    let boss: Option<Box<dyn Scene>> = is_boss.then(|| Box::new(bsn! { Boss }) as Box<dyn Scene>);
    bsn! {
        Name({format!("unit {hp}")})
        @{boss}                                   // an Option of a scene is a scene
    }
}
```

More conditional patterns are in [[Composing scenes]].

## One big example

Bevy's own reference snippet, annotated. Everything above in one scene:

```rust ignore
bsn! {
    @some_scene()                // include a scene function
    #SomeName                    // Name("SomeName") + reference handle
    ComponentA                   // default component
    ComponentB(0.0)              // value set, other fields default
    component                    // a variable
    Node { height: px(0.1) }     // named field, others untouched
    on(|e: On<MyEvent>, mut q: Query<&mut ComponentB>| { /* .. */ })
    Children [
        #Child1 ComponentA
        --
        Link(#SomeName)          // an entity reference
        @MySceneComponent { @some_prop: 3, normal_field: 5 }
        --
        Node { width: some_var } // variables without braces
        ComponentB({some_var + 3.})
        @Container { @items: bsn_list! { #Item1 -- #Item2 } }
    ]
}
```

## Next

[[Composing scenes]]: reuse what you wrote.

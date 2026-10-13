---
icon: puzzle
tags: [bsn, scenes, intermediate]
---
# Scene components

A **scene component** is a component that comes with a scene. Add `Player` and you also get its sword and its shield. Systems that query for `Player` can rely on the rest being there. #bsn #scenes

## Make one

Derive `SceneComponent` (it also derives `Component`), then write `scene()` on the type:

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)] struct Sword;
#[derive(Component, Default, Clone)] struct Shield;

#[derive(SceneComponent, Default, Clone)]
struct Player { score: usize }

impl Player {
    fn scene() -> impl Scene {
        bsn! {
            #Player
            Children [
                #RightHand Sword
                --
                #LeftHand Shield
            ]
        }
    }
}

fn start() -> impl Scene {
    bsn! { @Player { score: 0 } }      // spawns the component AND its scene
}
```

Notes:

- You do not write `Player` inside its own scene. It is added for you. (You *can* patch it there, to set a field from a prop.)
- Include it with `@Player`. Its fields are written like a normal component.
- Spawn one with `commands.spawn(Player { .. })` and Bevy logs an error: a scene component must be spawned **as a scene**.
- To use a function with another name, write `#[scene(player)]` above the struct and `fn player() -> impl Scene`.

## Props: settings for the scene

Sometimes a scene needs *input* that is not a field of the component. Those are **props**. They are a plain struct, passed to `scene()` when the scene is included. In BSN a prop is written with `@`:

```rust
use bevy::prelude::*;

/// A UI widget that repeats "hello".
#[derive(SceneComponent, Default, Clone)]
#[scene(HelloProps)]
struct Hello;

#[derive(Default)]
struct HelloProps {
    repeat: usize,
}

impl Hello {
    fn scene(props: HelloProps) -> impl Scene {
        let lines: Vec<_> = (0..props.repeat).map(|_| bsn! { Text("hello") }).collect();
        bsn! {
            Node { flex_direction: FlexDirection::Column }
            Children [ {lines} ]
        }
    }
}

fn greeting() -> impl Scene {
    bsn! { @Hello { @repeat: 5 } }
}
```

The difference to remember:

| | Field | Prop |
|:--|:--|:--|
| written | `value: 10` | `@border: true` |
| lives | on the component | only passed to `scene()` |
| patchable later | yes | **no**: it is used right when the scene is included |

Use both together, and let a prop feed a field by patching the component inside the scene:

```rust preview=bsn-meters
use bevy::prelude::*;

#[derive(SceneComponent, Default, Clone)]
#[scene(MeterProps)]
struct Meter { value: f32 }

#[derive(Default)]
struct MeterProps { percent: f32 }

impl Meter {
    fn scene(props: MeterProps) -> impl Scene {
        let fill = props.percent;
        bsn! {
            Meter { value: {fill / 100.0} }               // prop → field
            Node {
                width: px(160),
                height: px(12),
                border_radius: BorderRadius::all(px(6)),
                overflow: Overflow::clip(),
            }
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2))
            Children [
                Node { width: percent(fill), height: percent(100) }
                BackgroundColor(Color::srgb(0.4, 0.7, 1.0))
            ]
        }
    }
}

fn hud() -> impl Scene {
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(8) }
        Children [
            @Meter { @percent: 25.0 }
            --
            @Meter { @percent: 75.0 }
            --
            @Meter { @percent: 100.0 }
        ]
    }
}
```

Why props at all, when a scene function takes arguments? Because a scene component is **a type**: `Query<&Meter>` finds every meter, and the editor tools show it. A function has no type to query.

## Scene component or required components?

Both solve "when I add X I want Y". They differ in what they can do:

| | Required components | Scene components |
|:--|:--|:--|
| need context (world, assets, names) | no, `Default` only | **yes** |
| children | no | **yes** |
| patchable | no | **yes** |
| enforced when | every time, at runtime | only at scene spawn |
| overhead | very small | higher |

Rules of thumb:

- A hierarchy, assets, or entity references? **Scene component.**
- A flat set of components that must also work with a plain `commands.spawn`? **Required components** ([[Component lifecycle]]).
- Spawning thousands per frame? **Required components.**

## Next

[[Spawning scenes]]: all the ways to put a scene in the world.

---
parent: BSN syntax
icon: file-input
tags: [bsn, assets]
---
# Templates and assets

Why does one component work in `bsn!` with `#[derive(Default, Clone)]`, and another needs `#[derive(FromTemplate)]`? Because a scene is spawned **later**, when the world can hand it things a plain value cannot know: asset handles, entity ids. #bsn #assets

## The idea

A scene is not a bag of finished components. It is a bag of **templates**: recipes that, given the world and the entity being built, produce a component.

| Type | Template |
|:--|:--|
| `Health(u32)` | itself (nothing to look up) |
| `Handle<Image>` | `HandleTemplate<Image>`: "load this path when you spawn" |
| `Entity` | `EntityTemplate`: "the entity named `#Hero`" |

For simple components Bevy writes the template for you.

## Pick the derive

| Your component has | Derive |
|:--|:--|
| only plain data (numbers, strings, colours) | `Component, Default, Clone` |
| a `Handle<T>`, an `Entity`, or another template field | `Component, FromTemplate` |
| a handle **and** you want props and a scene | `SceneComponent`, see [[Scene components]] |

```rust
use bevy::prelude::*;

// Plain data: Default + Clone.
#[derive(Component, Default, Clone)]
struct Speed(f32);

// Holds a handle and an entity: FromTemplate.
#[derive(Component, FromTemplate)]
struct Icon {
    image: Handle<Image>,
    owner: Entity,
}

fn badge() -> impl Scene {
    bsn! {
        #Owner
        Speed(3.0)
        Children [
            Icon { image: "icons/sword.png", owner: #Owner }
        ]
    }
}
```

> [!WARNING]
> Derive `Default` **or** `FromTemplate`, never both: each writes the same trait impl. A `FromTemplate` type still gets a default for patching: the derive makes a companion `IconTemplate` that implements `Default`.

## Asset paths become handles

In a field of type `Handle<T>`, a string is a path. The scene loads it when it spawns, and reuses the handle if the file is already loaded:

```rust
use bevy::prelude::*;

fn player() -> impl Scene {
    bsn! {
        Sprite { image: "player.png" }
        Transform::from_xyz(0.0, 0.0, 1.0)
    }
}
```

Without BSN you carried an `AssetServer` down every function that spawns a thing. With BSN, `player()` takes no arguments at all. A path built at runtime works with braces:

```rust
use bevy::prelude::*;

fn character(name: &str) -> impl Scene {
    let path = format!("characters/{name}.png");
    bsn! { Sprite { image: {path} } }
}
```

## Assets you create: `asset_value`

For an asset that lives in code, not in a file (a mesh, a material), `asset_value(x)` stores it in `Assets<T>` and gives a handle:

```rust
use bevy::prelude::*;

fn crate_block() -> impl Scene {
    bsn! {
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
    }
}
```

## Entities: names are references

A field of type `Entity` takes `#Name`. The scene resolves it to the real entity at spawn time, even for siblings that spawn *after* the one that points at them. Never store the template; store the resolved `Entity` and derive `FromTemplate` so the conversion is automatic.

```rust
use bevy::prelude::*;

#[derive(Component, FromTemplate)]
struct Target(Entity);

fn duel() -> impl SceneList {
    bsn_list! {
        #Red Target(#Blue)
        --
        #Blue Target(#Red)
    }
}
```

## Values built right now

A component value you already built goes in as a variable. Anything the macro would misread goes in `~{ }`:

```rust
use bevy::prelude::*;

fn at(x: f32) -> impl Scene {
    let transform = Transform::from_xyz(x, 0.0, 0.0);
    bsn! {
        transform                              // a variable
        ~{ Name::new(format!("Marker {x}")) }  // an expression
    }
}
```

> [!NOTE]
> Older code wrapped these in `template_value(..)`. It still works but is deprecated.

## Anything else: `template`

`template(|ctx| ...)` gets the entity under construction and the world, and returns a `Result`. Use it when a component depends on something only the world knows:

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct Palette(Vec<Color>);

fn tinted(index: usize) -> impl Scene {
    bsn! {
        template(move |ctx| {
            let palette = ctx.resource::<Palette>();
            Ok(BackgroundColor(palette.0[index % palette.0.len()]))
        })
    }
}
```

`~Type { .. }` is the third door: it writes a value of a type that implements `Template` directly. You only need it when you write templates yourself. A template that is not `Clone + Default` needs both written by hand:

```rust
use bevy::prelude::*;

/// A template that produces a component.
struct HalfHealth { max: u32 }

impl Default for HalfHealth {
    fn default() -> Self { HalfHealth { max: 10 } }
}

#[derive(Component)]
struct Health(u32);

impl Template for HalfHealth {
    type Output = Health;
    fn build_template(&self, _: &mut bevy::ecs::template::TemplateContext) -> Result<Health> {
        Ok(Health(self.max / 2))
    }
    fn clone_template(&self) -> Self { HalfHealth { max: self.max } }
}

fn wounded() -> impl Scene {
    bsn! { ~HalfHealth { max: 100 } }
}
```

## Enums

Enums cannot be partial, since Rust has no "default variant" per field. Write the whole value: `Mode::Fast`, `Visibility::Hidden`, `Justify::Center`.

## Next

[[Scene components]]: scenes that carry their own children and settings.

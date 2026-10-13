---
icon: package
tags: [lib, audio]
bevy: 0.19
crates: bevy_seedling=0.8
---
# bevy_seedling

An **audio graph** for Bevy, built on Firewheel: sound sources feed into pools and buses where you add effects (reverb, EQ, filters, spatialisation) just by spawning entities and connecting them. The most flexible Bevy audio today. #lib #audio

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.8` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_seedling = "0.8"
```

(Its README lists the Bevy features it needs when you use `default-features = false`.)

## Play a sample

```rust
use bevy::prelude::*;
use bevy_seedling::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SeedlingPlugins))
        .add_systems(Startup, play)
        .run();
}

fn play(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn(SamplePlayer::new(server.load("sounds/theme.ogg")).looping());
}
```

A sound is an entity with a `SamplePlayer`; despawn it to stop it. Everything else (volume, pan, effects) is more components on that entity or on the pool it plays in.

## Compared

| | Bevy audio ([[Audio]]) | [[bevy_kira_audio]] | bevy_seedling |
|:--|:--|:--|:--|
| Model | entities with `AudioPlayer` | channels + a global `Audio` | an audio graph in the ECS |
| Effects | none | some | many, composable |
| Bevy 0.20 | yes | **yes** | pending |

Choose it when you want sophisticated mixing, dynamic music, or spatial audio with effects.

## Links

[crates.io](https://crates.io/crates/bevy_seedling) · [docs.rs](https://docs.rs/bevy_seedling) · [repo](https://github.com/CorvusPrudens/bevy_seedling)

## Next

[[Audio]], [[Libraries]].

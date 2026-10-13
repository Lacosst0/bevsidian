---
icon: puzzle
tags: [start, beginner]
---
# App and plugins

An [`App`] is a [[Entities and components|World]] (all your data), a set of [[Schedules and sets|schedules]] (all your systems) and a runner that loops. A **plugin** is a bundle of setup you can add to it. #start #beginner

## Building an app

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)    // window, renderer, input, assets...
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .add_systems(Startup, setup)    // runs once
        .add_systems(Update, tick)      // runs every frame
        .run();                         // enters the loop
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn tick(time: Res<Time>) {
    if time.elapsed_secs() < 0.02 {
        info!("first frame");
    }
}
```

Every `App` method just edits the world and the schedules, so the order of `.add_*` calls rarely matters, except for plugins.

## Plugins

A plugin is anything that can configure an app. The simplest is a function:

```rust
use bevy::prelude::*;

fn player_plugin(app: &mut App) {
    app.add_systems(Startup, spawn_player)
        .add_systems(Update, move_player);
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(Name::new("Player"));
}

fn move_player() {}

fn main() {
    App::new().add_plugins((DefaultPlugins, player_plugin)).run();
}
```

Use a struct when the plugin needs settings:

```rust
use bevy::prelude::*;

struct EnemyPlugin {
    count: usize,
}

#[derive(Resource)]
struct EnemyCount(usize);

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyCount(self.count));
    }
}

fn main() {
    App::new().add_plugins(EnemyPlugin { count: 10 }).run();
}
```

> [!TIP]
> Prefer a [[Resources|resource]] for settings: players and tools can change it while the game runs. Use plugin fields only for things decided once, like which schedule to use.

## DefaultPlugins is a group

[`DefaultPlugins`] is a `PluginGroup`: change one member with `.set(...)`, remove one with `.disable::<T>()`.

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "My game".into(),
                        resolution: (960, 540).into(),
                        ..default()
                    }),
                    ..default()
                })
                // Crisp pixel art: no smoothing when scaling images.
                .set(ImagePlugin::default_nearest()),
        )
        .run();
}
```

For tests and servers there is [`MinimalPlugins`]: just the loop and time, no window.

## Rules of thumb

- Plugins run in the order you add them. If one needs another, add that first.
- Adding the same plugin twice panics.
- One plugin per feature (`InventoryPlugin`, `AudioPlugin`): see [[Project structure]].

## Next

[[Entities and components]] (the data), [[Systems]] (the logic) and [[Schedules and sets]] (the order).

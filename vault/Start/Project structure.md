---
icon: folder
tags: [start]
---
# Project structure

How to split a game before `main.rs` gets out of hand. #start

## One plugin per feature

Group everything about one feature in one module, with a plugin as its front door:

```
src/
  main.rs          // builds the App, adds the plugins
  player.rs        // PlayerPlugin: components, systems, observers
  enemy.rs         // EnemyPlugin
  ui/
    mod.rs         // UiPlugin
    hud.rs
    menu.rs
  state.rs         // GameState and transitions
assets/
```

```rust
// main.rs
use bevy::prelude::*;

mod player {
    use bevy::prelude::*;
    pub struct PlayerPlugin;
    impl Plugin for PlayerPlugin {
        fn build(&self, _app: &mut App) {}
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, player::PlayerPlugin))
        .run();
}
```

## Keep types near their systems

A component, the systems that use it and the plugin that registers them belong in one file. If another feature needs the component, make it `pub`. That is the whole API between features.

## Talk through events, not imports

When the player dies, `player.rs` shouldn't call into `ui.rs`. Trigger an event and let the other plugin observe it. See [[Events and observers]] and [[Messages]].

## Game states

Menu, playing, paused: model them as [[States]]. Each plugin adds its systems with `run_if(in_state(...))` or `OnEnter`.

## Libraries

If something is useful in more than one game, move it to its own crate in a Cargo workspace. Libraries should expose a plugin and avoid doing anything outside it.

## Next

[[States]] for the game flow, [[Schedules and sets]] for ordering systems across plugins.

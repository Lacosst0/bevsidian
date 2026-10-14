---
parent: Models and glTF
icon: clapperboard
tags: [3d]
---
# 3D animation

Models bring clips (idle, walk, attack). An [`AnimationPlayer`] plays them, and an [`AnimationGraph`] says which and how they mix. #3d

## Play one clip

```rust
use bevy::{prelude::*, world_serialization::WorldInstanceReady};

const MODEL: &str = "models/fox.glb";

#[derive(Component)]
struct ToPlay {
    graph: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // A graph with one clip: animation #2 of the file (say, "Run").
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(MODEL)),
    );

    commands
        .spawn((
            ToPlay { graph: graphs.add(graph), index },
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL))),
        ))
        .observe(start_when_ready);
}

fn start_when_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    to_play: Query<&ToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    let Ok(to_play) = to_play.get(ready.entity) else { return };
    for child in children.iter_descendants(ready.entity) {
        if let Ok(mut player) = players.get_mut(child) {
            player.play(to_play.index).repeat();
            commands.entity(child).insert(AnimationGraphHandle(to_play.graph.clone()));
        }
    }
}
```

Steps: load the clip → put it in a graph → find the `AnimationPlayer` the model came with → tell it to `play` a node, and give that entity the graph.

## Blend several clips

A graph is a tree. Add several clips under a blend node, and weigh them:

```rust
use bevy::prelude::*;

fn build(asset_server: &AssetServer) -> (AnimationGraph, [AnimationNodeIndex; 2]) {
    let mut graph = AnimationGraph::new();
    let walk = graph.add_clip(asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/fox.glb")), 1.0, graph.root);
    let run = graph.add_clip(asset_server.load(GltfAssetLabel::Animation(2).from_asset("models/fox.glb")), 1.0, graph.root);
    (graph, [walk, run])
}

fn speed_to_blend(mut players: Query<&mut AnimationPlayer>, speed: f32, nodes: [AnimationNodeIndex; 2]) {
    for mut player in &mut players {
        // Cross-fade walk to run as speed rises.
        let t = (speed / 6.0).clamp(0.0, 1.0);
        if let Some(walk) = player.animation_mut(nodes[0]) { walk.set_weight(1.0 - t); }
        if let Some(run) = player.animation_mut(nodes[1]) { run.set_weight(t); }
    }
}
```

## Control playback

On the active animation returned by `play(...)`: `.repeat()`, `.set_speed(2.0)`, `.pause()`, `.seek_to(0.5)`, `.set_weight(0.5)`. On the player: `stop_all()`, `is_playing_animation(index)`.

## Animate anything: AnimationClip in code

Clips are not only for skeletons: animate a `Transform`, a colour or a light with a clip built in code (see Bevy's `animated_transform`, `color_animation` and `eased_motion` examples), or use `bevy_tweening` ([[bevy_tweening]]) for quick tweens.

## Events and masks

- **Animation events** fire at moments in a clip ("footstep") and arrive as observer events.
- **Masks** play different clips on different body parts: *shoot* with the upper body while *running* with the legs.

## Morph targets

Facial expressions and shapekeys play through the same player; the model needs morph targets exported.

## Next

[[Environment]].

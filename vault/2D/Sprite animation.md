---
icon: clapperboard
tags: [2d, beginner]
---
# Sprite animation

Flipbook animation: show frame 0, then 1, then 2... on a timer. #2d #beginner

## The recipe

You have a sprite sheet ([[Texture atlases]]), a [[Time and timers|timer]], and a system that advances the frame when the timer fires. The slime below is drawn with a four-frame sheet. Click it to speed it up; click the background to slow it down.

```rust example=animation fps=8
use bevy::prelude::*;

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn spawn_slime(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(10, 8), 4, 1, None, None));
    let indices = AnimationIndices { first: 0, last: 3 };

    commands.spawn((
        Sprite::from_atlas_image(
            asset_server.load("slime.png"),
            TextureAtlas { layout, index: indices.first },
        ),
        Transform::from_scale(Vec3::splat(8.0)),
        indices,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));
}

fn animate(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = if atlas.index == indices.last { indices.first } else { atlas.index + 1 };
        }
    }
}
```

The timer's duration is the time per frame: 0.2 s is 5 frames per second. Change it and everything speeds up or slows down:

```rust example=animation fps=8
use std::time::Duration;

#[derive(Component)]
struct Sky;

fn change_speed(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    mut timers: Query<&mut AnimationTimer>,
) {
    let seconds = if sky.contains(click.entity) { 0.2 } else { 0.06 };
    for mut timer in &mut timers {
        timer.set_duration(Duration::from_secs_f32(seconds));
    }
}
```

## Several animations: states

A character has *idle*, *run*, *jump*. Keep each as a range of frames and choose by the entity's state:

```rust
use bevy::prelude::*;

#[derive(Component, PartialEq, Clone, Copy)]
enum Action { Idle, Run }

impl Action {
    /// Frames in the sheet: (first, last)
    fn frames(self) -> (usize, usize) {
        match self {
            Action::Idle => (0, 3),
            Action::Run => (4, 9),
        }
    }
}

fn on_action_change(mut query: Query<(&Action, &mut Sprite), Changed<Action>>) {
    for (action, mut sprite) in &mut query {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = action.frames().0;   // start the new animation from its first frame
        }
    }
}
```

([[Change detection]] makes sure it only runs when the action changed.)

## Not only sprites

Anything numeric can be animated the same way. For smooth movement, scaling, colour and easing look at [[bevy_tweening]]. For skeletal and 3D animation Bevy has `AnimationPlayer` and `AnimationGraph` ([[3D animation]]). For sprite sheets with named clips and events, there is [[bevy_spritesheet_animation]].

## Next

[[Tilemaps]].

---
icon: type
tags: [2d, beginner]
---
# Text 2D

Words that live **in the world**: damage numbers, labels over enemies, signs. They move with the camera like sprites. (For menus and HUD use [[Text and images|UI text]].) #2d #beginner

## Hello, world

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text2d::new("Hello, Bevy!"),
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf").into(),
            font_size: FontSize::Px(50.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.4)),
        Transform::from_xyz(0.0, 100.0, 1.0),
    ));
}
```

- `Text2d` holds the string.
- `TextFont` chooses the font and size. A font file goes in `assets/fonts/`. Leave `font` as default for the built-in one.
- `TextColor` colours it.

## Change the text

```rust
use bevy::prelude::*;

#[derive(Component)] struct ScoreLabel;
#[derive(Resource)] struct Score(u32);

fn update_score(score: Res<Score>, mut label: Single<&mut Text2d, With<ScoreLabel>>) {
    if score.is_changed() {
        label.0 = format!("Score: {}", score.0);
    }
}
```

## Layout

```rust
use bevy::{prelude::*, text::{Justify, LineBreak, TextBounds}};

fn paragraph(mut commands: Commands) {
    commands.spawn((
        Text2d::new("A long line that should wrap inside a box."),
        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        TextBounds::new(200.0, 100.0),       // wrap at 200 wide
    ));
}
```

Use `Anchor` to align from a corner and `Text2dShadow` for a drop shadow.

## Floating damage numbers

A neat pattern: spawn text at the hit, rise and fade it with a [[Time and timers|timer]], despawn at the end.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Floating(Timer);

fn spawn_number(commands: &mut Commands, at: Vec2, amount: u32) {
    commands.spawn((
        Text2d::new(format!("-{amount}")),
        TextColor(Color::srgb(1.0, 0.3, 0.3)),
        Transform::from_translation(at.extend(10.0)),
        Floating(Timer::from_seconds(0.8, TimerMode::Once)),
    ));
}

fn float_up(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut Floating)>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut color, mut floating) in &mut query {
        transform.translation.y += 40.0 * time.delta_secs();
        color.0 = color.0.with_alpha(1.0 - floating.0.fraction());
        if floating.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

## Next

You can draw, animate and write in 2D. Add control: [[Keyboard and mouse]]. Or jump into [[A 2D game]].

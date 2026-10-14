---
icon: type
tags: [ui]
---
# Text and images

What goes inside boxes. #ui

## Text

```rust
use bevy::prelude::*;

fn labels(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf").into(),   // omit for the default font
            font_size: FontSize::Px(32.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            left: px(10),
            ..default()
        },
    ));
}
```

| Component | Role |
|:--|:--|
| `Text` | the string |
| `TextFont` | font file and `FontSize` (`Px`, `Rem`, `Vh`...) |
| `TextColor` | colour |
| `TextLayout` | alignment and wrapping |
| `TextShadow`, `Underline`, `TextBackgroundColor` | decoration |

## Change it

```rust
use bevy::prelude::*;

#[derive(Component)] struct ScoreText;
#[derive(Resource)] struct Score(u32);

fn show_score(score: Res<Score>, mut text: Single<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        text.0 = format!("Score: {}", score.0);
    }
}
```

## Several styles in one line

Child [`TextSpan`]s share the parent's line but have their own style:

```rust
use bevy::prelude::*;

fn fps(mut commands: Commands) {
    commands
        .spawn((Text::new("FPS: "), TextFont { font_size: FontSize::Px(24.0), ..default() }))
        .with_child((
            TextSpan::new("60"),
            TextColor(Color::srgb(0.4, 1.0, 0.4)),
        ));
}
```

## Images

```rust
use bevy::prelude::*;

fn portrait(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        ImageNode::new(asset_server.load("portrait.png")),
        Node { width: px(96), height: px(96), ..default() },
    ));
}
```

`ImageNode` can use a [[Texture atlases|texture atlas]] for icon sheets, `ImageNode::with_mode(NodeImageMode::Sliced(...))` for 9-slice panels that stretch without distorting corners.

## Boxes and borders

```rust
use bevy::prelude::*;

fn panel() -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(16)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
        BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
        BoxShadow::new(Color::BLACK.with_alpha(0.5), px(0), px(4), px(0), px(10)),
    )
}
```

Gradients, shadows and rounded corners are all built in.

## Fonts

- Put `.ttf`/`.otf` files in `assets/fonts/`.
- With the `default_font` feature, Bevy has a built-in font, fine for prototypes.
- System fonts and generic families (`FontSource::families(...)`) are supported on native.

## Next

[[Buttons and widgets]].

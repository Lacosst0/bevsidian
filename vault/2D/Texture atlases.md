---
parent: Sprites
icon: table
tags: [2d]
---
# Texture atlases

Many small pictures in one image. Faster to load and draw, and how animation frames and tilesets are stored. #2d

## A sprite sheet

An image of equal-sized frames in a grid. Tell Bevy how it is cut up with a `TextureAtlasLayout`, then pick a frame by index:

```rust
use bevy::prelude::*;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    let sheet = asset_server.load("hero_sheet.png");
    // 24x24 pixel frames, 7 columns, 1 row.
    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None));

    commands.spawn((
        Sprite::from_atlas_image(sheet, TextureAtlas { layout, index: 3 }),
        Transform::from_scale(Vec3::splat(6.0)),
    ));
}
```

`from_grid(frame_size, columns, rows, padding, offset)`. Frame `0` is the top-left one, counting along the row.

## Change the frame

```rust
use bevy::prelude::*;

fn next_frame(mut sprites: Query<&mut Sprite>) {
    for mut sprite in &mut sprites {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = (atlas.index + 1) % 7;
        }
    }
}
```

That is all animation is. The next note puts a timer behind it: [[Sprite animation]].

## Why atlases?

- **One image, one draw call.** Bevy batches sprites that share an image.
- **One load**, one handle to keep alive ([[Assets]]).
- **Tilesets** for [[Tilemaps]] are atlases too.

## Padding

Neighbouring frames can bleed into each other when scaled. If you see thin lines, leave a gap between frames in the image and pass it as `padding`, or use `ImagePlugin::default_nearest()`.

## Building an atlas at runtime

Have a folder of loose images? `TextureAtlasBuilder` packs them into one atlas at startup (see Bevy's `texture_atlas` example). Better: pack at build time with a tool such as `TexturePacker` or `Aseprite`'s export.

## Next

[[Sprite animation]].

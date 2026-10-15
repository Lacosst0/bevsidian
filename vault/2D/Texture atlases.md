---
parent: Sprites
icon: table
tags: [2d]
---
# Texture atlases

Many small pictures in one image: faster to load and draw, and how animation frames and tilesets are stored. #2d

## A sprite sheet

An image of equal-sized frames in a grid. A `TextureAtlasLayout` says how it's cut up; the sprite picks a frame by index. Here, a slime in four frames. Press **Next frame**:

```rust example=animation
use bevy::prelude::*;

fn spawn_slime(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // 10 × 8 pixel frames: 4 columns, 1 row, no padding.
    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(10, 8), 4, 1, None, None));
    commands.spawn((
        Sprite::from_atlas_image(asset_server.load("slime.png"), TextureAtlas { layout, index: 0 }),
        Transform::from_scale(Vec3::splat(8.0)),
    ));
}

fn next_frame(mut sprites: Query<&mut Sprite>) {
    for mut sprite in &mut sprites {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = (atlas.index + 1) % 4;
        }
    }
}
```

`from_grid(frame_size, columns, rows, padding, offset)`; frame `0` is top-left, counting along each row. Changing the index on a timer is all animation is: [[Sprite animation]].

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

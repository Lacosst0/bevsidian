//! The game view: draws a preview world's sprites, and is that world's picking backend.
//!
//! Each sprite of the preview world is drawn by a UI node (an image node at the sprite's size
//! and position, scaled to the view). Pointer input over a drawn sprite is triggered in the
//! preview world as Bevy's own picking events (`PointerPress`, `PointerClick`, drags…) on that
//! sprite's entity, with the hit position in world coordinates, so the example's observers
//! are ordinary Bevy picking code.

use std::collections::HashMap;

use bevy::{
    camera::NormalizedRenderTarget,
    picking::{backend::HitData, events::Pointer, pointer::Location},
    prelude::*,
    sprite::Anchor,
    ui::UiGlobalTransform,
};
use bevy_ui_textselect::NoTextSelection;
use bevy_ui_touchscroll::{OwnsDrags, TouchScroll};

use crate::{WorldPreview, runtime::PreviewRuntime};

/// A game preview, `size` world units across.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct GameView {
    pub size: Vec2,
}

/// A node drawing the sprite of this entity of the preview world.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Drawn(Entity);

/// The view. When `interactive` (the game observes pointer input), touch drags over it are
/// the game's rather than page scrolls.
pub(crate) fn game_frame(
    preview: WorldPreview,
    size: Vec2,
    background: Color,
    interactive: bool,
) -> Box<dyn Scene> {
    let view = GameView { size };
    let aspect_ratio = Some(size.x / size.y);
    let frame = bsn! {
        preview
        view
        Node { width: Val::Percent(100.0), aspect_ratio: aspect_ratio, overflow: Overflow::clip() }
        BackgroundColor(background)
        on(press)
        on(click)
        on(drag_start)
        on(drag)
        on(drag_end)
        // Its presses are the game's, not the start of a text selection.
        NoTextSelection
    };
    if interactive {
        Box::new(bsn! { @{frame} OwnsDrags })
    } else {
        Box::new(frame)
    }
}

/// Updates the nodes of the sprites of the games that changed or resized.
pub(crate) fn draw_games(
    mut games: Query<
        (
            Entity,
            &mut PreviewRuntime,
            &GameView,
            &ComputedNode,
            Option<&Children>,
        ),
        Or<(Changed<PreviewRuntime>, Changed<ComputedNode>)>,
    >,
    mut drawn: Query<(
        &Drawn,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
        &mut ZIndex,
    )>,
    images: Res<Assets<Image>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    for (host, mut runtime, view, computed, children) in &mut games {
        // Logical px per world unit.
        let scale = computed.size().x * computed.inverse_scale_factor() / view.size.x;
        if scale <= 0.0 {
            continue;
        }
        let mut nodes: HashMap<Entity, Entity> = children
            .into_iter()
            .flatten()
            .filter_map(|&child| drawn.get(child).ok().map(|d| (d.0.0, child)))
            .collect();
        let world = &mut runtime.bypass_change_detection().world;
        let mut sprites = world.query::<(Entity, &Sprite, &Transform, Option<&Anchor>)>();
        for (entity, sprite, transform, anchor) in sprites.iter(world) {
            let size = sprite_size(sprite, &images, &layouts) * transform.scale.truncate().abs();
            let center =
                transform.translation.truncate() - anchor.map_or(Vec2::ZERO, |a| a.0) * size;
            let top_left = Vec2::new(
                center.x - size.x / 2.0 + view.size.x / 2.0,
                view.size.y / 2.0 - center.y - size.y / 2.0,
            ) * scale;
            let width = Val::Px(size.x * scale);
            let height = Val::Px(size.y * scale);
            // Screen y points down, so a counter-clockwise turn is a negative angle.
            let angle = transform.rotation.to_euler(EulerRot::ZYX).0;
            let translation = UiTransform {
                translation: Val2::px(top_left.x, top_left.y),
                rotation: Rot2::radians(-angle),
                ..default()
            };
            let image = ImageNode {
                image: sprite.image.clone(),
                color: sprite.color,
                flip_x: sprite.flip_x ^ (transform.scale.x < 0.0),
                flip_y: sprite.flip_y ^ (transform.scale.y < 0.0),
                rect: sprite.rect,
                texture_atlas: sprite.texture_atlas.clone(),
                image_mode: match sprite.image_mode {
                    SpriteImageMode::Tiled {
                        tile_x,
                        tile_y,
                        stretch_value,
                    } => NodeImageMode::Tiled {
                        tile_x,
                        tile_y,
                        stretch_value: stretch_value * scale,
                    },
                    _ => NodeImageMode::Stretch,
                },
                ..default()
            };
            let z = ZIndex((transform.translation.z * 16.0) as i32);
            if let Some(node) = nodes.remove(&entity)
                && let Ok((_, mut style, mut at, mut current, mut order)) = drawn.get_mut(node)
            {
                if style.width != width || style.height != height {
                    style.width = width;
                    style.height = height;
                }
                at.set_if_neq(translation);
                if !same_image(&current, &image) {
                    *current = image;
                }
                order.set_if_neq(z);
            } else {
                commands.spawn((
                    Drawn(entity),
                    Node {
                        position_type: PositionType::Absolute,
                        width,
                        height,
                        ..default()
                    },
                    translation,
                    image,
                    z,
                    ChildOf(host),
                ));
            }
        }
        // Sprites that are gone.
        for node in nodes.into_values() {
            commands.entity(node).despawn();
        }
    }
}

fn same_image(a: &ImageNode, b: &ImageNode) -> bool {
    a.image == b.image
        && a.color == b.color
        && a.flip_x == b.flip_x
        && a.flip_y == b.flip_y
        && a.rect == b.rect
        && a.texture_atlas == b.texture_atlas
        && a.image_mode == b.image_mode
}

/// A sprite's size in world units, before scaling.
fn sprite_size(
    sprite: &Sprite,
    images: &Assets<Image>,
    layouts: &Assets<TextureAtlasLayout>,
) -> Vec2 {
    sprite
        .custom_size
        .or_else(|| sprite.rect.map(|rect| rect.size()))
        .or_else(|| {
            let atlas = sprite.texture_atlas.as_ref()?;
            let cell = layouts.get(&atlas.layout)?.textures.get(atlas.index)?;
            Some(cell.size().as_vec2())
        })
        .or_else(|| images.get(&sprite.image).map(Image::size_f32))
        .unwrap_or_default()
}

/// Where a pointer is in a game: the preview entity drawn under it, the pointer in the game's
/// screen (world units from its top-left) and the hit in world coordinates.
struct Spot {
    target: Entity,
    pointer: Pointer,
    hit: HitData,
    /// Logical px per world unit.
    scale: f32,
}

type Views<'w, 's> = Query<
    'w,
    's,
    (
        &'static GameView,
        &'static ComputedNode,
        &'static UiGlobalTransform,
    ),
>;

fn spot(
    host: Entity,
    under: Entity,
    pointer: &Pointer,
    views: &Views,
    drawn: &Query<&Drawn>,
) -> Option<Spot> {
    let target = drawn.get(under).ok()?.0;
    let (view, node, at) = views.get(host).ok()?;
    let inverse = node.inverse_scale_factor();
    let size = node.size() * inverse;
    let scale = size.x / view.size.x;
    let top_left = at.translation * inverse - size / 2.0;
    let position = (pointer.position - top_left) / scale;
    let world = Vec2::new(
        position.x - view.size.x / 2.0,
        view.size.y / 2.0 - position.y,
    );
    let screen = NormalizedRenderTarget::None {
        width: view.size.x as u32,
        height: view.size.y as u32,
    };
    Some(Spot {
        target,
        pointer: Pointer::new(
            pointer.id,
            Location {
                target: screen,
                position,
            },
        ),
        hit: HitData::new(Entity::PLACEHOLDER, 0.0, Some(world.extend(0.0)), None),
        scale,
    })
}

fn press(
    press: On<PointerPress>,
    views: Views,
    drawn: Query<&Drawn>,
    mut runtimes: Query<&mut PreviewRuntime>,
) {
    let Some(spot) = spot(
        press.entity,
        press.original_event_target(),
        &press.pointer,
        &views,
        &drawn,
    ) else {
        return;
    };
    if let Ok(mut runtime) = runtimes.get_mut(press.entity) {
        // Playing with it starts it.
        runtime.playing = true;
        runtime.trigger(PointerPress {
            entity: spot.target,
            pointer: spot.pointer,
            button: press.button,
            hit: spot.hit,
            count: press.count,
        });
    }
}

fn click(
    click: On<PointerClick>,
    views: Views,
    drawn: Query<&Drawn>,
    touch: Res<TouchScroll>,
    mut runtimes: Query<&mut PreviewRuntime>,
) {
    // A touch that scrolled the page over a game is not a click in it.
    if touch.gesture_scrolled() {
        return;
    }
    let Some(spot) = spot(
        click.entity,
        click.original_event_target(),
        &click.pointer,
        &views,
        &drawn,
    ) else {
        return;
    };
    if let Ok(mut runtime) = runtimes.get_mut(click.entity) {
        runtime.trigger(PointerClick {
            entity: spot.target,
            pointer: spot.pointer,
            button: click.button,
            hit: spot.hit,
            duration: click.duration,
            count: click.count,
        });
    }
}

fn drag_start(
    drag: On<PointerDragStart>,
    views: Views,
    drawn: Query<&Drawn>,
    mut runtimes: Query<&mut PreviewRuntime>,
) {
    let Some(spot) = spot(
        drag.entity,
        drag.original_event_target(),
        &drag.pointer,
        &views,
        &drawn,
    ) else {
        return;
    };
    if let Ok(mut runtime) = runtimes.get_mut(drag.entity) {
        runtime.trigger(PointerDragStart {
            entity: spot.target,
            pointer: spot.pointer,
            button: drag.button,
            hit: spot.hit,
        });
    }
}

fn drag(
    drag: On<PointerDrag>,
    views: Views,
    drawn: Query<&Drawn>,
    mut runtimes: Query<&mut PreviewRuntime>,
) {
    let Some(spot) = spot(
        drag.entity,
        drag.original_event_target(),
        &drag.pointer,
        &views,
        &drawn,
    ) else {
        return;
    };
    if let Ok(mut runtime) = runtimes.get_mut(drag.entity) {
        runtime.trigger(PointerDrag {
            entity: spot.target,
            pointer: spot.pointer,
            button: drag.button,
            distance: drag.distance / spot.scale,
            delta: drag.delta / spot.scale,
        });
    }
}

fn drag_end(
    drag: On<PointerDragEnd>,
    views: Views,
    drawn: Query<&Drawn>,
    mut runtimes: Query<&mut PreviewRuntime>,
) {
    let Some(spot) = spot(
        drag.entity,
        drag.original_event_target(),
        &drag.pointer,
        &views,
        &drawn,
    ) else {
        return;
    };
    if let Ok(mut runtime) = runtimes.get_mut(drag.entity) {
        runtime.trigger(PointerDragEnd {
            entity: spot.target,
            pointer: spot.pointer,
            button: drag.button,
            distance: drag.distance / spot.scale,
        });
    }
}

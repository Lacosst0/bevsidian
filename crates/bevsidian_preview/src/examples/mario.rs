//! `Mario.md`: a one-screen platformer. Mario stands on the ground and falls onto whatever is
//! below him; goombas walk until they hit something and turn around; Mario landing on a goomba
//! squishes it.
//!
//! The level, gravity, collisions and stomping run in every preview. The notes' blocks add
//! goombas, platforms placed by clicking, and dragging Mario around.

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::example::{Example, Function, Setup, View, observer, startup, update};

/// The view: 320 × 160 world units, the origin in the middle.
const SIZE: Vec2 = Vec2::new(320.0, 160.0);
const SKY: Color = Color::srgb(0.361, 0.580, 0.988);

pub static EXAMPLE: Example = Example {
    setup,
    base: BASE,
    functions: FUNCTIONS,
    tick: None,
    actions: &[],
    also: &[],
    view: View::Game {
        size: SIZE,
        background: SKY,
    },
};

const BASE: &[Function] = &[
    startup!(spawn_level),
    update!(gravity),
    update!(land),
    update!(walk),
    update!(stomp),
    update!(squash),
    update!(fall_out),
];

const FUNCTIONS: &[Function] = &[
    update!(spawn_goombas),
    observer!(spawn_platform),
    observer!(grab),
    observer!(drag_mario),
    observer!(release),
];

fn setup(app: &World) -> Setup {
    let sprites = app.get_resource::<Sprites>().cloned().unwrap_or_default();
    Box::new(move |world| {
        world.insert_resource(sprites.clone());
        world.init_resource::<GoombaTimer>();
    })
}

/// Draws the sprites once for the app (previews share them).
pub fn plugin(app: &mut App) {
    app.add_systems(Startup, create_sprites);
}

// ---------------------------------------------------------------------------
// The program, as the note shows it
// ---------------------------------------------------------------------------

/// The pixel art.
#[derive(Resource, Clone, Default)]
pub struct Sprites {
    pub mario: Handle<Image>,
    pub goomba: Handle<Image>,
    pub brick: Handle<Image>,
    pub ground: Handle<Image>,
}

#[derive(Component)]
struct Mario;

#[derive(Component)]
struct Goomba;

/// The background: clicks that hit nothing else land on it.
#[derive(Component)]
struct Sky;

/// Ground and platforms: bodies land on them.
#[derive(Component)]
struct Solid;

/// The box a body collides with, in world units.
#[derive(Component)]
struct Collider(Vec2);

#[derive(Component, Default)]
struct Velocity(Vec2);

/// Being dragged: gravity leaves it alone.
#[derive(Component)]
struct Held;

/// A stomped goomba, gone when the timer ends.
#[derive(Component)]
struct Squished(Timer);

#[derive(Resource)]
struct GoombaTimer(Timer);

impl Default for GoombaTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

const GRAVITY: f32 = 900.0;
const MAX_FALL: f32 = 300.0;
const GOOMBA_SPEED: f32 = 40.0;
const PLATFORM: Vec2 = Vec2::new(48.0, 16.0);
/// On the ground (32 units high), towards the left.
const MARIO_START: Vec3 = Vec3::new(-100.0, -SIZE.y / 2.0 + 40.0, 2.0);

fn spawn_level(sprites: Res<Sprites>, mut commands: Commands) {
    commands.spawn((
        Sky,
        Sprite::from_color(SKY, SIZE),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    let ground = Vec2::new(SIZE.x, 32.0);
    commands.spawn((
        Solid,
        Collider(ground),
        Sprite {
            custom_size: Some(ground),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0,
            },
            ..Sprite::from_image(sprites.ground.clone())
        },
        Transform::from_xyz(0.0, -SIZE.y / 2.0 + 16.0, 0.0),
    ));
    commands.spawn((
        Mario,
        Velocity::default(),
        Collider(Vec2::new(12.0, 16.0)),
        Sprite::from_image(sprites.mario.clone()),
        Transform::from_translation(MARIO_START),
    ));
}

fn gravity(time: Res<Time>, mut bodies: Query<(&mut Transform, &mut Velocity), Without<Held>>) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut bodies {
        velocity.0.y = (velocity.0.y - GRAVITY * dt).max(-MAX_FALL);
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}

/// Pushes bodies out of solids: up onto them when falling, sideways (turning around) when
/// walking into them.
fn land(
    mut bodies: Query<(&mut Transform, &mut Velocity, &Collider), Without<Solid>>,
    solids: Query<(&Transform, &Collider), With<Solid>>,
) {
    for (mut body, mut velocity, collider) in &mut bodies {
        for (solid, size) in &solids {
            let offset = body.translation.truncate() - solid.translation.truncate();
            // Negative on both axes when they overlap: by how much.
            let gap = offset.abs() - (collider.0 + size.0) / 2.0;
            if gap.x >= 0.0 || gap.y >= 0.0 {
                continue;
            }
            if gap.y > gap.x {
                // Less overlap vertically: on top of it (or bumping it from below).
                body.translation.y -= offset.y.signum() * gap.y;
                velocity.0.y = 0.0;
            } else {
                body.translation.x -= offset.x.signum() * gap.x;
                velocity.0.x = offset.x.signum() * velocity.0.x.abs();
            }
        }
    }
}

/// Goombas walk, turning around at the edges of the screen.
fn walk(mut goombas: Query<(&Transform, &mut Velocity), With<Goomba>>) {
    let edge = SIZE.x / 2.0 - 8.0;
    for (transform, mut velocity) in &mut goombas {
        if transform.translation.x < -edge {
            velocity.0.x = GOOMBA_SPEED;
        } else if transform.translation.x > edge {
            velocity.0.x = -GOOMBA_SPEED;
        }
    }
}

/// Mario falling onto a goomba squishes it and bounces.
fn stomp(
    mut marios: Query<(&Transform, &mut Velocity), (With<Mario>, Without<Held>)>,
    mut goombas: Query<
        (Entity, &mut Transform, &mut Sprite),
        (With<Goomba>, Without<Mario>, Without<Squished>),
    >,
    mut commands: Commands,
) {
    for (mario, mut velocity) in &mut marios {
        if velocity.0.y >= 0.0 {
            continue;
        }
        for (goomba, mut transform, mut sprite) in &mut goombas {
            let offset = mario.translation - transform.translation;
            if offset.x.abs() < 14.0 && (4.0..18.0).contains(&offset.y) {
                sprite.custom_size = Some(Vec2::new(16.0, 8.0));
                transform.translation.y -= 4.0;
                commands
                    .entity(goomba)
                    .remove::<Velocity>()
                    .insert(Squished(Timer::from_seconds(0.5, TimerMode::Once)));
                velocity.0.y = 260.0;
            }
        }
    }
}

fn squash(time: Res<Time>, mut squished: Query<(Entity, &mut Squished)>, mut commands: Commands) {
    for (goomba, mut timer) in &mut squished {
        if timer.0.tick(time.delta()).is_finished() {
            commands.entity(goomba).despawn();
        }
    }
}

/// Goombas that fall off the screen are gone; Mario starts over.
fn fall_out(
    mut bodies: Query<(Entity, &mut Transform, &mut Velocity, Has<Mario>)>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut velocity, mario) in &mut bodies {
        if transform.translation.y > -SIZE.y {
            continue;
        }
        if mario {
            transform.translation = MARIO_START;
            velocity.0 = Vec2::ZERO;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_goombas(
    time: Res<Time>,
    mut timer: ResMut<GoombaTimer>,
    goombas: Query<(), With<Goomba>>,
    sprites: Res<Sprites>,
    mut commands: Commands,
) {
    if !timer.0.tick(time.delta()).just_finished() || goombas.iter().count() >= 3 {
        return;
    }
    commands.spawn((
        Goomba,
        Velocity(Vec2::new(-GOOMBA_SPEED, 0.0)),
        Collider(Vec2::splat(16.0)),
        Sprite::from_image(sprites.goomba.clone()),
        Transform::from_xyz(130.0, 90.0, 1.0),
    ));
}

fn spawn_platform(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    sprites: Res<Sprites>,
    mut commands: Commands,
) {
    // Only clicks on the sky: not on Mario, a goomba or another platform.
    if !sky.contains(click.entity) {
        return;
    }
    let Some(at) = click.hit.position else {
        return;
    };
    // Snap to the level's 16-unit grid.
    let at = (at.truncate() / 16.0).floor() * 16.0 + 8.0;
    commands.spawn((
        Solid,
        Collider(PLATFORM),
        Sprite {
            custom_size: Some(PLATFORM),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: false,
                stretch_value: 1.0,
            },
            ..Sprite::from_image(sprites.brick.clone())
        },
        Transform::from_translation(at.extend(0.0)),
    ));
}

fn grab(drag: On<PointerDragStart>, marios: Query<(), With<Mario>>, mut commands: Commands) {
    if marios.contains(drag.entity) {
        commands.entity(drag.entity).insert(Held);
    }
}

fn drag_mario(drag: On<PointerDrag>, mut marios: Query<&mut Transform, With<Mario>>) {
    if let Ok(mut transform) = marios.get_mut(drag.entity) {
        // Screen y points down, world y up.
        transform.translation += Vec3::new(drag.delta.x, -drag.delta.y, 0.0);
    }
}

fn release(
    drag: On<PointerDragEnd>,
    mut marios: Query<&mut Velocity, With<Mario>>,
    mut commands: Commands,
) {
    if let Ok(mut velocity) = marios.get_mut(drag.entity) {
        velocity.0 = Vec2::ZERO;
        commands.entity(drag.entity).remove::<Held>();
    }
}

// ---------------------------------------------------------------------------
// Pixel art
// ---------------------------------------------------------------------------

const MARIO: &[&str] = &[
    "...RRRRR....",
    "..RRRRRRRRR.",
    "..HHHSSKS...",
    ".HSHSSSKSSS.",
    ".HSHHSSSKSSS",
    ".HHSSSSKKKK.",
    "...SSSSSSS..",
    "..RRBRRR....",
    ".RRRBRRBRRR.",
    "RRRRBBBBRRRR",
    "SSRBYBBYBRSS",
    "SSSBBBBBBSSS",
    "SSBBBBBBBBSS",
    "..BBB..BBB..",
    ".HHH....HHH.",
    "HHHH....HHHH",
];

const GOOMBA: &[&str] = &[
    "......GGGG......",
    ".....GGGGGG.....",
    "....GGGGGGGG....",
    "...GGGGGGGGGG...",
    "..GWWGGGGGGWWG..",
    ".GGWKWGGGGWKWGG.",
    ".GGWKWWWWWWKWGG.",
    "GGGWWWGGGGWWWGGG",
    "GGGGGGGGGGGGGGGG",
    ".GGGGGGGGGGGGGG.",
    "....TTTTTTTT....",
    "...TTTTTTTTTT...",
    "...TTTTTTTTTT...",
    "..KKKTTTTTTKKK..",
    ".KKKKKK..KKKKKK.",
    ".KKKKK....KKKKK.",
];

const BRICK: &[&str] = &[
    "OOOOOOOMOOOOOOOM",
    "OOOOOOOMOOOOOOOM",
    "OOOOOOOMOOOOOOOM",
    "MMMMMMMMMMMMMMMM",
    "OOOMOOOOOOOMOOOO",
    "OOOMOOOOOOOMOOOO",
    "OOOMOOOOOOOMOOOO",
    "MMMMMMMMMMMMMMMM",
    "OOOOOOOMOOOOOOOM",
    "OOOOOOOMOOOOOOOM",
    "OOOOOOOMOOOOOOOM",
    "MMMMMMMMMMMMMMMM",
    "OOOMOOOOOOOMOOOO",
    "OOOMOOOOOOOMOOOO",
    "OOOMOOOOOOOMOOOO",
    "MMMMMMMMMMMMMMMM",
];

const GROUND: &[&str] = &[
    "LLLLLLLLLLLLLLLM",
    "LOOOOOOOOOOOOOOM",
    "LOOOOOOOOOOOOOOM",
    "LOOOMOOOOOOOOOOM",
    "LOOOOMOOOOOOOOOM",
    "LOOOOOOOOOOOOOOM",
    "LOOOOOOOOOOOOOOM",
    "LOOOOOOOOOOMOOOM",
    "LOOOOOOOOOOOMOOM",
    "LOOOOOOOOOOOOOOM",
    "LOOOOOOOOOOOOOOM",
    "LOOMOOOOOOOOOOOM",
    "LOOOMOOOOOOOOOOM",
    "LOOOOOOOOOOOOOOM",
    "LOOOOOOOOOOOOOOM",
    "MMMMMMMMMMMMMMMM",
];

fn palette(c: char) -> [u8; 4] {
    match c {
        'R' => [0xd8, 0x28, 0x00, 0xff],
        'H' => [0x88, 0x70, 0x00, 0xff],
        'S' => [0xfc, 0x98, 0x38, 0xff],
        'K' => [0x10, 0x10, 0x10, 0xff],
        'B' => [0x20, 0x38, 0xec, 0xff],
        'Y' => [0xfc, 0xd8, 0x00, 0xff],
        'G' => [0xa0, 0x50, 0x20, 0xff],
        'W' => [0xfc, 0xfc, 0xfc, 0xff],
        'T' => [0xf0, 0xc0, 0x90, 0xff],
        'O' => [0xc8, 0x4c, 0x0c, 0xff],
        'L' => [0xfc, 0xbc, 0xb0, 0xff],
        'M' => [0x50, 0x20, 0x08, 0xff],
        _ => [0, 0, 0, 0],
    }
}

/// An image from rows of palette letters (`.` is transparent), sampled without smoothing.
fn pixel_art(rows: &[&str]) -> Image {
    let width = rows[0].len() as u32;
    let data = rows
        .iter()
        .flat_map(|row| row.chars())
        .flat_map(palette)
        .collect();
    let mut image = Image::new(
        Extent3d {
            width,
            height: rows.len() as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn create_sprites(images: Option<ResMut<Assets<Image>>>, mut commands: Commands) {
    let Some(mut images) = images else {
        return;
    };
    commands.insert_resource(Sprites {
        mario: images.add(pixel_art(MARIO)),
        goomba: images.add(pixel_art(GOOMBA)),
        brick: images.add(pixel_art(BRICK)),
        ground: images.add(pixel_art(GROUND)),
    });
}

#[cfg(test)]
mod tests {
    use bevy::{
        camera::NormalizedRenderTarget,
        picking::{
            backend::HitData,
            events::Pointer,
            pointer::{Location, PointerButton, PointerId},
        },
    };

    use std::time::Duration;

    use super::*;
    use crate::runtime::PreviewRuntime;

    fn preview(code: &str) -> PreviewRuntime {
        PreviewRuntime::new(&EXAMPLE, setup(&World::new()), code, None)
    }

    fn run(preview: &mut PreviewRuntime, seconds: f32) {
        for _ in 0..(seconds * 60.0) as usize {
            preview.advance(1.0 / 60.0);
        }
    }

    fn count<F: bevy::ecs::query::QueryFilter>(preview: &mut PreviewRuntime) -> usize {
        preview
            .world
            .query_filtered::<(), F>()
            .iter(&preview.world)
            .count()
    }

    fn single<F: bevy::ecs::query::QueryFilter>(preview: &mut PreviewRuntime) -> Entity {
        preview
            .world
            .query_filtered::<Entity, F>()
            .single(&preview.world)
            .unwrap()
    }

    fn mario_at(preview: &mut PreviewRuntime) -> Vec3 {
        let mario = single::<With<Mario>>(preview);
        preview.world.get::<Transform>(mario).unwrap().translation
    }

    fn pointer_at(at: Vec2) -> (Pointer, HitData) {
        let screen = NormalizedRenderTarget::None {
            width: 320,
            height: 160,
        };
        (
            Pointer::new(
                PointerId::Mouse,
                Location {
                    target: screen,
                    position: Vec2::ZERO,
                },
            ),
            HitData::new(Entity::PLACEHOLDER, 0.0, Some(at.extend(0.0)), None),
        )
    }

    #[test]
    fn mario_stands_on_the_ground() {
        let mut preview = preview("");
        run(&mut preview, 1.0);
        assert!((mario_at(&mut preview).y - MARIO_START.y).abs() < 0.5);
        assert_eq!(
            count::<With<Goomba>>(&mut preview),
            0,
            "goombas come with their block"
        );
    }

    #[test]
    fn goombas_spawn_walk_and_get_stomped() {
        let mut preview = preview("spawn_goombas");
        run(&mut preview, 2.5);
        assert_eq!(count::<With<Goomba>>(&mut preview), 1);
        run(&mut preview, 2.0);
        // Two by now; the first has walked furthest.
        let at = preview
            .world
            .query_filtered::<&Transform, With<Goomba>>()
            .iter(&preview.world)
            .map(|t| t.translation)
            .min_by(|a, b| a.x.total_cmp(&b.x))
            .unwrap();
        assert!(
            at.x < 120.0 && (at.y - MARIO_START.y).abs() < 0.5,
            "walking on the ground: {at}"
        );
        // Drop Mario on it.
        let mario = single::<With<Mario>>(&mut preview);
        preview
            .world
            .get_mut::<Transform>(mario)
            .unwrap()
            .translation = at + Vec3::new(0.0, 30.0, 1.0);
        run(&mut preview, 0.3);
        assert_eq!(count::<With<Squished>>(&mut preview), 1);
        run(&mut preview, 0.5);
        assert_eq!(
            count::<With<Goomba>>(&mut preview),
            1,
            "squished, then the next one"
        );
    }

    #[test]
    fn clicking_the_sky_places_a_platform() {
        let mut preview = preview("fn spawn_platform");
        let sky = single::<With<Sky>>(&mut preview);
        let (pointer, hit) = pointer_at(Vec2::new(3.0, 20.0));
        preview.trigger(PointerClick {
            entity: sky,
            pointer,
            button: PointerButton::Primary,
            hit,
            duration: Duration::ZERO,
            count: 1,
        });
        let solids: Vec<Vec3> = preview
            .world
            .query_filtered::<&Transform, With<Solid>>()
            .iter(&preview.world)
            .map(|t| t.translation)
            .collect();
        assert_eq!(solids.len(), 2, "the ground and the platform");
        assert!(
            solids.contains(&Vec3::new(8.0, 24.0, 0.0)),
            "on the grid: {solids:?}"
        );
        // A click on Mario is not on the sky.
        let mario = single::<With<Mario>>(&mut preview);
        let (pointer, hit) = pointer_at(Vec2::ZERO);
        preview.trigger(PointerClick {
            entity: mario,
            pointer,
            button: PointerButton::Primary,
            hit,
            duration: Duration::ZERO,
            count: 1,
        });
        assert_eq!(count::<With<Solid>>(&mut preview), 2);
    }

    #[test]
    fn dragging_moves_mario_and_dropping_lets_him_fall() {
        let mut preview = preview("grab drag_mario release");
        let mario = single::<With<Mario>>(&mut preview);
        let (pointer, hit) = pointer_at(Vec2::ZERO);
        preview.trigger(PointerDragStart {
            entity: mario,
            pointer: pointer.clone(),
            button: PointerButton::Primary,
            hit,
        });
        preview.trigger(PointerDrag {
            entity: mario,
            pointer: pointer.clone(),
            button: PointerButton::Primary,
            distance: Vec2::new(10.0, -60.0),
            delta: Vec2::new(10.0, -60.0),
        });
        let held = mario_at(&mut preview);
        assert_eq!(held, MARIO_START + Vec3::new(10.0, 60.0, 0.0));
        run(&mut preview, 0.5);
        assert_eq!(mario_at(&mut preview), held, "held in the air");
        preview.trigger(PointerDragEnd {
            entity: mario,
            pointer,
            button: PointerButton::Primary,
            distance: Vec2::new(10.0, -60.0),
        });
        run(&mut preview, 1.0);
        assert!(
            (mario_at(&mut preview).y - MARIO_START.y).abs() < 0.5,
            "back on the ground"
        );
    }

    #[test]
    fn pixel_art_rows_are_even() {
        for art in [MARIO, GOOMBA, BRICK, GROUND] {
            assert!(art.iter().all(|row| row.len() == art[0].len()));
            assert_eq!(pixel_art(art).size().y as usize, art.len());
        }
    }
}

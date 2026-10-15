//! `Sprite animation.md`: a slime on a four-frame sprite sheet, and a timer that flips frames.
//! `Texture atlases.md` steps through the frames with a button.

use bevy::prelude::*;

use super::stage::{self, Sky, sheet, spawn_sky};
use crate::example::{Action, Example, Setup, observer, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky)],
    functions: &[
        startup!(spawn_slime),
        update!(animate),
        observer!(change_speed),
    ],
    tick: None,
    actions: &[Action {
        name: "next_frame",
        label: "Next frame",
        run: next_frame,
    }],
    also: &[],
    view: stage::view(),
};

fn setup(app: &World) -> Setup {
    let slime = app
        .get_resource::<SlimeSheet>()
        .cloned()
        .unwrap_or_default();
    Box::new(move |world| {
        world.insert_resource(slime.clone());
    })
}

/// Draws the sheet once for the app (previews share it).
pub fn plugin(app: &mut App) {
    app.add_systems(Startup, create_sheet);
}

/// The sheet and how it is cut into frames.
#[derive(Resource, Clone, Default)]
struct SlimeSheet {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn spawn_slime(sheet: Res<SlimeSheet>, mut commands: Commands) {
    let indices = AnimationIndices { first: 0, last: 3 };
    commands.spawn((
        Sprite::from_atlas_image(
            sheet.image.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: indices.first,
            },
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
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

/// `Texture atlases.md`'s button: shows the next frame of the sheet.
fn next_frame(world: &mut World) {
    let mut sprites = world.query::<&mut Sprite>();
    for mut sprite in sprites.iter_mut(world) {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = (atlas.index + 1) % 4;
        }
    }
}

/// Click the slime to make it faster, click the sky to slow it down again.
fn change_speed(
    click: On<PointerClick>,
    sky: Query<(), With<Sky>>,
    mut timers: Query<&mut AnimationTimer>,
) {
    let seconds = if sky.contains(click.entity) {
        0.2
    } else {
        0.06
    };
    for mut timer in &mut timers {
        timer.set_duration(std::time::Duration::from_secs_f32(seconds));
    }
}

const FRAME_0: &[&str] = &[
    "..........",
    "..gggggg..",
    ".gGGGGGGg.",
    ".GWKGGWKG.",
    ".GGGGGGGG.",
    ".GGGGGGGG.",
    ".DGGGGGGD.",
    "..DDDDDD..",
];
const FRAME_1: &[&str] = &[
    "..gggggg..",
    ".gGGGGGGg.",
    ".GWKGGWKG.",
    ".GGGGGGGG.",
    ".GGGGGGGG.",
    "..GGGGGG..",
    "..DDDDDD..",
    "...DDDD...",
];
const FRAME_3: &[&str] = &[
    "..........",
    "..........",
    "..........",
    "..gggggg..",
    ".GWKGGWKG.",
    "GGGGGGGGGG",
    "DGGGGGGGGD",
    ".DDDDDDDD.",
];

fn colours(c: char) -> [u8; 4] {
    match c {
        'g' => [0x9a, 0xee, 0x96, 0xff],
        'G' => [0x4c, 0xc2, 0x58, 0xff],
        'D' => [0x2a, 0x84, 0x3c, 0xff],
        'W' => [0xff, 0xff, 0xff, 0xff],
        'K' => [0x10, 0x14, 0x1c, 0xff],
        _ => [0, 0, 0, 0],
    }
}

fn create_sheet(
    images: Option<ResMut<Assets<Image>>>,
    layouts: Option<ResMut<Assets<TextureAtlasLayout>>>,
    mut commands: Commands,
) {
    let (Some(mut images), Some(mut layouts)) = (images, layouts) else {
        return;
    };
    commands.insert_resource(SlimeSheet {
        image: images.add(sheet(&[FRAME_0, FRAME_1, FRAME_0, FRAME_3], colours)),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(10, 8),
            4,
            1,
            None,
            None,
        )),
    });
}

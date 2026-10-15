//! `3D.md`: the reader's voxel model ("Make it yours"), drawn from above at an angle with one
//! sprite per cube; a click turns it a quarter turn.

use bevy::prelude::*;

use super::stage::{self, spawn_sky};
use crate::{
    example::{Example, Setup, observer, startup, update},
    yours::{VOXELS, Yours},
};
use bevsidian_ui::color::PIXEL_PALETTE;

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky)],
    functions: &[update!(build), observer!(turn)],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

/// The reader's model and the cube picture, from the app.
fn setup(app: &World) -> Setup {
    let (cells, cube) = app.get_resource::<Yours>().map_or_else(
        || (Vec::new(), Handle::default()),
        |yours| {
            let cells = yours
                .filled()
                .into_iter()
                .map(|(at, colour)| (at, PIXEL_PALETTE[colour]))
                .collect();
            (cells, yours.cube.clone())
        },
    );
    Box::new(move |world| {
        world.insert_resource(Voxels(cells.clone()));
        world.insert_resource(Cube(cube.clone()));
    })
}

/// The model: a colour for each filled cell, `x` and `y` across, `z` up.
#[derive(Resource)]
struct Voxels(Vec<(IVec3, Color)>);

/// One cube, white with shaded sides: tinted, it draws any colour.
#[derive(Resource)]
struct Cube(Handle<Image>);

#[derive(Component)]
struct Block;

fn build(
    voxels: Res<Voxels>,
    cube: Res<Cube>,
    blocks: Query<Entity, With<Block>>,
    mut commands: Commands,
) {
    if !voxels.is_changed() {
        return;
    }
    for block in &blocks {
        commands.entity(block).despawn();
    }
    // Further cubes first, lower ones first: each is drawn over those before it.
    let mut cells = voxels.0.clone();
    cells.sort_by_key(|(at, _)| (at.x + at.y, at.z));
    for (order, (at, color)) in cells.into_iter().enumerate() {
        // `x` goes right and down the screen, `y` left and down, `z` up.
        let x = (at.x - at.y) as f32 * 8.0;
        let y = -(at.x + at.y) as f32 * 4.0 + at.z as f32 * 8.0;
        commands.spawn((
            Block,
            Sprite {
                image: cube.0.clone(),
                color,
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            Transform::from_xyz(x, y + 8.0, order as f32 / 16.0),
        ));
    }
}

fn turn(_click: On<PointerClick>, mut voxels: ResMut<Voxels>) {
    // A quarter turn about the middle: (x, y) becomes (y, 7 - x).
    for (at, _) in &mut voxels.0 {
        *at = IVec3::new(at.y, 7 - at.x, at.z);
    }
}

const _: () = assert!(VOXELS == 8, "`turn` turns an 8-wide model");

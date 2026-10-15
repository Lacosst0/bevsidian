//! "Make it yours": a pixel sprite, a voxel model and sound effects the reader makes. They
//! are kept as assets in the app's world and turn up in the examples: the player-like sprites
//! (the Dodge player, the ship, the agent, the balls) as the reader's sprite, rocks and guards
//! as their model; the examples' code is unchanged, the game view swaps the pictures. Sounds
//! are the examples' own `Sounds` resources, handed the reader's: an example plays one with an
//! `AudioPlayer` as any game would, and [`play_sounds`] plays it in the app.

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use bevy::audio::{AudioPlayer, AudioSource, PlaybackMode, PlaybackSettings};

use crate::{
    runtime::PreviewRuntime,
    sound::{self, Sfx, Slot},
};
use bevsidian_core::nav::{Nav, Studio};
use bevsidian_ui::color::PIXEL_PALETTE;

/// The sprite is `SPRITE` × `SPRITE` pixels; the model `VOXELS` cells on each side.
pub const SPRITE: usize = 16;
pub const VOXELS: usize = SIDE as usize;
/// [`VOXELS`] and [`CUBE`] for screen maths.
const SIDE: i32 = 8;
const STAMP: i32 = 8;

/// One pixel or voxel: 0 is empty, `n` is colour `n - 1` of [`PIXEL_PALETTE`].
pub type Cell = u8;

/// The reader's sprite and model, and the images drawn from them.
#[derive(Resource)]
pub struct Yours {
    /// Pixels, row by row from the top.
    pub pixels: Vec<Cell>,
    /// Voxels, index `x + y * VOXELS + z * VOXELS²`, `z` up.
    pub voxels: Vec<Cell>,
    /// Whether the reader made each: until then the examples keep their own pictures.
    pub made_sprite: bool,
    pub made_voxels: bool,
    pub sprite: Handle<Image>,
    pub model: Handle<Image>,
    /// One white cube with shaded sides, which the voxel example tints.
    pub cube: Handle<Image>,
    /// The sound of each [`Slot`], whether the reader changed them, and the sounds.
    pub sounds: Vec<Sfx>,
    pub made_sounds: bool,
    pub audio: Vec<Handle<AudioSource>>,
    /// Bumped whenever the model changes (previews that build from it start again).
    pub revision: u64,
}

impl Yours {
    pub fn voxel(&self, x: usize, y: usize, z: usize) -> Cell {
        self.voxels[x + y * VOXELS + z * VOXELS * VOXELS]
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, cell: Cell) {
        self.voxels[x + y * VOXELS + z * VOXELS * VOXELS] = cell;
        self.made_voxels = true;
    }

    /// The sound the examples play for `slot`.
    pub fn sound(&self, slot: Slot) -> Handle<AudioSource> {
        self.audio.get(slot.index()).cloned().unwrap_or_default()
    }

    pub fn set_sound(&mut self, slot: Slot, sfx: Sfx) {
        self.sounds[slot.index()] = sfx.clamped();
        self.made_sounds = true;
    }

    pub fn set_pixel(&mut self, index: usize, cell: Cell) {
        self.pixels[index] = cell;
        self.made_sprite = true;
    }

    /// The filled voxels, as (cell, colour index).
    pub fn filled(&self) -> Vec<(IVec3, usize)> {
        filled(&self.voxels)
            .into_iter()
            .map(|(at, cell)| (at, usize::from(cell - 1)))
            .collect()
    }
}

/// Cells as text, one character each (`0` empty, then `1`–`9`, `a`–`g`), to store.
pub fn encode(cells: &[Cell]) -> String {
    cells
        .iter()
        .map(|&c| char::from_digit(u32::from(c), 17).unwrap_or('0'))
        .collect()
}

/// Cells from [`encode`]d text, or `None` when it does not fit `len`.
pub fn decode(text: &str, len: usize) -> Option<Vec<Cell>> {
    let cells: Vec<Cell> = text
        .trim()
        .chars()
        .map(|c| c.to_digit(17).and_then(|d| Cell::try_from(d).ok()))
        .collect::<Option<_>>()?;
    (cells.len() == len).then_some(cells)
}

/// The starting sprite: a little ship (dark blue outline, blue hull, white cockpit, a
/// yellow and orange flame), to draw over.
pub fn default_pixels() -> Vec<Cell> {
    const ROWS: [&str; SPRITE] = [
        "................",
        ".......22.......",
        "......2dd2......",
        "......2dd2......",
        ".....22dd22.....",
        ".....2d88d2.....",
        "....22d88d22....",
        "...2dddddddd2...",
        "..2dddddddddd2..",
        ".2dd2dddddd2dd2.",
        ".2d2.2dddd2.2d2.",
        ".22..2dddd2..22.",
        ".....22dd22.....",
        "......abba......",
        ".......aa.......",
        "................",
    ];
    ROWS.iter()
        .flat_map(|row| row.chars())
        .map(|c| c.to_digit(17).map_or(0, |d| d as Cell))
        .collect()
}

/// The starting model: a little house, to build on.
pub fn default_voxels() -> Vec<Cell> {
    let mut cells = vec![0; VOXELS * VOXELS * VOXELS];
    let mut set = |x: usize, y: usize, z: usize, c: Cell| {
        cells[x + y * VOXELS + z * VOXELS * VOXELS] = c;
    };
    for x in 2..6 {
        for y in 2..6 {
            set(x, y, 0, 6); // grey floor
            let wall = x == 2 || x == 5 || y == 2 || y == 5;
            if wall {
                set(x, y, 1, 16); // peach walls
                set(x, y, 2, 16);
            }
            set(x, y, 3, 9); // red roof
        }
    }
    for x in 3..5 {
        for y in 3..5 {
            set(x, y, 4, 9);
        }
    }
    set(3, 2, 1, 5); // a brown door
    cells
}

impl Default for Yours {
    fn default() -> Self {
        Self {
            pixels: default_pixels(),
            voxels: default_voxels(),
            made_sprite: false,
            made_voxels: false,
            sprite: Handle::default(),
            model: Handle::default(),
            cube: Handle::default(),
            sounds: Slot::ALL.iter().map(|s| s.default_sound()).collect(),
            made_sounds: false,
            audio: Vec::new(),
            revision: 0,
        }
    }
}

/// The examples whose pictures become the reader's, by the name of a component on the
/// sprite: the player-like ones show the sprite, the obstacles and enemies the model.
pub const SPRITE_OF: [&str; 4] = ["Player", "Ship", "Agent", "Ball"];
pub const MODEL_OF: [&str; 2] = ["Rock", "Guard"];

/// What "Make it yours" a preview of the example offers, if any.
pub fn studio_for(example: &str) -> Option<Studio> {
    match example {
        "voxels" => Some(Studio::Voxels),
        "sounds" => Some(Studio::Sounds),
        "dodge" | "movement" | "ai" | "pathfinding" | "sprites" | "bouncing" => {
            Some(Studio::Sprite)
        }
        _ => None,
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Yours>()
        .add_systems(PreStartup, create_images)
        .add_systems(PostUpdate, (redraw, remake_sounds));
}

fn create_images(
    mut yours: ResMut<Yours>,
    images: Option<ResMut<Assets<Image>>>,
    audio: Option<ResMut<Assets<AudioSource>>>,
) {
    if let Some(mut audio) = audio {
        yours.audio = yours
            .sounds
            .iter()
            .map(|sfx| audio.add(sound::audio(sfx)))
            .collect();
    }
    let Some(mut images) = images else {
        return;
    };
    yours.sprite = images.add(sprite_image(&yours.pixels));
    yours.model = images.add(model_image(&yours.voxels));
    yours.cube = images.add(cube_image());
}

/// Remakes the sounds that changed.
fn remake_sounds(
    yours: Res<Yours>,
    audio: Option<ResMut<Assets<AudioSource>>>,
    mut last: Local<Vec<Sfx>>,
) {
    let Some(mut audio) = audio else {
        return;
    };
    if *last == yours.sounds {
        return;
    }
    for (index, sfx) in yours.sounds.iter().enumerate() {
        if last.get(index) != Some(sfx)
            && let Some(handle) = yours.audio.get(index)
            && let Some(mut source) = audio.get_mut(handle)
        {
            *source = sound::audio(sfx);
        }
    }
    last.clone_from(&yours.sounds);
}

/// A sound an example played, playing in the app.
#[derive(Component)]
pub struct PreviewSound;

/// Most sounds playing at once: a pile of bouncing balls stays pleasant.
const MOST_SOUNDS: usize = 6;

/// Plays in the app the sounds the examples' worlds start: each `AudioPlayer` an example
/// spawns is taken out of its world and spawned here, where Bevy's audio plays it (unless the
/// reader turned sound off).
pub fn play_sounds(
    mut previews: Query<&mut PreviewRuntime>,
    playing: Query<(), With<PreviewSound>>,
    nav: Res<Nav>,
    mut commands: Commands,
) {
    let mut count = playing.iter().count();
    let on = nav.sound.get_untracked();
    for mut runtime in &mut previews {
        let world = &mut runtime.bypass_change_detection().world;
        let mut players = world.query::<(Entity, &AudioPlayer, Option<&PlaybackSettings>)>();
        let started: Vec<(Entity, Handle<AudioSource>, PlaybackSettings)> = players
            .iter(world)
            .map(|(entity, player, settings)| {
                (
                    entity,
                    player.0.clone(),
                    settings.copied().unwrap_or_default(),
                )
            })
            .collect();
        for (entity, handle, mut settings) in started {
            world.despawn(entity);
            if !on || count >= MOST_SOUNDS {
                continue;
            }
            // Played once, the entity goes when it ends.
            if matches!(settings.mode, PlaybackMode::Once | PlaybackMode::Remove) {
                settings.mode = PlaybackMode::Despawn;
            }
            commands.spawn((PreviewSound, AudioPlayer(handle), settings));
            count += 1;
        }
    }
}

/// Redraws the images when the sprite or model changes.
fn redraw(
    mut yours: ResMut<Yours>,
    images: Option<ResMut<Assets<Image>>>,
    mut last: Local<Option<(Vec<Cell>, Vec<Cell>)>>,
) {
    let Some(mut images) = images else {
        return;
    };
    let (pixels_changed, voxels_changed) = match last.as_ref() {
        Some((pixels, voxels)) => (*pixels != yours.pixels, *voxels != yours.voxels),
        None => (true, true),
    };
    if !pixels_changed && !voxels_changed {
        return;
    }
    if pixels_changed && let Some(mut image) = images.get_mut(&yours.sprite) {
        *image = sprite_image(&yours.pixels);
    }
    if voxels_changed {
        if let Some(mut image) = images.get_mut(&yours.model) {
            *image = model_image(&yours.voxels);
        }
        yours.revision += 1;
    }
    *last = Some((yours.pixels.clone(), yours.voxels.clone()));
}

fn rgba(cell: Cell, shade: f32) -> [u8; 4] {
    if cell == 0 {
        return [0; 4];
    }
    let colour = PIXEL_PALETTE[usize::from(cell - 1) % PIXEL_PALETTE.len()].to_srgba();
    let channel = |v: f32| (v * shade * 255.0).round().clamp(0.0, 255.0) as u8;
    [
        channel(colour.red),
        channel(colour.green),
        channel(colour.blue),
        255,
    ]
}

fn image(width: usize, height: usize, data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
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

pub fn sprite_image(pixels: &[Cell]) -> Image {
    let data = pixels.iter().flat_map(|&c| rgba(c, 1.0)).collect();
    image(SPRITE, SPRITE, data)
}

/// The faces of a cube drawn from above at an angle, in a [`CUBE`] × [`CUBE`] stamp.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    None,
    Top,
    Left,
    Right,
}

/// A cube's stamp is this many pixels square: a 2:1 diamond on top, the sides below.
pub const CUBE: usize = STAMP as usize;

/// The filled cells of a model, with their cells.
fn filled(voxels: &[Cell]) -> Vec<(IVec3, Cell)> {
    let mut out = Vec::new();
    for z in 0..SIDE {
        for y in 0..SIDE {
            for x in 0..SIDE {
                let cell = voxels[(x + y * SIDE + z * SIDE * SIDE) as usize];
                if cell != 0 {
                    out.push((IVec3::new(x, y, z), cell));
                }
            }
        }
    }
    out
}

/// How light each face is: the top in full light, the sides in shade.
pub fn shade(face: Face) -> f32 {
    match face {
        Face::Top | Face::None => 1.0,
        Face::Left => 0.78,
        Face::Right => 0.58,
    }
}

/// Which face of the cube covers pixel (`col`, `row`) of its stamp.
pub fn face(col: usize, row: usize) -> Face {
    let half = CUBE as f32 / 2.0;
    let (x, y) = (col as f32 + 0.5 - half, row as f32 + 0.5);
    // The top diamond: centred `half / 2` down, `half` wide each way, `half / 2` tall.
    let across = x.abs() / half;
    if across > 1.0 {
        return Face::None;
    }
    let top = half / 2.0;
    if (y - top).abs() / top + across <= 1.0 {
        Face::Top
    } else if y > top && y - half <= top * (1.0 - across) + top {
        if x < 0.0 { Face::Left } else { Face::Right }
    } else {
        Face::None
    }
}

/// The model drawn from above at an angle (isometric), cropped to what is drawn.
pub fn model_image(voxels: &[Cell]) -> Image {
    // Screen position of a voxel's stamp: `x` goes right and down, `y` left and down, `z`
    // up; cells further back are drawn first, so nearer ones cover them.
    let n = SIDE;
    let step = STAMP / 2;
    let width = (2 * n * step) as usize;
    let height = (2 * n * step / 2 + n * step + step) as usize;
    let mut data = vec![0u8; width * height * 4];
    let origin = IVec2::new((n - 1) * step, (n - 1) * step);
    let mut cells = filled(voxels);
    cells.sort_by_key(|(at, _)| (at.x + at.y, at.z));
    for (at, cell) in cells {
        let left = origin.x + (at.x - at.y) * step;
        let top = origin.y + (at.x + at.y) * step / 2 - at.z * step;
        for row in 0..STAMP {
            for col in 0..STAMP {
                let face = face(col as usize, row as usize);
                if face == Face::None {
                    continue;
                }
                let (px, py) = (left + col, top + row);
                if px < 0 || py < 0 || px as usize >= width || py as usize >= height {
                    continue;
                }
                let i = (py as usize * width + px as usize) * 4;
                data[i..i + 4].copy_from_slice(&rgba(cell, shade(face)));
            }
        }
    }
    crop(width, height, &data)
}

/// The smallest image holding every drawn pixel (one clear pixel when nothing is).
fn crop(width: usize, height: usize, data: &[u8]) -> Image {
    let drawn = |x: usize, y: usize| data[(y * width + x) * 4 + 3] != 0;
    let mut bounds: Option<(usize, usize, usize, usize)> = None;
    for y in 0..height {
        for x in 0..width {
            if drawn(x, y) {
                let b = bounds.get_or_insert((x, y, x, y));
                *b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
            }
        }
    }
    let Some((x0, y0, x1, y1)) = bounds else {
        return image(1, 1, vec![0; 4]);
    };
    let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
    let mut out = Vec::with_capacity(w * h * 4);
    for y in y0..=y1 {
        out.extend_from_slice(&data[(y * width + x0) * 4..(y * width + x1 + 1) * 4]);
    }
    image(w, h, out)
}

/// One cube, white with shaded sides, to tint: what the voxel example draws each cell with.
pub fn cube_image() -> Image {
    let mut data = Vec::with_capacity(CUBE * CUBE * 4);
    for row in 0..CUBE {
        for col in 0..CUBE {
            let face = face(col, row);
            if face == Face::None {
                data.extend_from_slice(&[0; 4]);
            } else {
                let v = (shade(face) * 255.0).round() as u8;
                data.extend_from_slice(&[v, v, v, 255]);
            }
        }
    }
    image(CUBE, CUBE, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_survive_being_stored() {
        let pixels = default_pixels();
        assert_eq!(pixels.len(), SPRITE * SPRITE);
        assert!(pixels.iter().any(|&c| c != 0));
        assert_eq!(decode(&encode(&pixels), pixels.len()), Some(pixels));
        let voxels = default_voxels();
        assert_eq!(decode(&encode(&voxels), voxels.len()), Some(voxels));
        assert_eq!(decode("12", 3), None, "too short");
        assert_eq!(decode("1z2", 3), None, "not a cell");
    }

    #[test]
    fn a_cube_has_a_top_and_two_sides() {
        let faces: Vec<Face> = (0..CUBE)
            .flat_map(|row| (0..CUBE).map(move |col| face(col, row)))
            .collect();
        for wanted in [Face::Top, Face::Left, Face::Right] {
            assert!(faces.contains(&wanted), "{wanted:?}");
        }
        // The top is in the upper half, the sides below it.
        assert_eq!(face(CUBE / 2, 1), Face::Top);
        assert_eq!(face(0, CUBE - 3), Face::Left);
        assert_eq!(face(CUBE - 1, CUBE - 3), Face::Right);
    }

    #[test]
    fn the_model_is_cropped_to_what_is_drawn() {
        let mut voxels = vec![0; VOXELS * VOXELS * VOXELS];
        voxels[0] = 1;
        let one = model_image(&voxels);
        assert_eq!((one.width(), one.height()), (CUBE as u32, CUBE as u32));
        let empty = model_image(&vec![0; VOXELS * VOXELS * VOXELS]);
        assert_eq!((empty.width(), empty.height()), (1, 1));
    }
}

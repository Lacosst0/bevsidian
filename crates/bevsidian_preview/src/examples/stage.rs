//! What the 2D previews share: a small stage with a sky that catches clicks, and pixel art.

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::example::View;

/// The view: 320 × 160 world units, the origin in the middle.
pub const SIZE: Vec2 = Vec2::new(320.0, 160.0);
const BACKGROUND: Color = Color::srgb(0.11, 0.12, 0.17);

pub const fn view() -> View {
    View::Game {
        size: SIZE,
        background: BACKGROUND,
    }
}

/// The background: clicks that hit nothing else land on it.
#[derive(Component)]
pub struct Sky;

pub fn spawn_sky(mut commands: Commands) {
    commands.spawn((
        Sky,
        Sprite::from_color(BACKGROUND, SIZE),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
}

/// Pastel colours to pick from, by number.
pub fn colour(index: usize) -> Color {
    const COLOURS: [Color; 6] = [
        Color::srgb(0.95, 0.45, 0.4),
        Color::srgb(0.98, 0.75, 0.3),
        Color::srgb(0.55, 0.85, 0.45),
        Color::srgb(0.35, 0.75, 0.9),
        Color::srgb(0.6, 0.5, 0.95),
        Color::srgb(0.95, 0.55, 0.8),
    ];
    COLOURS[index % COLOURS.len()]
}

/// An image from rows of palette letters (`.` is transparent), sampled without smoothing.
pub fn pixel_art(rows: &[&str], palette: fn(char) -> [u8; 4]) -> Image {
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

/// Frames laid side by side: one sheet, as an artist would draw it.
pub fn sheet(frames: &[&[&str]], palette: fn(char) -> [u8; 4]) -> Image {
    let rows: Vec<String> = (0..frames[0].len())
        .map(|y| frames.iter().map(|frame| frame[y]).collect::<String>())
        .collect();
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    pixel_art(&rows, palette)
}

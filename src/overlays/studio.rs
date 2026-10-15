//! "Make it yours": draw a pixel sprite or build a voxel model. Both are saved, and the
//! examples play with them: the player and the ships become the sprite, rocks and guards
//! the model (see `bevsidian_preview::yours`). The sidebar's mark shows the sprite too.
//!
//! Opened from the brush and box buttons above the examples, and from the sidebar's mark.

use bevy::{picking::events::PointerOver, prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::view;

use bevsidian_core::nav::{Nav, Studio};
use bevsidian_preview::yours::{Cell, SPRITE, VOXELS, Yours, default_pixels, default_voxels};
use bevsidian_ui::{color::PIXEL_PALETTE, prelude::*};

pub fn plugin(app: &mut App) {
    app.init_resource::<Brush>()
        .add_systems(Update, (show_cells, show_pictures, show_mark));
}

/// What the brush paints, and which layer of the model is being built.
#[derive(Resource)]
struct Brush {
    cell: Cell,
    layer: usize,
    /// The grid cell last painted while the pointer is held: a fast drag paints the cells
    /// between too, so strokes have no gaps.
    last: Option<(usize, usize)>,
}

impl Default for Brush {
    fn default() -> Self {
        // Blue, on the floor.
        Self {
            cell: 13,
            layer: 0,
            last: None,
        }
    }
}

/// The grid cells from `from` to `to`, a straight line without gaps.
fn stroke(from: (usize, usize), to: (usize, usize)) -> Vec<(usize, usize)> {
    let (dx, dy) = (to.0 as f32 - from.0 as f32, to.1 as f32 - from.1 as f32);
    let steps = dx.abs().max(dy.abs()).max(1.0) as usize;
    (0..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            (
                (from.0 as f32 + dx * t).round() as usize,
                (from.1 as f32 + dy * t).round() as usize,
            )
        })
        .collect()
}

/// A pixel of the sprite, by index.
#[derive(Component, Clone, Copy, Default)]
struct PixelCell(usize);

/// A cell of the model's current layer, `x` across and `y` down the grid.
#[derive(Component, Clone, Copy, Default)]
struct VoxelCell(usize, usize);

/// A colour to paint with (0 erases).
#[derive(Component, Clone, Copy, Default)]
struct Swatch(Cell);

/// A layer of the model to build on.
#[derive(Component, Clone, Copy, Default)]
struct LayerChip(usize);

/// Shows the reader's sprite or model.
#[derive(Component, Clone, Copy)]
struct Picture(Studio);

impl Default for Picture {
    fn default() -> Self {
        Picture(Studio::Sprite)
    }
}

/// The reader's sprite over the sidebar's mark, once drawn.
#[derive(Component, Clone, Copy, Default)]
pub struct YourMark;

pub fn studio_dialog(nav: Nav) -> impl Scene {
    let open = nav.studio.clone();
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::DIALOG)
        Pickable::IGNORE
        Children [
            if $open.is_some() {
                let close = nav.studio.clone();
                @{scrim(move || close.set(None))}
                Node { justify_content: JustifyContent::Center, align_items: AlignItems::Start, padding: UiRect::top(Val::Percent(4.0)) }
                Children [ @{panel(nav.clone())} ]
            }
        ]
    }
}

fn panel(nav: Nav) -> impl Scene {
    let (close, done, tab) = (nav.clone(), nav.clone(), nav.studio.clone());
    let tabs = segmented(
        vec![
            (Some(Studio::Sprite), "Pixel sprite".to_owned()),
            (Some(Studio::Voxels), "Voxel model".to_owned()),
        ],
        nav.studio.clone(),
    );
    view! {
        @{dialog(640.0)}
        Node { row_gap: px(16) }
        Children [
            Node { flex_direction: FlexDirection::Column, row_gap: px(6) }
            Children [
                @{dialog_header("Make it yours", move || close.studio.set(None))}
                --
                @{muted("Draw a sprite or build a voxel model. It is saved, and the examples play with it: the player and the ships become your sprite, rocks and guards your model.")}
            ]
            --
            @{tabs}
            --
            if $tab == Some(Studio::Sprite) {
                @{sprite_editor()}
            }
            --
            if $tab == Some(Studio::Voxels) {
                @{voxel_editor()}
            }
            --
            @{dialog_footer()}
            Children [
                @{text_button(Variant::Primary, Size::Sm, "Done")}
                on(move |_: On<Activate>| done.studio.set(None))
            ]
        ]
    }
}

/// The 16 × 16 grid to draw on, the palette and the sprite at its size.
fn sprite_editor() -> impl Scene {
    let cells: Vec<Box<dyn Scene>> = (0..SPRITE * SPRITE)
        .map(|index| {
            Box::new(bsn! {
                Node { width: px(18), height: px(18) }
                BackgroundColor(color::CARD)
                PixelCell(index)
                on(paint_pixel)
                on(drag_pixel)
            }) as Box<dyn Scene>
        })
        .collect();
    bsn! {
        Node { column_gap: px(20), align_items: AlignItems::Start, flex_wrap: FlexWrap::Wrap, row_gap: px(16) }
        Children [
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px::<Vec<RepeatedGridTrack>>(SPRITE as u16, 18.0),
                border: UiRect::all(px(1)),
            }
            BorderColor::all(color::INPUT)
            Children [ {cells} ]
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(12) }
            Children [
                @{picture(Studio::Sprite, 96.0)}
                --
                @{palette()}
                --
                Node { column_gap: px(6) }
                Children [
                    @{text_button(Variant::Outline, Size::Sm, "Clear")}
                    on(|_: On<Activate>, mut yours: ResMut<Yours>| {
                        yours.pixels.fill(0);
                        yours.made_sprite = true;
                    })
                    --
                    @{text_button(Variant::Outline, Size::Sm, "Start over")}
                    on(|_: On<Activate>, mut yours: ResMut<Yours>| {
                        yours.pixels = default_pixels();
                        yours.made_sprite = false;
                    })
                ]
            ]
        ]
    }
}

/// One layer of the 8 × 8 × 8 model to build on (the layer below shows faintly), the layers,
/// the palette and the model as the examples draw it.
fn voxel_editor() -> impl Scene {
    let cells: Vec<Box<dyn Scene>> = (0..VOXELS * VOXELS)
        .map(|index| {
            let cell = VoxelCell(index % VOXELS, index / VOXELS);
            Box::new(bsn! {
                Node { width: px(34), height: px(34) }
                BackgroundColor(color::CARD)
                cell
                on(paint_voxel)
                on(drag_voxel)
            }) as Box<dyn Scene>
        })
        .collect();
    let chips: Vec<Box<dyn Scene>> = (0..VOXELS)
        .rev()
        .map(|layer| {
            let label = (layer + 1).to_string();
            Box::new(bsn! {
                @{text_button(Variant::Ghost, Size::Xs, &label)}
                LayerChip(layer)
                on(move |_: On<Activate>, mut brush: ResMut<Brush>| brush.layer = layer)
            }) as Box<dyn Scene>
        })
        .collect();
    bsn! {
        Node { column_gap: px(16), align_items: AlignItems::Start, flex_wrap: FlexWrap::Wrap, row_gap: px(16) }
        Children [
            Node { flex_direction: FlexDirection::Column, row_gap: px(2), align_items: AlignItems::Center }
            Children [
                @{caption("Layer")}
                --
                {chips}
            ]
            --
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px::<Vec<RepeatedGridTrack>>(VOXELS as u16, 34.0),
                border: UiRect::all(px(1)),
            }
            BorderColor::all(color::INPUT)
            Children [ {cells} ]
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(12) }
            Children [
                @{picture(Studio::Voxels, 140.0)}
                --
                @{palette()}
                --
                Node { column_gap: px(6) }
                Children [
                    @{text_button(Variant::Outline, Size::Sm, "Clear")}
                    on(|_: On<Activate>, mut yours: ResMut<Yours>| {
                        yours.voxels.fill(0);
                        yours.made_voxels = true;
                    })
                    --
                    @{text_button(Variant::Outline, Size::Sm, "Start over")}
                    on(|_: On<Activate>, mut yours: ResMut<Yours>| {
                        yours.voxels = default_voxels();
                        yours.made_voxels = false;
                    })
                ]
            ]
        ]
    }
}

/// The reader's sprite or model, `size` px square at most, crisp.
fn picture(studio: Studio, size: f32) -> impl Scene {
    bsn! {
        Node { width: px(size), height: px(size), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(px(1)), border_radius: BorderRadius::all(px(radius::MD)) }
        BorderColor::all(color::BORDER)
        BackgroundColor(color::ACCENT)
        Children [
            Node { width: Val::Percent(90.0), height: Val::Percent(90.0) }
            ImageNode::default()
            Picture(studio)
            Pickable::IGNORE
        ]
    }
}

/// The sixteen colours and an eraser.
fn palette() -> impl Scene {
    let swatches: Vec<Box<dyn Scene>> = (0..=PIXEL_PALETTE.len())
        .map(|i| {
            let cell = i as Cell;
            let tip = if cell == 0 { "Eraser" } else { "Paint with this colour" };
            Box::new(bsn! {
                Node { width: px(22), height: px(22), border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(radius::SM)), justify_content: JustifyContent::Center, align_items: AlignItems::Center }
                BackgroundColor(color::CARD)
                BorderColor::all(color::INPUT)
                Tooltip::new(tip)
                Swatch(cell)
                on(move |_: On<PointerPress>, mut brush: ResMut<Brush>| brush.cell = cell)
            }) as Box<dyn Scene>
        })
        .collect();
    bsn! {
        Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px::<Vec<RepeatedGridTrack>>(6, 22.0),
            column_gap: px(4),
            row_gap: px(4),
        }
        Children [ {swatches} ]
    }
}

/// Whether the pointer is held down (a mouse button or a finger), for painting by dragging.
fn held(mouse: &ButtonInput<MouseButton>, touches: &Touches) -> bool {
    mouse.pressed(MouseButton::Left) || touches.iter().next().is_some()
}

fn paint_pixel(
    press: On<PointerPress>,
    cells: Query<&PixelCell>,
    mut brush: ResMut<Brush>,
    mut yours: ResMut<Yours>,
) {
    if let Ok(cell) = cells.get(press.entity) {
        yours.set_pixel(cell.0, brush.cell);
        brush.last = Some((cell.0 % SPRITE, cell.0 / SPRITE));
    }
}

fn drag_pixel(
    over: On<PointerOver>,
    cells: Query<&PixelCell>,
    mut brush: ResMut<Brush>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut yours: ResMut<Yours>,
) {
    if held(&mouse, &touches)
        && let Ok(cell) = cells.get(over.entity)
    {
        let at = (cell.0 % SPRITE, cell.0 / SPRITE);
        for (x, y) in stroke(brush.last.unwrap_or(at), at) {
            yours.set_pixel(x + y * SPRITE, brush.cell);
        }
        brush.last = Some(at);
    }
}

fn paint_voxel(
    press: On<PointerPress>,
    cells: Query<&VoxelCell>,
    mut brush: ResMut<Brush>,
    mut yours: ResMut<Yours>,
) {
    if let Ok(&VoxelCell(x, y)) = cells.get(press.entity) {
        // The grid shows the layer from above with `y` growing down: the model's `y` too.
        yours.set_voxel(x, y, brush.layer, brush.cell);
        brush.last = Some((x, y));
    }
}

fn drag_voxel(
    over: On<PointerOver>,
    cells: Query<&VoxelCell>,
    mut brush: ResMut<Brush>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut yours: ResMut<Yours>,
) {
    if held(&mouse, &touches)
        && let Ok(&VoxelCell(x, y)) = cells.get(over.entity)
    {
        for (x, y) in stroke(brush.last.unwrap_or((x, y)), (x, y)) {
            yours.set_voxel(x, y, brush.layer, brush.cell);
        }
        brush.last = Some((x, y));
    }
}

fn swatch_colour(cell: Cell) -> Color {
    if cell == 0 {
        color::CARD
    } else {
        PIXEL_PALETTE[usize::from(cell - 1)]
    }
}

/// Paints the grids, the palette and the layers from the sprite, the model and the brush.
#[allow(clippy::type_complexity)]
fn show_cells(
    yours: Res<Yours>,
    brush: Res<Brush>,
    added: Query<(), Or<(Added<PixelCell>, Added<VoxelCell>, Added<Swatch>)>>,
    mut pixels: Query<(&PixelCell, &mut BackgroundColor), (Without<VoxelCell>, Without<Swatch>)>,
    mut voxels: Query<(&VoxelCell, &mut BackgroundColor), (Without<PixelCell>, Without<Swatch>)>,
    mut swatches: Query<
        (&Swatch, &mut BackgroundColor, &mut BorderColor),
        (Without<PixelCell>, Without<VoxelCell>),
    >,
    mut chips: Query<(&LayerChip, &mut Selected)>,
) {
    if !yours.is_changed() && !brush.is_changed() && added.is_empty() {
        return;
    }
    // Empty cells are a checkerboard, so clear reads as clear.
    let empty = |a: usize, b: usize| {
        if (a + b).is_multiple_of(2) {
            color::CARD
        } else {
            color::ACCENT
        }
    };
    for (cell, mut background) in &mut pixels {
        let value = yours.pixels[cell.0];
        let wanted = if value == 0 {
            empty(cell.0 % SPRITE, cell.0 / SPRITE)
        } else {
            swatch_colour(value)
        };
        background.set_if_neq(BackgroundColor(wanted));
    }
    for (&VoxelCell(x, y), mut background) in &mut voxels {
        let value = yours.voxel(x, y, brush.layer);
        let below = if brush.layer > 0 {
            yours.voxel(x, y, brush.layer - 1)
        } else {
            0
        };
        let wanted = if value != 0 {
            swatch_colour(value)
        } else if below != 0 {
            // The layer below, faintly: what this one builds on.
            swatch_colour(below).mix(&color::CARD, 0.75)
        } else {
            empty(x, y)
        };
        background.set_if_neq(BackgroundColor(wanted));
    }
    for (swatch, mut background, mut border) in &mut swatches {
        background.set_if_neq(BackgroundColor(swatch_colour(swatch.0)));
        let ring = if swatch.0 == brush.cell {
            color::FOREGROUND
        } else {
            color::INPUT
        };
        border.set_if_neq(BorderColor::all(ring));
    }
    for (chip, mut selected) in &mut chips {
        selected.set_if_neq(Selected(chip.0 == brush.layer));
    }
}

/// Puts the sprite and model images in their pictures.
fn show_pictures(yours: Res<Yours>, mut pictures: Query<(&Picture, &mut ImageNode)>) {
    for (picture, mut image) in &mut pictures {
        let wanted = match picture.0 {
            Studio::Sprite => &yours.sprite,
            Studio::Voxels => &yours.model,
        };
        if image.image != *wanted {
            image.image = wanted.clone();
        }
    }
}

/// Shows the reader's sprite over the sidebar's mark once they have drawn one.
fn show_mark(
    yours: Res<Yours>,
    mut marks: Query<(&mut ImageNode, &mut Visibility), With<YourMark>>,
) {
    for (mut image, mut visibility) in &mut marks {
        if image.image != yours.sprite {
            image.image = yours.sprite.clone();
        }
        visibility.set_if_neq(if yours.made_sprite {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::stroke;

    #[test]
    fn a_stroke_has_no_gaps() {
        let cells = stroke((0, 0), (5, 2));
        assert_eq!(cells.first(), Some(&(0, 0)));
        assert_eq!(cells.last(), Some(&(5, 2)));
        for pair in cells.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            assert!(
                a.0.abs_diff(b.0) <= 1 && a.1.abs_diff(b.1) <= 1,
                "{a:?} to {b:?}"
            );
        }
        assert_eq!(stroke((3, 3), (3, 3)), vec![(3, 3), (3, 3)]);
    }
}

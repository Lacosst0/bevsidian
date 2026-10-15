//! "Make it yours": draw a pixel sprite, build a voxel model, make sound effects. All are
//! saved, and the examples play with them: the player and the ships become the sprite, rocks
//! and guards the model, and the examples' hits, bounces and pops sound like the reader's
//! sounds (see `bevsidian_preview::yours`). The sidebar's mark shows the sprite too.
//!
//! Opened from the brush, box and speaker buttons above the examples, and from the sidebar's
//! mark.

use bevy::{
    audio::{AudioPlayer, PlaybackSettings},
    picking::events::PointerOver,
    prelude::*,
    ui_widgets::Activate,
};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::nav::{Nav, Studio};
use bevsidian_preview::{
    sound::{self, LENGTH, PITCH, PRESETS, SLIDE, Sfx, Slot, VOLUME, Wave},
    yours::{Cell, SPRITE, VOXELS, Yours, default_pixels, default_voxels},
};
use bevsidian_ui::{color::PIXEL_PALETTE, prelude::*};

pub fn plugin(app: &mut App) {
    app.init_resource::<Brush>()
        .init_resource::<SoundSlot>()
        .add_systems(Update, (show_cells, show_pictures, show_mark, play_edits));
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

/// The sound editor; its slot is the sound being made.
#[derive(Component, Clone, Copy, Default)]
struct SoundEditor;

/// Which of the examples' sounds is being made.
#[derive(Resource, Clone, Copy, Default)]
pub(crate) struct SoundSlot(usize);

impl SoundSlot {
    fn slot(self) -> Slot {
        Slot::ALL[self.0.min(Slot::ALL.len() - 1)]
    }
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

#[component]
pub fn studio_dialog(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::DIALOG)
        Pickable::IGNORE
        Children [
            if $nav.studio.is_some() {
                @{scrim(close)}
                Node { justify_content: JustifyContent::Center, align_items: AlignItems::Start, padding: UiRect::top(Val::Percent(4.0)) }
                Children [ @{panel()} ]
            }
        ]
    }
}

fn close(world: &mut World) {
    world.resource_mut::<Nav>().studio = None;
}

#[component]
fn panel(nav: Watch<Res<Nav>>) -> impl Scene {
    let tabs = segmented(
        vec![
            (Some(Studio::Sprite), "Pixel sprite".to_owned()),
            (Some(Studio::Voxels), "Voxel model".to_owned()),
            (Some(Studio::Sounds), "Sounds".to_owned()),
        ],
        Binding::resource(
            nav.clone(),
            |n: &Nav| n.studio,
            |n, studio| n.studio = studio,
        ),
    );
    view! {
        @{dialog(640.0)}
        Node { row_gap: px(16) }
        Children [
            Node { flex_direction: FlexDirection::Column, row_gap: px(6) }
            Children [
                @{dialog_header("Make it yours", close)}
                --
                @{muted("Draw a sprite, build a voxel model, make sounds. They are saved, and the examples play with them: the player and the ships become your sprite, rocks and guards your model, and hits, bounces and pops sound like yours.")}
            ]
            --
            @{tabs}
            --
            match $nav.studio {
                Some(Studio::Sprite) => { @{sprite_editor()} },
                Some(Studio::Voxels) => { @{voxel_editor()} },
                Some(Studio::Sounds) => { @{sound_editor()} },
                None => {},
            }
            --
            @{dialog_footer()}
            Children [
                @{text_button(Variant::Primary, Size::Sm, "Done")}
                on(|_: On<Activate>, mut n: ResMut<Nav>| n.studio = None)
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

/// A number of the slot's sound, which a control shows and sets.
fn sound_number(
    yours: &Watch<Res<'static, Yours>>,
    slot: &Watch<Res<'static, SoundSlot>>,
    get: fn(&Sfx) -> f32,
    set: fn(&mut Sfx, f32),
) -> Binding<f32> {
    let (read, write) = (slot.clone(), slot.clone());
    Binding::resource(
        yours.clone(),
        move |yours: &Yours| get(&yours.sounds[read.get().slot().index()]),
        move |yours, value| {
            let slot = write.get_untracked().slot();
            let mut sfx = yours.sounds[slot.index()];
            set(&mut sfx, value);
            yours.set_sound(slot, sfx);
        },
    )
}

/// The sound editor: the examples' sounds to pick from, and the picked one's wave, pitch,
/// slide, length and volume, with ready-made sounds and a surprise. It plays as it changes
/// ([`play_edits`]).
#[component]
fn sound_editor(yours: Watch<Res<Yours>>, slot: Watch<Res<SoundSlot>>) -> impl Scene {
    let slots: Vec<Box<dyn Scene>> = Slot::ALL
        .into_iter()
        .map(|choice| {
            let current = slot.clone();
            let index = choice.index();
            Box::new(view! {
                @{item()}
                Node { height: px(44) }
                Selected($current.0 == index)
                on(move |_: On<Activate>, mut slot: ResMut<SoundSlot>, yours: Res<Yours>, mut commands: Commands| {
                    slot.0 = index;
                    play(&mut commands, &yours, choice);
                })
                Children [
                    @{item_icon("volume-2")}
                    --
                    Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, min_width: px(0) }
                    Pickable::IGNORE
                    Children [
                        @{label(choice.name(), size::SM, bevy::text::FontWeight::MEDIUM, color::FOREGROUND)}
                        --
                        @{caption(choice.plays())}
                    ]
                ]
            }) as Box<dyn Scene>
        })
        .collect();
    let presets: Vec<Box<dyn Scene>> = PRESETS
        .iter()
        .map(|&(name, sfx)| {
            Box::new(bsn! {
                @{text_button(Variant::Outline, Size::Xs, name)}
                on(move |_: On<Activate>, mut yours: ResMut<Yours>, slot: Res<SoundSlot>| {
                    yours.set_sound(slot.slot(), sfx);
                })
            }) as Box<dyn Scene>
        })
        .collect();
    let wave = {
        let (read, write) = (slot.clone(), slot.clone());
        Binding::resource(
            yours.clone(),
            move |yours: &Yours| yours.sounds[read.get().slot().index()].wave,
            move |yours, wave| {
                let slot = write.get_untracked().slot();
                let sfx = Sfx {
                    wave,
                    ..yours.sounds[slot.index()]
                };
                yours.set_sound(slot, sfx);
            },
        )
    };
    let waves = segmented(
        Wave::ALL
            .iter()
            .map(|&w| (w, w.name().to_owned()))
            .collect(),
        wave,
    );
    let row = |name: &str, value: Binding<f32>, range: (f32, f32), shown: fn(f32) -> String| {
        let number = value.clone();
        Box::new(view! {
            Node { align_items: AlignItems::Center, column_gap: px(12) }
            Children [
                Node { width: px(56), flex_shrink: 0.0 }
                Children [ @{caption(name)} ]
                --
                @{slider(value.clone(), range.0..=range.1)}
                --
                Text($(shown($number)))
                @{insert(font(size::XS, bevy::text::FontWeight::NORMAL))}
                TextColor(color::MUTED_FOREGROUND)
                Node { width: px(64), flex_shrink: 0.0 }
                Pickable::IGNORE
            ]
        }) as Box<dyn Scene>
    };
    let number = |get, set| sound_number(&yours, &slot, get, set);
    let rows: Vec<Box<dyn Scene>> = vec![
        row(
            "Pitch",
            number(|s| s.pitch, |s, v| s.pitch = v),
            PITCH,
            |v| format!("{v:.0} Hz"),
        ),
        row(
            "Slide",
            number(|s| s.slide, |s, v| s.slide = v),
            SLIDE,
            |v| format!("{v:+.1} oct"),
        ),
        row(
            "Length",
            number(|s| s.length, |s, v| s.length = v),
            LENGTH,
            |v| format!("{:.0} ms", v * 1000.0),
        ),
        row(
            "Volume",
            number(|s| s.volume, |s, v| s.volume = v),
            VOLUME,
            |v| format!("{:.0} %", v * 100.0),
        ),
    ];
    bsn! {
        Node { column_gap: px(20), align_items: AlignItems::Start, flex_wrap: FlexWrap::Wrap, row_gap: px(16) }
        SoundEditor
        Children [
            Node { flex_direction: FlexDirection::Column, width: px(220), row_gap: px(2) }
            Children [ {slots} ]
            --
            Node { flex_direction: FlexDirection::Column, row_gap: px(12), flex_grow: 1.0, min_width: px(280) }
            Children [
                @{waves}
                --
                Node { flex_direction: FlexDirection::Column, row_gap: px(10) }
                Children [ {rows} ]
                --
                Node { flex_wrap: FlexWrap::Wrap, column_gap: px(4), row_gap: px(4) }
                Children [ {presets} ]
                --
                Node { column_gap: px(6) }
                Children [
                    @{text_button(Variant::Secondary, Size::Sm, "Play")}
                    on(|_: On<Activate>, yours: Res<Yours>, slot: Res<SoundSlot>, mut commands: Commands| {
                        play(&mut commands, &yours, slot.slot());
                    })
                    --
                    @{text_button(Variant::Outline, Size::Sm, "Surprise me")}
                    on(|_: On<Activate>, time: Res<Time<Real>>, mut yours: ResMut<Yours>, slot: Res<SoundSlot>| {
                        let seed = (time.elapsed_secs_f64() * 1000.0) as u32;
                        yours.set_sound(slot.slot(), sound::random(seed));
                    })
                    --
                    @{text_button(Variant::Outline, Size::Sm, "Start over")}
                    on(|_: On<Activate>, mut yours: ResMut<Yours>| {
                        yours.sounds = Slot::ALL.iter().map(|s| s.default_sound()).collect();
                        yours.made_sounds = false;
                    })
                ]
            ]
        ]
    }
}

/// Plays one of the reader's sounds now.
fn play(commands: &mut Commands, yours: &Yours, slot: Slot) {
    commands.spawn((AudioPlayer(yours.sound(slot)), PlaybackSettings::DESPAWN));
}

/// While the sound editor is open, plays the slot's sound once an edit settles (a slider
/// drag stops for a moment), after its audio is remade.
fn play_edits(
    editors: Query<(), With<SoundEditor>>,
    yours: Res<Yours>,
    slot: Res<SoundSlot>,
    time: Res<Time<Real>>,
    mut last: Local<Option<(Slot, Sfx)>>,
    mut settle: Local<Option<f64>>,
    mut commands: Commands,
) {
    if editors.is_empty() {
        *last = None;
        *settle = None;
        return;
    }
    let slot = slot.slot();
    let sfx = yours.sounds[slot.index()];
    match *last {
        Some((was, before)) if was == slot && before != sfx => {
            *settle = Some(time.elapsed_secs_f64() + 0.3);
        }
        _ => {}
    }
    *last = Some((slot, sfx));
    if settle.is_some_and(|at| time.elapsed_secs_f64() >= at) {
        *settle = None;
        play(&mut commands, &yours, slot);
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
            Studio::Sounds => continue,
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

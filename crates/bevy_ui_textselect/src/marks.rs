//! Marked text: ranges of selectable blocks highlighted in colours of their own, such as the
//! matches of a find-in-page, and [`TextBlocks`] to read blocks and locate ranges on screen.

use bevy::{prelude::*, ui::UiGlobalTransform};
use parley::{Affinity, Cursor, Selection};

use crate::{Blocks, SelectionGroup, text};

/// Text to highlight besides the selection (drawn under it), e.g. find-in-page matches.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct TextMarks(pub Vec<TextMark>);

/// A highlighted byte range of a [`SelectableText`](crate::SelectableText) block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMark {
    pub block: Entity,
    pub start: usize,
    pub end: usize,
    pub color: Color,
}

/// Reads selectable text: the blocks of a [`SelectionGroup`], their text, and where a range
/// of one is on screen.
#[derive(bevy::ecs::system::SystemParam)]
pub struct TextBlocks<'w, 's> {
    blocks: Blocks<'w, 's>,
}

impl TextBlocks<'_, '_> {
    /// The selectable blocks inside `group`, in reading order.
    pub fn blocks_in(&self, group: Entity) -> Vec<Entity> {
        self.blocks.blocks_of(group)
    }

    /// A block's text as laid out (byte indices of [`TextMark`]s point into it); `None`
    /// before its first layout.
    pub fn text(&self, block: Entity) -> Option<String> {
        let (computed, ..) = self.blocks.texts.get(block).ok()?;
        Some(text::block_text(
            computed,
            &self.blocks.text,
            &self.blocks.spans,
        ))
    }

    /// The bounds of a byte range of a block, in UI physical pixels (the space of
    /// [`UiGlobalTransform`]); `None` before its layout.
    pub fn range_rect(&self, block: Entity, start: usize, end: usize) -> Option<Rect> {
        let (computed, node, transform, _) = self.blocks.texts.get(block).ok()?;
        let content_min = node.content_box().min;
        let to_ui = |x: f64, y: f64| {
            transform.transform_point2(content_min + Vec2::new(x as f32, y as f32))
        };
        geometry(computed.buffer(), start, end)
            .map(|(min, max)| Rect::from_corners(to_ui(min.0, min.1), to_ui(max.0, max.1)))
            .reduce(|a, b| a.union(b))
    }
}

/// Line boxes covering a byte range of a layout, as ((x0, y0), (x1, y1)).
fn geometry(
    layout: &parley::Layout<bevy::text::TextBrush>,
    start: usize,
    end: usize,
) -> impl Iterator<Item = ((f64, f64), (f64, f64))> {
    Selection::new(
        Cursor::from_byte_index(layout, start, Affinity::Downstream),
        Cursor::from_byte_index(layout, end, Affinity::Upstream),
    )
    .geometry(layout)
    .into_iter()
    .map(|(bbox, _)| ((bbox.x0, bbox.y0), (bbox.x1, bbox.y1)))
}

/// The layer of mark rectangles inside a group.
#[derive(Component)]
pub(crate) struct MarkLayer;

/// Draws [`TextMarks`]: a layer of rectangles in each group that has marks, redrawn when the
/// marks change or their blocks move within their groups.
pub(crate) fn draw_marks(
    marks: Res<TextMarks>,
    blocks: Blocks,
    groups: Query<(&ComputedNode, &UiGlobalTransform), With<SelectionGroup>>,
    layers: Query<Entity, With<MarkLayer>>,
    mut commands: Commands,
    mut last: Local<Option<(TextMarks, Vec<Vec2>)>>,
) {
    // Where each marked block sits in its group: when that changes, the rects move.
    let mut placed = Vec::new();
    let mut rects: Vec<(Entity, Rect, Color)> = Vec::new();
    for mark in &marks.0 {
        let group = blocks.group_of(mark.block);
        let (Ok((computed, node, transform, _)), Ok((group_node, group_transform))) =
            (blocks.texts.get(mark.block), groups.get(group))
        else {
            continue;
        };
        let Some(group_inverse) = group_transform.try_inverse() else {
            continue;
        };
        let origin = group_node.padding_box().min;
        let inv = group_node.inverse_scale_factor;
        let content_min = node.content_box().min;
        let to_group = |x: f64, y: f64| {
            let ui = transform.transform_point2(content_min + Vec2::new(x as f32, y as f32));
            (group_inverse.transform_point2(ui) - origin) * inv
        };
        placed.push(to_group(0.0, 0.0));
        for (min, max) in geometry(computed.buffer(), mark.start, mark.end) {
            rects.push((
                group,
                Rect::from_corners(to_group(min.0, min.1), to_group(max.0, max.1)),
                mark.color,
            ));
        }
    }
    let key = (marks.clone(), placed);
    if last.as_ref() == Some(&key) {
        return;
    }
    *last = Some(key);
    for layer in &layers {
        commands.entity(layer).despawn();
    }
    let mut layers: Vec<(Entity, Entity)> = Vec::new();
    for (group, rect, color) in rects {
        let layer = if let Some(&(_, layer)) = layers.iter().find(|(g, _)| *g == group) {
            layer
        } else {
            let layer = commands
                .spawn((
                    MarkLayer,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    // Under the selection's layer.
                    ZIndex(999),
                    Pickable::IGNORE,
                    ChildOf(group),
                ))
                .id();
            layers.push((group, layer));
            layer
        };
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(rect.min.x),
                top: Val::Px(rect.min.y),
                width: Val::Px(rect.width()),
                height: Val::Px(rect.height()),
                ..default()
            },
            BackgroundColor(color),
            Pickable::IGNORE,
            ChildOf(layer),
        ));
    }
}

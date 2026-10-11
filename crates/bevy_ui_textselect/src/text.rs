//! Mapping between Bevy text hierarchies and the byte offsets of their parley layout.
//!
//! Bevy builds a block's layout from the concatenation of the root [`Text`] and every
//! [`TextSpan`], in the order reported by [`ComputedTextBlock::entities`]. Byte indices from
//! parley therefore index into that concatenation.

use core::ops::Range;

use bevy::{
    prelude::*,
    text::{ComputedTextBlock, TextBrush},
};
use parley::Layout;

use crate::CopyText;

fn section_text<'a>(
    entity: Entity,
    text: &'a Query<&Text>,
    spans: &'a Query<&TextSpan>,
) -> Option<&'a str> {
    text.get(entity)
        .map(|t| t.0.as_str())
        .or_else(|_| spans.get(entity).map(|s| s.0.as_str()))
        .ok()
}

/// Iterates `(entity, byte range)` for each text section of a block.
pub fn sections<'a>(
    computed: &'a ComputedTextBlock,
    text: &'a Query<&Text>,
    spans: &'a Query<&TextSpan>,
) -> impl Iterator<Item = (Entity, Range<usize>, &'a str)> + 'a {
    let mut offset = 0;
    computed.entities().iter().filter_map(move |e| {
        let s = section_text(e.entity, text, spans)?;
        let range = offset..offset + s.len();
        offset += s.len();
        Some((e.entity, range, s))
    })
}

/// The full text of a block, exactly as laid out.
pub fn block_text(
    computed: &ComputedTextBlock,
    text: &Query<&Text>,
    spans: &Query<&TextSpan>,
) -> String {
    sections(computed, text, spans).map(|(.., s)| s).collect()
}

/// Text for the byte range `range` of a block, honouring [`CopyText`] overrides of spans
/// that are selected in full.
pub fn slice_text(
    computed: &ComputedTextBlock,
    text: &Query<&Text>,
    spans: &Query<&TextSpan>,
    copy: &Query<&CopyText>,
    range: Range<usize>,
) -> String {
    let mut out = String::new();
    for (entity, section, s) in sections(computed, text, spans) {
        let start = range.start.max(section.start);
        let end = range.end.min(section.end);
        if start >= end {
            continue;
        }
        if start == section.start
            && end == section.end
            && let Ok(copy) = copy.get(entity)
        {
            out.push_str(&copy.0);
            continue;
        }
        let local = clamp_char_boundary(s, start - section.start)
            ..clamp_char_boundary(s, end - section.start);
        out.push_str(&s[local]);
    }
    out
}

fn clamp_char_boundary(s: &str, mut i: usize) -> usize {
    i = i.min(s.len());
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// The section entity containing byte `index` (a cursor at the end of a span belongs to it).
pub fn span_at(
    computed: &ComputedTextBlock,
    text: &Query<&Text>,
    spans: &Query<&TextSpan>,
    index: usize,
) -> Option<Entity> {
    let mut last = None;
    for (entity, range, _) in sections(computed, text, spans) {
        if range.is_empty() {
            continue;
        }
        if range.contains(&index) {
            return Some(entity);
        }
        last = Some((entity, range));
    }
    last.filter(|(_, r)| r.end == index).map(|(e, _)| e)
}

/// `true` if `point` (layout space) lies on a laid-out line, horizontally within its glyphs.
pub fn point_inside_text(layout: &Layout<TextBrush>, point: Vec2) -> bool {
    for line in layout.lines() {
        let metrics = line.metrics();
        let top = metrics.block_min_coord;
        let bottom = metrics.block_max_coord;
        if point.y >= top && point.y <= bottom {
            let left = metrics.offset;
            let right = left + metrics.advance;
            return point.x >= left && point.x <= right;
        }
    }
    false
}

/// Whether the browser reports a Mac/iOS host.
#[cfg(target_arch = "wasm32")]
pub(crate) fn web_is_mac() -> bool {
    web_sys::window().is_some_and(|w| {
        let navigator = w.navigator();
        // `platform()` is deprecated but universally shipped; fall back to the UA string.
        let platform = navigator.platform().unwrap_or_default();
        platform.starts_with("Mac")
            || platform.starts_with("iP")
            || (platform.is_empty() && navigator.user_agent().unwrap_or_default().contains("Mac"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_boundaries() {
        let s = "aé";
        assert_eq!(clamp_char_boundary(s, 2), 1);
        assert_eq!(clamp_char_boundary(s, 9), 3);
    }
}

//! The notes' look: the markdown theme built from the UI kit's tokens, and the kit's icons for
//! the renderer (callouts, copy buttons).

use bevy::{prelude::*, text::FontWeight};
use bevy_ui_commonmark::MdTheme;

use bevsidian_ui::color;

/// shadcn/ui typography: Geist prose in neutral greys, orange links, neutral callouts (only
/// warnings and cautions get colour) and GitHub-dark code colours.
pub fn note_theme() -> MdTheme {
    MdTheme {
        body: bevy::text::FontSource::families("Geist, sans-serif"),
        mono: bevy::text::FontSource::families("Geist Mono, monospace"),
        line_height: 1.75,
        block_gap: 16.0,
        heading_weight: FontWeight::SEMIBOLD,
        heading_space: 24.0,
        heading_rule: 2,
        text: color::BODY,
        muted: color::MUTED_FOREGROUND,
        heading: color::FOREGROUND,
        strong: color::FOREGROUND,
        link: color::LINK,
        unresolved_link: color::LINK.with_alpha(0.5),
        tag: color::BODY,
        tag_bg: color::ACCENT,
        code: color::BODY,
        code_bg: color::ACCENT,
        code_block_bg: color::BACKGROUND,
        highlight: color::LINK_SOFT,
        border: color::BORDER,
        table_header_bg: Color::NONE,
        table_header: color::MUTED_FOREGROUND,
        surface: color::CARD,
        radius: bevsidian_ui::radius::XL,
        callouts: [
            color::FOREGROUND,
            color::FOREGROUND,
            color::FOREGROUND,
            color::WARNING,
            color::DESTRUCTIVE,
        ],
        syntax_keyword: Color::srgb_u8(0xff, 0x7b, 0x72),
        syntax_string: Color::srgb_u8(0xa5, 0xd6, 0xff),
        syntax_comment: Color::srgb_u8(0x8b, 0x94, 0x9e),
        syntax_number: Color::srgb_u8(0x79, 0xc0, 0xff),
        syntax_type: Color::srgb_u8(0xd2, 0xa8, 0xff),
        ..MdTheme::dark()
    }
}

/// A Lucide icon for the markdown renderer.
pub fn icon(name: &str, size: f32, color: Color) -> Box<dyn Scene> {
    Box::new(bevsidian_ui::icon(name, size, color))
}

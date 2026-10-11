//! Visual theme for rendered markdown.

use bevy::{prelude::*, text::FontSource};

use crate::document::Callout;

/// Fonts, sizes and colours used by [`markdown_scene`](crate::markdown_scene).
///
/// The default is a dark, Obsidian-like palette using the `Inter` and `JetBrains Mono`
/// families, falling back to Bevy's default font when they are not loaded.
#[derive(Resource, Clone, Debug)]
pub struct MdTheme {
    /// Body font.
    pub body: FontSource,
    /// Monospace font for code.
    pub mono: FontSource,
    /// Paragraph font size in logical px.
    pub base_size: f32,
    /// Line height relative to font size.
    pub line_height: f32,
    /// Vertical gap between blocks, in logical px.
    pub block_gap: f32,
    /// Body text.
    pub text: Color,
    /// Secondary text (list markers, captions).
    pub muted: Color,
    /// Heading text.
    pub heading: Color,
    /// Bold text.
    pub strong: Color,
    /// Resolved `[[wikilink]]` and URL text.
    pub link: Color,
    /// `[[wikilink]]` whose note does not exist.
    pub unresolved_link: Color,
    /// `#tag` text.
    pub tag: Color,
    /// `#tag` background.
    pub tag_bg: Color,
    /// Inline and block code text.
    pub code: Color,
    /// Inline and block code background.
    pub code_bg: Color,
    /// Block quote bar and table borders.
    pub border: Color,
    /// Table header background.
    pub table_header_bg: Color,
    /// Syntax highlighting: keywords.
    pub syntax_keyword: Color,
    /// Syntax highlighting: strings.
    pub syntax_string: Color,
    /// Syntax highlighting: comments.
    pub syntax_comment: Color,
    /// Syntax highlighting: numbers and constants.
    pub syntax_number: Color,
    /// Syntax highlighting: types and macros.
    pub syntax_type: Color,
}

impl Default for MdTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl MdTheme {
    /// Dark theme.
    pub fn dark() -> Self {
        Self {
            body: FontSource::families("Inter, sans-serif"),
            mono: FontSource::families("JetBrains Mono, monospace"),
            base_size: 16.0,
            line_height: 1.6,
            block_gap: 14.0,
            text: Color::srgb_u8(0xda, 0xda, 0xda),
            muted: Color::srgb_u8(0x99, 0x99, 0x99),
            heading: Color::srgb_u8(0xf2, 0xf2, 0xf2),
            strong: Color::srgb_u8(0xff, 0xff, 0xff),
            link: Color::srgb_u8(0xa8, 0x8b, 0xfa),
            unresolved_link: Color::srgb_u8(0x8a, 0x7a, 0xb8),
            tag: Color::srgb_u8(0xb6, 0xa5, 0xff),
            tag_bg: Color::srgba_u8(0x8b, 0x6c, 0xef, 0x2e),
            code: Color::srgb_u8(0xe6, 0x9d, 0x7b),
            code_bg: Color::srgb_u8(0x26, 0x26, 0x2b),
            border: Color::srgb_u8(0x3a, 0x3a, 0x42),
            table_header_bg: Color::srgb_u8(0x24, 0x24, 0x2a),
            syntax_keyword: Color::srgb_u8(0xc6, 0x78, 0xdd),
            syntax_string: Color::srgb_u8(0x98, 0xc3, 0x79),
            syntax_comment: Color::srgb_u8(0x7f, 0x84, 0x8e),
            syntax_number: Color::srgb_u8(0xd1, 0x9a, 0x66),
            syntax_type: Color::srgb_u8(0x61, 0xaf, 0xef),
        }
    }

    /// Light theme.
    pub fn light() -> Self {
        Self {
            text: Color::srgb_u8(0x2e, 0x2e, 0x33),
            muted: Color::srgb_u8(0x6b, 0x6b, 0x75),
            heading: Color::srgb_u8(0x10, 0x10, 0x14),
            strong: Color::srgb_u8(0x00, 0x00, 0x00),
            link: Color::srgb_u8(0x70, 0x4c, 0xe0),
            unresolved_link: Color::srgb_u8(0x9a, 0x88, 0xc8),
            tag: Color::srgb_u8(0x5a, 0x3c, 0xc8),
            tag_bg: Color::srgba_u8(0x8b, 0x6c, 0xef, 0x24),
            code: Color::srgb_u8(0xb5, 0x4a, 0x1f),
            code_bg: Color::srgb_u8(0xf0, 0xf0, 0xf3),
            border: Color::srgb_u8(0xd8, 0xd8, 0xde),
            table_header_bg: Color::srgb_u8(0xf3, 0xf3, 0xf6),
            syntax_keyword: Color::srgb_u8(0xa6, 0x26, 0xa4),
            syntax_string: Color::srgb_u8(0x50, 0xa1, 0x4f),
            syntax_comment: Color::srgb_u8(0xa0, 0xa1, 0xa7),
            syntax_number: Color::srgb_u8(0x98, 0x68, 0x01),
            syntax_type: Color::srgb_u8(0x40, 0x78, 0xf2),
            ..Self::dark()
        }
    }

    /// Font size of a heading level.
    pub fn heading_size(&self, level: u8) -> f32 {
        self.base_size
            * match level {
                1 => 2.0,
                2 => 1.6,
                3 => 1.35,
                4 => 1.15,
                5 => 1.0,
                _ => 0.9,
            }
    }

    /// Accent colour of a callout.
    pub fn callout_color(&self, callout: Callout) -> Color {
        match callout {
            Callout::Note => Color::srgb_u8(0x44, 0x8a, 0xff),
            Callout::Tip => Color::srgb_u8(0x00, 0xbf, 0xa5),
            Callout::Important => Color::srgb_u8(0xa8, 0x8b, 0xfa),
            Callout::Warning => Color::srgb_u8(0xff, 0x91, 0x00),
            Callout::Caution => Color::srgb_u8(0xff, 0x52, 0x52),
        }
    }
}

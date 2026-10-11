//! Builds Bevy UI scenes (BSN) from an [`MdDocument`].

use bevy::{
    clipboard::Clipboard,
    picking::events::PointerClick,
    prelude::*,
    text::{FontSource, FontStyle, FontWeight, LineHeight},
};
use bevy_ui_textselect::{ClickableSpan, SelectableText, SelectionGroup};

use crate::{
    document::{
        Block, Callout, ColumnAlign, Inline, LinkTarget, ListItem, MdDocument, WikiLink,
        inline_text,
    },
    highlight::{TokenKind, highlight},
    insert,
    theme::MdTheme,
};

/// A clickable markdown link on a text span.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct MdLink(pub LinkTarget);

impl Default for MdLink {
    fn default() -> Self {
        Self(LinkTarget::Url(String::new()))
    }
}

/// A heading block, for outlines and `[[Note#Heading]]` scrolling.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct MdHeading {
    /// 1..=6
    pub level: u8,
    /// Anchor slug.
    pub slug: String,
}

/// A footnote definition block.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct MdFootnote(pub String);

/// Root of a rendered markdown document.
#[derive(Component, Clone, Debug, Default)]
pub struct MdRoot;

/// Options for [`markdown_scene`].
pub struct RenderOptions<'a> {
    /// Visual theme.
    pub theme: &'a MdTheme,
    /// Whether a wikilink target exists (unresolved links are dimmed).
    pub resolve: &'a dyn Fn(&WikiLink) -> bool,
}

type Scenes = Vec<Box<dyn Scene>>;

/// Renders a document as a column of selectable blocks.
///
/// The root is a [`SelectionGroup`], so a drag can select across all blocks.
pub fn markdown_scene(doc: &MdDocument, options: &RenderOptions) -> impl Scene + use<> {
    let theme = options.theme;
    let blocks = render_blocks(&doc.blocks, options, 0);
    let node = Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.block_gap),
        width: Val::Percent(100.0),
        ..default()
    };
    bsn! {
        MdRoot
        SelectionGroup
        node
        Children [ {blocks} ]
    }
}

fn render_blocks(blocks: &[Block], options: &RenderOptions, depth: usize) -> Scenes {
    blocks
        .iter()
        .map(|b| render_block(b, options, depth))
        .collect()
}

#[allow(clippy::too_many_lines)]
fn render_block(block: &Block, options: &RenderOptions, depth: usize) -> Box<dyn Scene> {
    let theme = options.theme;
    match block {
        Block::Heading {
            level,
            inlines,
            slug,
        } => {
            let size = theme.heading_size(*level);
            let base = SpanStyle {
                font: theme.body.clone(),
                size,
                weight: if *level <= 2 {
                    FontWeight::BOLD
                } else {
                    FontWeight::SEMIBOLD
                },
                color: theme.heading,
                ..SpanStyle::body(theme)
            };
            let heading = MdHeading {
                level: *level,
                slug: slug.clone(),
            };
            let node = Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Px(if *level <= 2 { 12.0 } else { 6.0 })),
                ..default()
            };
            let text = text_block(inlines, &base, options);
            let rule = (*level == 1).then(|| {
                let line = Node {
                    height: Val::Px(1.0),
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                };
                let bg = BackgroundColor(theme.border);
                bsn! { line bg }
            });
            Box::new(bsn! {
                node
                Children [
                    @{text} heading
                    --
                    {rule}
                ]
            })
        }
        Block::Paragraph(inlines) => {
            let base = SpanStyle::body(theme);
            Box::new(text_block(inlines, &base, options))
        }
        Block::CodeBlock { lang, code } => code_block(lang.as_deref(), code, theme),
        Block::Quote { callout, blocks } => {
            let accent = callout.map_or(theme.border, |c| theme.callout_color(c));
            let mut children = Vec::new();
            if let Some(callout) = callout {
                children.push(callout_title(*callout, theme));
            }
            children.extend(render_blocks(blocks, options, depth));
            let node = Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(theme.block_gap * 0.6),
                padding: UiRect::new(Val::Px(16.0), Val::Px(12.0), Val::Px(10.0), Val::Px(10.0)),
                border: UiRect::left(Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            };
            let bg = BackgroundColor(if callout.is_some() {
                accent.with_alpha(0.1)
            } else {
                Color::NONE
            });
            let border = BorderColor::all(accent);
            Box::new(bsn! { node bg border Children [ {children} ] })
        }
        Block::List { start, items } => list(*start, items, options, depth),
        Block::Table {
            align,
            header,
            rows,
        } => table(align, header, rows, options),
        Block::Rule => {
            let node = Node {
                height: Val::Px(1.0),
                width: Val::Percent(100.0),
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            };
            let bg = BackgroundColor(theme.border);
            Box::new(bsn! { node bg })
        }
        Block::Footnote { label, blocks } => {
            let mut base = SpanStyle::body(theme);
            base.size *= 0.85;
            base.color = theme.muted;
            let marker = text_block(&[Inline::plain(format!("{label}."))], &base, options);
            let content = render_blocks(blocks, options, depth);
            let footnote = MdFootnote(label.clone());
            let row = Node {
                column_gap: Val::Px(8.0),
                ..default()
            };
            Box::new(bsn! {
                row
                footnote
                Children [
                    @{marker}
                    --
                    Node { flex_direction: FlexDirection::Column, flex_grow: 1.0 }
                    Children [ {content} ]
                ]
            })
        }
        Block::Html(html) => code_block(Some("html"), html, theme),
    }
}

fn callout_title(callout: Callout, theme: &MdTheme) -> Box<dyn Scene> {
    let style = SpanStyle {
        weight: FontWeight::BOLD,
        color: theme.callout_color(callout),
        ..SpanStyle::body(theme)
    };
    let style = style.scene();
    let text = Text::new(callout.title());
    Box::new(bsn! { text @{style} SelectableText })
}

fn list(
    start: Option<u64>,
    items: &[ListItem],
    options: &RenderOptions,
    depth: usize,
) -> Box<dyn Scene> {
    let theme = options.theme;
    let rows: Scenes = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let marker: Box<dyn Scene> = if let Some(checked) = item.task {
                checkbox(checked, theme)
            } else {
                let label = match start {
                    Some(n) => format!("{}.", n + i as u64),
                    None => ["•", "◦", "▪"][depth % 3].to_owned(),
                };
                let style = SpanStyle {
                    color: theme.muted,
                    ..SpanStyle::body(theme)
                };
                let style = style.scene();
                let text = Text::new(label);
                let node = Node {
                    min_width: Val::Px(if start.is_some() { 22.0 } else { 14.0 }),
                    justify_content: JustifyContent::End,
                    ..default()
                };
                Box::new(bsn! { text @{style} node })
            };
            let content = render_blocks(&item.blocks, options, depth + 1);
            let content_node = Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(theme.block_gap * 0.4),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            };
            let row = Node {
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Start,
                ..default()
            };
            Box::new(bsn! {
                row
                Children [
                    @{marker}
                    --
                    content_node
                    Children [ {content} ]
                ]
            }) as Box<dyn Scene>
        })
        .collect();
    let node = Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.block_gap * 0.35),
        padding: UiRect::left(Val::Px(if depth == 0 { 4.0 } else { 0.0 })),
        ..default()
    };
    Box::new(bsn! { node Children [ {rows} ] })
}

fn checkbox(checked: bool, theme: &MdTheme) -> Box<dyn Scene> {
    let node = Node {
        width: Val::Px(15.0),
        height: Val::Px(15.0),
        margin: UiRect::top(Val::Px(5.0)),
        border: UiRect::all(Val::Px(1.5)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_shrink: 0.0,
        ..default()
    };
    let border = BorderColor::all(if checked { theme.link } else { theme.muted });
    let bg = BackgroundColor(if checked { theme.link } else { Color::NONE });
    let tick = checked.then(|| {
        let text = Text::new("✓");
        let font = insert(TextFont {
            font: theme.body.clone(),
            font_size: 11.0.into(),
            weight: FontWeight::BOLD,
            ..default()
        });
        bsn! { text @{font} TextColor(Color::BLACK) }
    });
    Box::new(bsn! { node border bg Children [ {tick} ] })
}

fn table(
    align: &[ColumnAlign],
    header: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    options: &RenderOptions,
) -> Box<dyn Scene> {
    let theme = options.theme;
    let columns = header
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0))
        .max(1);
    let mut cells: Scenes = Vec::new();
    let mut push_row = |row: &[Vec<Inline>], is_header: bool, row_index: usize| {
        for col in 0..columns {
            let empty = Vec::new();
            let inlines = row.get(col).unwrap_or(&empty);
            let mut style = SpanStyle::body(theme);
            if is_header {
                style.weight = FontWeight::SEMIBOLD;
                style.color = theme.heading;
            }
            let text = text_block(inlines, &style, options);
            let justify = match align.get(col).copied().unwrap_or_default() {
                ColumnAlign::Center => JustifyContent::Center,
                ColumnAlign::Right => JustifyContent::End,
                _ => JustifyContent::Start,
            };
            let node = Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                border: UiRect::new(
                    Val::Px(if col == 0 { 0.0 } else { 1.0 }),
                    Val::Px(0.0),
                    Val::Px(if row_index == 0 { 0.0 } else { 1.0 }),
                    Val::Px(0.0),
                ),
                justify_content: justify,
                ..default()
            };
            let border = BorderColor::all(theme.border);
            let bg = BackgroundColor(if is_header {
                theme.table_header_bg
            } else {
                Color::NONE
            });
            cells.push(Box::new(bsn! { node border bg Children [ @{text} ] }));
        }
    };
    push_row(header, true, 0);
    for (i, row) in rows.iter().enumerate() {
        push_row(row, false, i + 1);
    }
    let node = Node {
        display: Display::Grid,
        grid_template_columns: vec![RepeatedGridTrack::auto(columns as u16)],
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        overflow: Overflow::clip(),
        align_self: AlignSelf::Start,
        max_width: Val::Percent(100.0),
        ..default()
    };
    let border = BorderColor::all(theme.border);
    Box::new(bsn! { node border Children [ {cells} ] })
}

fn code_block(lang: Option<&str>, code: &str, theme: &MdTheme) -> Box<dyn Scene> {
    let style = SpanStyle {
        font: theme.mono.clone(),
        size: theme.base_size * 0.875,
        color: theme.text,
        line_height: 1.5,
        ..SpanStyle::body(theme)
    };
    let spans: Scenes = highlight(code, lang)
        .into_iter()
        .map(|(text, kind)| {
            let mut s = style.clone();
            s.color = match kind {
                TokenKind::Plain => theme.text,
                TokenKind::Keyword => theme.syntax_keyword,
                TokenKind::String => theme.syntax_string,
                TokenKind::Comment => theme.syntax_comment,
                TokenKind::Number => theme.syntax_number,
                TokenKind::Type => theme.syntax_type,
            };
            if kind == TokenKind::Comment {
                s.italic = true;
            }
            span_scene(text.to_owned(), &s, None)
        })
        .collect();
    let style_scene = style.scene();
    let root = Text::default();
    // Code keeps its line breaks and scrolls sideways instead of wrapping.
    let no_wrap = TextLayout {
        linebreak: LineBreak::NoWrap,
        ..default()
    };
    let code_text: Box<dyn Scene> = Box::new(bsn! {
        Node {
            overflow: Overflow::scroll_x(),
            width: Val::Percent(100.0),
            padding: UiRect::bottom(Val::Px(4.0)),
        }
        bevy::ui_widgets::ScrollArea
        Children [
            root @{style_scene} no_wrap SelectableText
            Node { flex_shrink: 0.0 }
            Children [ {spans} ]
        ]
    });

    let copy_source = code.to_owned();
    let label_style = SpanStyle {
        size: theme.base_size * 0.75,
        color: theme.muted,
        ..SpanStyle::body(theme)
    };
    let label_scene = label_style.scene();
    let copy_scene = label_style.scene();
    let lang_label = Text::new(lang.unwrap_or("").to_owned());
    let copy_label = Text::new("Copy");
    let node = Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::new(Val::Px(14.0), Val::Px(14.0), Val::Px(8.0), Val::Px(12.0)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        row_gap: Val::Px(4.0),
        ..default()
    };
    let bg = BackgroundColor(theme.code_bg);
    let hover_bg = theme.border;
    Box::new(bsn! {
        node bg SelectionGroup
        Children [
            Node { justify_content: JustifyContent::SpaceBetween }
            Children [
                lang_label @{label_scene}
                --
                MdCopyButton
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0))
                }
                BackgroundColor(hover_bg)
                ClickableSpan
                bevy::ui_widgets::Button
                on(move |_: On<PointerClick>, mut clipboard: ResMut<Clipboard>| {
                    if let Err(err) = clipboard.set_text(copy_source.clone()) {
                        warn!("copy failed: {err:?}");
                    }
                })
                Children [ copy_label @{copy_scene} Pickable::IGNORE ]
            ]
            --
            @{code_text}
        ]
    })
}

/// The "Copy" button of a code block.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct MdCopyButton;

/// Resolved style of one text span.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
struct SpanStyle {
    font: FontSource,
    size: f32,
    weight: FontWeight,
    italic: bool,
    color: Color,
    line_height: f32,
    background: Option<Color>,
    strike: bool,
    underline: bool,
}

impl SpanStyle {
    fn body(theme: &MdTheme) -> Self {
        Self {
            font: theme.body.clone(),
            size: theme.base_size,
            weight: FontWeight::NORMAL,
            italic: false,
            color: theme.text,
            line_height: theme.line_height,
            background: None,
            strike: false,
            underline: false,
        }
    }

    fn scene(&self) -> impl Scene + use<> {
        let (font, color, line) = self.components();
        (insert(font), insert(color), insert(line))
    }

    fn components(&self) -> (TextFont, TextColor, LineHeight) {
        (
            TextFont {
                font: self.font.clone(),
                font_size: self.size.into(),
                weight: self.weight,
                style: if self.italic {
                    FontStyle::Italic
                } else {
                    FontStyle::Normal
                },
                ..default()
            },
            TextColor(self.color),
            LineHeight::RelativeToFont(self.line_height),
        )
    }
}

fn span_scene(text: String, style: &SpanStyle, link: Option<MdLink>) -> Box<dyn Scene> {
    let style_scene = style.scene();
    let span = TextSpan(text);
    let bg = style.background.map(|c| {
        let c = TextBackgroundColor(c);
        bsn! { c }
    });
    let strike = style.strike.then_some(bsn! { Strikethrough });
    let underline = style.underline.then_some(bsn! { Underline });
    let link = link.map(|link| bsn! { link ClickableSpan });
    Box::new(bsn! {
        span @{style_scene}
        @{bg}
        @{strike}
        @{underline}
        @{link}
    })
}

fn inline_style(inline: &Inline, base: &SpanStyle, options: &RenderOptions) -> SpanStyle {
    let theme = options.theme;
    let mut s = base.clone();
    if inline.style.strong {
        s.weight = FontWeight(base.weight.0.max(650).saturating_add(100).min(800));
        if s.color == theme.text {
            s.color = theme.strong;
        }
    }
    s.italic |= inline.style.emphasis;
    s.strike |= inline.style.strike;
    if inline.style.code {
        s.font = theme.mono.clone();
        s.size = base.size * 0.88;
        s.color = theme.code;
        s.background = Some(theme.code_bg);
    }
    match &inline.link {
        Some(LinkTarget::Wiki(link)) => {
            s.color = if (options.resolve)(link) {
                theme.link
            } else {
                theme.unresolved_link
            };
        }
        Some(LinkTarget::Url(_)) => {
            s.color = theme.link;
            s.underline = true;
        }
        Some(LinkTarget::Tag(_)) => {
            s.color = theme.tag;
            s.background = Some(theme.tag_bg);
        }
        Some(LinkTarget::Footnote(_)) => {
            s.color = theme.link;
            s.size = base.size * 0.8;
        }
        None => {}
    }
    s
}

/// A selectable text block made of styled spans.
fn text_block(inlines: &[Inline], base: &SpanStyle, options: &RenderOptions) -> impl Scene + use<> {
    let spans: Scenes = inlines
        .iter()
        .map(|inline| {
            let style = inline_style(inline, base, options);
            span_scene(inline.text.clone(), &style, inline.link.clone().map(MdLink))
        })
        .collect();
    let style = base.scene();
    let root = Text::default();
    let label = Name::new(truncate(&inline_text(inlines), 40));
    bsn! {
        root @{style} label SelectableText
        Children [ {spans} ]
    }
}

fn truncate(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        Some((i, _)) => format!("{}…", &s[..i]),
        None => s.to_owned(),
    }
}

//! Builds Bevy UI scenes (BSN) from an [`MdDocument`].

use bevy::{
    clipboard::Clipboard,
    input::mouse::MouseScrollPixelsPerLine,
    picking::events::PointerScroll,
    prelude::*,
    text::{FontSource, FontStyle, FontWeight, LineHeight},
};
use bevy_ui_textselect::{ClickableSpan, CopyText, SelectableText, SelectionGroup};

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

/// A top-level block of a rendered document. Blocks scrolled far out of view are hidden
/// (see [`CommonMarkPlugin`](crate::CommonMarkPlugin)) so they cost nothing to render.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct MdBlock;

/// Options for [`markdown_scene`].
pub struct RenderOptions<'a> {
    /// Visual theme.
    pub theme: &'a MdTheme,
    /// Whether a wikilink target exists (unresolved links are dimmed).
    pub resolve: &'a dyn Fn(&WikiLink) -> bool,
    /// Shown before the document title (its first top-level level-1 heading), e.g. a note
    /// icon, top-aligned with it. Called with the heading's font size and line height (px).
    pub title_badge: Option<&'a dyn Fn(f32, f32) -> Box<dyn Scene>>,
    /// Shown above a code block when it returns one, e.g. a live preview of the code. The
    /// two share one frame under an [`MdCodeGroup`]; the preview's actions go in the code
    /// block's header, and the code's lines can be highlighted through the
    /// [`CodeHighlight`] in the group.
    pub code_preview: Option<&'a dyn Fn(&CodeBlockInfo) -> Option<CodePreview>>,
    /// Draws a named icon (`"copy"`, `"info"`, …) at a size and colour. Without it the copy
    /// button draws its own symbol and callouts have no icon.
    pub icon: Option<&'a dyn Fn(&str, f32, Color) -> Box<dyn Scene>>,
}

/// What [`RenderOptions::code_preview`] shows for a code block.
pub struct CodePreview {
    /// Shown above the code.
    pub body: Box<dyn Scene>,
    /// Buttons in the code block's header, before the copy button.
    pub actions: Option<Box<dyn Scene>>,
}

/// A code block, as passed to [`RenderOptions::code_preview`].
pub struct CodeBlockInfo<'a> {
    /// Info-string language, e.g. `rust`.
    pub lang: Option<&'a str>,
    /// The whole info string, e.g. `rust preview=health`.
    pub info: &'a str,
    /// The code.
    pub code: &'a str,
}

/// Groups a code block with the preview shown above it.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct MdCodeGroup;

/// Lines (0-based, end exclusive) of a code block drawn with a highlight behind them.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeHighlight(pub Vec<core::ops::Range<usize>>);

/// Height of one line of a code block, in logical px.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct CodeLineHeight(pub f32);

/// Space (logical px) between a code block's edges and its code.
const CODE_PADDING: Vec2 = Vec2::new(16.0, 12.0);

/// A highlight bar behind code lines (see [`CodeHighlight`]).
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct CodeHighlightBar;

type Scenes = Vec<Box<dyn Scene>>;

/// Renders a document as a column of selectable blocks.
///
/// The root is a [`SelectionGroup`], so a drag can select across all blocks.
pub fn markdown_scene(doc: &MdDocument, options: &RenderOptions) -> impl Scene + use<> {
    let theme = options.theme;
    let title = doc
        .blocks
        .iter()
        .position(|b| matches!(b, Block::Heading { level: 1, .. }));
    let blocks: Scenes = doc
        .blocks
        .iter()
        .enumerate()
        .map(|(i, block)| match (block, options.title_badge) {
            (
                Block::Heading {
                    level,
                    inlines,
                    slug,
                },
                Some(badge),
            ) if Some(i) == title => heading_block(
                *level,
                inlines,
                slug,
                options,
                Some(badge(
                    theme.heading_size(1),
                    theme.heading_size(1) * theme.line_height,
                )),
            ),
            _ => render_block(block, options, 0),
        })
        .map(|block| Box::new(bsn! { @{block} MdBlock }) as Box<dyn Scene>)
        .collect();
    let node = Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.block_gap),
        width: Val::Percent(100.0),
        min_width: Val::Px(0.0),
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

fn heading_block(
    level: u8,
    inlines: &[Inline],
    slug: &str,
    options: &RenderOptions,
    badge: Option<Box<dyn Scene>>,
) -> Box<dyn Scene> {
    let theme = options.theme;
    let size = theme.heading_size(level);
    let base = SpanStyle {
        font: theme.body.clone(),
        size,
        weight: if level <= 2 {
            theme.heading_weight
        } else {
            FontWeight::SEMIBOLD
        },
        color: theme.heading,
        ..SpanStyle::body(theme)
    };
    let heading = MdHeading {
        level,
        slug: slug.to_owned(),
    };
    let node = Node {
        flex_direction: FlexDirection::Column,
        margin: UiRect::top(Val::Px(if level <= 2 {
            theme.heading_space
        } else {
            theme.heading_space * 0.5
        })),
        ..default()
    };
    let text = text_block(inlines, &base, options);
    let rule = (level == theme.heading_rule).then(|| {
        let line = Node {
            height: Val::Px(1.0),
            width: Val::Percent(100.0),
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        };
        let bg = BackgroundColor(theme.border);
        bsn! { line bg }
    });
    let title: Box<dyn Scene> = match badge {
        Some(badge) => {
            let row = Node {
                align_items: AlignItems::FlexStart,
                column_gap: Val::Px(size * 0.45),
                ..default()
            };
            Box::new(
                bsn! { row Children [ @{badge} -- @{text} heading Node { flex_shrink: 1.0 } ] },
            )
        }
        None => Box::new(bsn! { @{text} heading }),
    };
    Box::new(bsn! {
        node
        Children [
            @{title}
            --
            {rule}
        ]
    })
}

#[allow(clippy::too_many_lines)]
fn render_block(block: &Block, options: &RenderOptions, depth: usize) -> Box<dyn Scene> {
    let theme = options.theme;
    match block {
        Block::Heading {
            level,
            inlines,
            slug,
        } => heading_block(*level, inlines, slug, options, None),
        Block::Paragraph(inlines) => {
            let base = SpanStyle::body(theme);
            Box::new(text_block(inlines, &base, options))
        }
        Block::CodeBlock { lang, info, code } => {
            let info = CodeBlockInfo {
                lang: lang.as_deref(),
                info,
                code,
            };
            match options.code_preview.and_then(|preview| preview(&info)) {
                Some(CodePreview { body, actions }) => {
                    let block = code_block(lang.as_deref(), code, options, actions, true);
                    let frame = Node {
                        flex_direction: FlexDirection::Column,
                        min_width: Val::Px(0.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(theme.radius + 2.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    };
                    let border = BorderColor::all(theme.border);
                    // Children are clipped to a rectangle: round the preview's top corners
                    // too, or its background hides the frame's.
                    let top = BorderRadius::top(Val::Px(theme.radius + 1.0));
                    Box::new(bsn! {
                        frame border
                        MdCodeGroup
                        Children [ @{body} Node { border_radius: top } -- @{block} ]
                    })
                }
                None => code_block(lang.as_deref(), code, options, None, false),
            }
        }
        Block::Quote {
            callout: Some(callout),
            blocks,
        } => callout_block(*callout, blocks, options, depth),
        Block::Quote {
            callout: None,
            blocks,
        } => {
            let children = render_blocks(blocks, options, depth);
            let node = Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(theme.block_gap * 0.6),
                padding: UiRect::new(Val::Px(16.0), Val::Px(0.0), Val::Px(2.0), Val::Px(2.0)),
                border: UiRect::left(Val::Px(2.0)),
                min_width: Val::Px(0.0),
                ..default()
            };
            let border = BorderColor::all(theme.border);
            Box::new(bsn! { node border Children [ {children} ] })
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
        Block::Html(html) => code_block(Some("html"), html, options, None, false),
    }
}

/// A callout (`> [!TIP]`): its icon, then its title over its content, in a bordered box.
/// The accent colours the icon and title, and the border unless it is the text colour.
fn callout_block(
    callout: Callout,
    blocks: &[Block],
    options: &RenderOptions,
    depth: usize,
) -> Box<dyn Scene> {
    let theme = options.theme;
    let accent = theme.callout_color(callout);
    let title_style = SpanStyle {
        weight: FontWeight::MEDIUM,
        color: accent,
        ..SpanStyle::body(theme)
    }
    .scene();
    let title = Text::new(callout.title());
    let content = render_blocks(blocks, options, depth);
    let icon: Scenes = options
        .icon
        .map(|icon| {
            let line = theme.base_size * theme.line_height;
            let icon = icon(callout.icon(), 16.0, accent);
            let top = ((line - 16.0) / 2.0).max(0.0);
            Box::new(bsn! { @{icon} Node { margin: UiRect::top(Val::Px(top)), flex_shrink: 0.0 } })
                as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    let node = Node {
        column_gap: Val::Px(12.0),
        padding: UiRect::axes(Val::Px(16.0), Val::Px(14.0)),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(theme.radius)),
        min_width: Val::Px(0.0),
        ..default()
    };
    let border = BorderColor::all(if accent == theme.heading || accent == theme.text {
        theme.border
    } else {
        accent.with_alpha(0.1)
    });
    let bg = BackgroundColor(theme.surface);
    let column = Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.block_gap * 0.4),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        min_width: Val::Px(0.0),
        ..default()
    };
    Box::new(bsn! {
        node bg border
        Children [
            {icon}
            --
            column
            Children [
                title @{title_style} SelectableText
                --
                {content}
            ]
        ]
    })
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
                checkbox(checked, options)
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

/// A task-list box, centred on the item's first line. Checked boxes are filled with the link
/// colour and show a check mark: the `"check"` icon when [`RenderOptions::icon`] is set, or
/// two drawn strokes otherwise (fonts rarely have a `✓`).
fn checkbox(checked: bool, options: &RenderOptions) -> Box<dyn Scene> {
    let theme = options.theme;
    let size = 16.0;
    let line = theme.base_size * theme.line_height;
    let node = Node {
        width: Val::Px(size),
        height: Val::Px(size),
        margin: UiRect::top(Val::Px(((line - size) / 2.0).max(0.0))),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_shrink: 0.0,
        ..default()
    };
    let border = BorderColor::all(if checked { theme.link } else { theme.muted });
    let bg = BackgroundColor(if checked { theme.link } else { Color::NONE });
    let mark = Color::srgb(0.04, 0.04, 0.04);
    let tick: Scenes = match (checked, options.icon) {
        (false, _) => Vec::new(),
        (true, Some(icon)) => vec![icon("check", 12.0, mark)],
        (true, None) => {
            let stroke = |left: f32, top: f32, width: f32, angle: f32| {
                let node = Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left),
                    top: Val::Px(top),
                    width: Val::Px(width),
                    height: Val::Px(2.0),
                    ..default()
                };
                let turn = UiTransform::from_rotation(Rot2::degrees(angle));
                Box::new(bsn! { node turn BackgroundColor(mark) }) as Box<dyn Scene>
            };
            vec![stroke(2.0, 7.5, 5.0, 45.0), stroke(4.5, 6.0, 8.0, -50.0)]
        }
    };
    Box::new(bsn! { node border bg Children [ {tick} ] })
}

/// A table (shadcn/ui `Table`): a bordered grid whose rows light up under the pointer. Wide
/// tables scroll sideways inside their border instead of overflowing the page.
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
    // The header's background must not cover the frame's rounded corners.
    let corner = Val::Px((theme.radius - 1.0).max(0.0));
    let mut cells: Scenes = Vec::new();
    let mut push_row = |row: &[Vec<Inline>], is_header: bool, row_index: usize| {
        for col in 0..columns {
            let empty = Vec::new();
            let inlines = row.get(col).unwrap_or(&empty);
            let mut style = SpanStyle::body(theme);
            if is_header {
                style.weight = FontWeight::MEDIUM;
                style.color = theme.table_header;
            }
            let text = text_block(inlines, &style, options);
            let justify = match align.get(col).copied().unwrap_or_default() {
                ColumnAlign::Center => JustifyContent::Center,
                ColumnAlign::Right => JustifyContent::End,
                _ => JustifyContent::Start,
            };
            let mut radius = BorderRadius::ZERO;
            if is_header && col == 0 {
                radius.top_left = corner.into();
            }
            if is_header && col == columns - 1 {
                radius.top_right = corner.into();
            }
            let node = Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(if is_header { 8.0 } else { 10.0 })),
                border: UiRect::top(Val::Px(if row_index == 0 { 0.0 } else { 1.0 })),
                border_radius: radius,
                justify_content: justify,
                align_items: AlignItems::Center,
                ..default()
            };
            let border = BorderColor::all(theme.border);
            let bg = BackgroundColor(if is_header {
                theme.table_header_bg
            } else {
                Color::NONE
            });
            let cell = TableCell {
                row: (!is_header).then_some(row_index),
            };
            cells.push(Box::new(bsn! { node border bg cell Children [ @{text} ] }));
        }
    };
    push_row(header, true, 0);
    for (i, row) in rows.iter().enumerate() {
        push_row(row, false, i + 1);
    }
    let grid = Node {
        display: Display::Grid,
        grid_template_columns: vec![RepeatedGridTrack::auto(columns as u16)],
        // As wide as its content, down to the narrowest the columns can wrap to.
        flex_shrink: 1.0,
        ..default()
    };
    let frame = Node {
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(theme.radius)),
        overflow: Overflow::scroll_x(),
        align_self: AlignSelf::Start,
        max_width: Val::Percent(100.0),
        min_width: Val::Px(0.0),
        ..default()
    };
    let border = BorderColor::all(theme.border);
    let hover = MdTable {
        row_hover: theme.table_row_hover,
    };
    Box::new(bsn! {
        frame border
        // Not a `ScrollArea`: that would swallow vertical wheel scrolling over the table.
        ScrollPosition
        on(scroll_code_sideways)
        Children [ grid hover Children [ {cells} ] ]
    })
}

/// A table's grid, with the colour of its hovered row.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct MdTable {
    row_hover: Color,
}

/// A table cell and its body row (`None` in the header).
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct TableCell {
    row: Option<usize>,
}

/// Lights up the body row under the pointer, like shadcn/ui's `hover:bg-muted/50` rows.
pub(crate) fn hover_table_rows(
    hover: Res<bevy::picking::hover::HoverMap>,
    parents: Query<&ChildOf>,
    cells: Query<&TableCell>,
    tables: Query<(&MdTable, &Children)>,
    mut backgrounds: Query<&mut BackgroundColor>,
    mut lit: Local<Option<(Entity, usize)>>,
) {
    // The table and row of the cell under the pointer, if any.
    let current = hover
        .values()
        .flat_map(|hits| hits.keys())
        .find_map(|&hit| {
            std::iter::once(hit)
                .chain(parents.iter_ancestors(hit))
                .find_map(|e| Some((e, cells.get(e).ok()?.row?)))
                .and_then(|(cell, row)| Some((parents.get(cell).ok()?.parent(), row)))
        });
    if *lit == current {
        return;
    }
    let mut paint = |(table, row): (Entity, usize), lit: bool| {
        let Ok((style, children)) = tables.get(table) else {
            return;
        };
        for &child in children {
            if cells.get(child).is_ok_and(|c| c.row == Some(row))
                && let Ok(mut background) = backgrounds.get_mut(child)
            {
                background.0 = if lit { style.row_hover } else { Color::NONE };
            }
        }
    };
    if let Some(previous) = *lit {
        paint(previous, false);
    }
    if let Some(next) = current {
        paint(next, true);
    }
    *lit = current;
}

/// A code block: a header with the language, `actions` and a copy button, over the code.
/// `framed` blocks sit inside a frame shared with a preview and draw only a top border.
fn code_block(
    lang: Option<&str>,
    code: &str,
    options: &RenderOptions,
    actions: Option<Box<dyn Scene>>,
    framed: bool,
) -> Box<dyn Scene> {
    let theme = options.theme;
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
    let line_height = CodeLineHeight(style.size * style.line_height);
    let highlight_color = theme.highlight;
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
            min_width: Val::Px(0.0),
        }
        // Not a `ScrollArea`: that would swallow vertical wheel scrolling over the block.
        ScrollPosition
        on(scroll_code_sideways)
        Children [
            // The padded code, at least as wide as the block: highlight bars span it and
            // scroll sideways with the text.
            Node {
                min_width: Val::Percent(100.0),
                flex_shrink: 0.0,
                padding: UiRect::axes(Val::Px(CODE_PADDING.x), Val::Px(CODE_PADDING.y)),
            }
            CodeHighlight
            line_height
            HighlightColor(highlight_color)
            Children [
                root @{style_scene} no_wrap SelectableText
                Node { flex_shrink: 0.0 }
                Children [ {spans} ]
            ]
        ]
    });

    let copy_source = code.to_owned();
    let label_style = SpanStyle {
        font: theme.mono.clone(),
        size: theme.base_size * 0.75,
        color: theme.muted,
        ..SpanStyle::body(theme)
    };
    let label_scene = label_style.scene();
    let lang_label = Text::new(lang.unwrap_or("").to_owned());
    let node = Node {
        flex_direction: FlexDirection::Column,
        min_width: Val::Px(0.0),
        overflow: Overflow::clip(),
        border: if framed {
            UiRect::top(Val::Px(1.0))
        } else {
            UiRect::all(Val::Px(1.0))
        },
        // In a frame (rounded `radius + 2`, 1 px border) only the bottom corners show.
        border_radius: if framed {
            BorderRadius::bottom(Val::Px(theme.radius + 1.0))
        } else {
            BorderRadius::all(Val::Px(theme.radius))
        },
        ..default()
    };
    // The header's background would otherwise hide the block's rounded top corners.
    let header_radius = if framed {
        BorderRadius::ZERO
    } else {
        BorderRadius::top(Val::Px((theme.radius - 1.0).max(0.0)))
    };
    let border = BorderColor::all(theme.border);
    let bg = BackgroundColor(theme.code_block_bg);
    let header_bg = BackgroundColor(theme.surface);
    let header_border = BorderColor::all(theme.border);
    let icon: Box<dyn Scene> = match options.icon {
        Some(icon) => icon("copy", 14.0, theme.muted),
        None => Box::new(copy_icon(theme)),
    };
    let actions: Scenes = actions.into_iter().collect();
    let bubble_bg = theme.border;
    let bubble_text = theme.text;
    let bubble_font = label_style.font.clone();
    Box::new(bsn! {
        node bg border SelectionGroup
        Children [
            Node {
                height: Val::Px(44.0),
                padding: UiRect::new(Val::Px(14.0), Val::Px(8.0), Val::Px(0.0), Val::Px(0.0)),
                column_gap: Val::Px(4.0),
                align_items: AlignItems::Center,
                border: UiRect::bottom(Val::Px(1.0)),
                border_radius: header_radius,
            }
            header_bg
            header_border
            Children [
                lang_label @{label_scene} Node { flex_grow: 1.0 }
                --
                {actions}
                --
                MdCopyButton
                Node {
                    width: Val::Px(28.0),
                    height: Val::Px(28.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                }
                bevy::ui_widgets::Button
                bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
                // `Activate` (not a raw click): a touch that scrolls the page does not copy.
                on(move |click: On<bevy::ui_widgets::Activate>,
                         mut clipboard: ResMut<Clipboard>,
                         time: Res<Time<Real>>,
                         bubbles: Query<(Entity, &ChildOf), With<CopiedBubble>>,
                         mut commands: Commands| {
                    if let Err(err) = clipboard.set_text(copy_source.clone()) {
                        warn!("copy failed: {err:?}");
                        return;
                    }
                    let until = time.elapsed_secs_f64() + COPIED_SECONDS;
                    // Restart an already visible bubble instead of stacking another.
                    for (bubble, parent) in &bubbles {
                        if parent.parent() == click.entity {
                            commands.entity(bubble).insert(CopiedBubble { until });
                            return;
                        }
                    }
                    commands.spawn((
                        CopiedBubble { until },
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(34.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(bubble_bg),
                        Pickable::IGNORE,
                        ChildOf(click.entity),
                        children![(
                            Text::new("Copied!"),
                            TextFont {
                                font: bubble_font.clone(),
                                font_size: 12.0.into(),
                                ..default()
                            },
                            TextColor(bubble_text),
                            TextLayout::no_wrap(),
                            Pickable::IGNORE,
                        )],
                    ));
                })
                Children [ @{icon} ]
            ]
            --
            @{code_text}
        ]
    })
}

/// How long the "Copied!" bubble stays.
const COPIED_SECONDS: f64 = 1.2;

/// The "Copied!" bubble next to a code block's copy button; removed at `until`
/// (seconds of real time).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CopiedBubble {
    /// When to remove it.
    pub until: f64,
}

/// Colour of the [`CodeHighlight`] bars.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct HighlightColor(pub Color);

/// Redraws the bars behind highlighted code lines when the [`CodeHighlight`] changes.
pub(crate) fn draw_code_highlights(
    blocks: Query<
        (
            Entity,
            &CodeHighlight,
            &CodeLineHeight,
            &HighlightColor,
            Option<&Children>,
        ),
        Changed<CodeHighlight>,
    >,
    bars: Query<(), With<CodeHighlightBar>>,
    mut commands: Commands,
) {
    for (block, highlight, line, color, children) in &blocks {
        for &child in children.into_iter().flatten() {
            if bars.contains(child) {
                commands.entity(child).despawn();
            }
        }
        for lines in &highlight.0 {
            commands.spawn((
                CodeHighlightBar,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    // Absolute insets start at the padding box, the text inside the padding.
                    top: Val::Px(CODE_PADDING.y + lines.start as f32 * line.0),
                    height: Val::Px(lines.len() as f32 * line.0),
                    ..default()
                },
                BackgroundColor(color.0),
                // Behind the code text.
                ZIndex(-1),
                Pickable::IGNORE,
                ChildOf(block),
            ));
        }
    }
}

/// Removes expired "Copied!" bubbles, keeping frames coming while one is shown.
pub(crate) fn expire_copied_bubbles(
    time: Res<Time<Real>>,
    bubbles: Query<(Entity, &CopiedBubble)>,
    mut commands: Commands,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    for (entity, bubble) in &bubbles {
        if time.elapsed_secs_f64() >= bubble.until {
            commands.entity(entity).despawn();
        } else {
            redraw.write(bevy::window::RequestRedraw);
        }
    }
}

/// Two overlapping outlined sheets: the usual "copy" symbol, drawn with UI nodes.
fn copy_icon(theme: &MdTheme) -> impl Scene + use<> {
    let color = BorderColor::all(theme.muted);
    let front_bg = BackgroundColor(theme.surface);
    let sheet = |left: f32, top: f32| Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left),
        top: Val::Px(top),
        width: Val::Px(10.0),
        height: Val::Px(12.0),
        border: UiRect::all(Val::Px(1.5)),
        border_radius: BorderRadius::all(Val::Px(2.5)),
        ..default()
    };
    let (back, front) = (sheet(0.0, 0.0), sheet(4.0, 4.0));
    bsn! {
        Node { width: Val::Px(14.0), height: Val::Px(16.0) }
        Pickable::IGNORE
        Children [
            back color Pickable::IGNORE
            --
            front color front_bg Pickable::IGNORE
        ]
    }
}

/// Wheel scrolling over a code block: sideways deltas (or Shift + wheel) scroll the code;
/// vertical ones bubble up and scroll the page.
fn scroll_code_sideways(
    mut scroll: On<PointerScroll>,
    keys: Res<ButtonInput<KeyCode>>,
    pixels_per_line: Res<MouseScrollPixelsPerLine>,
    mut areas: Query<(&ComputedNode, &mut ScrollPosition)>,
) {
    let delta = scroll.to_pixels(&pixels_per_line);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let sideways = if shift && delta.x == 0.0 {
        delta.y
    } else if delta.x.abs() > delta.y.abs() {
        delta.x
    } else {
        return;
    };
    let Ok((computed, mut position)) = areas.get_mut(scroll.entity) else {
        return;
    };
    let max =
        ((computed.content_size().x - computed.size().x) * computed.inverse_scale_factor).max(0.0);
    if max <= 0.0 {
        return;
    }
    scroll.propagate(false);
    position.x = (position.x - sideways).clamp(0.0, max);
}

/// The copy button of a code block.
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

/// A no-break space in the body font with `chip`'s size and background: side padding for a
/// chip span. It is left out when the selection is copied.
fn pad_span(chip: &SpanStyle, base: &SpanStyle) -> Box<dyn Scene> {
    let style = SpanStyle {
        font: base.font.clone(),
        weight: FontWeight::NORMAL,
        italic: false,
        strike: false,
        underline: false,
        ..chip.clone()
    };
    let pad = span_scene("\u{a0}".to_owned(), &style, None);
    Box::new(bsn! { @{pad} CopyText(String::new()) })
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
        // API references: a code chip (when written in backticks) in the link colour.
        Some(LinkTarget::Docs(_)) => {
            s.color = theme.link;
            s.underline = !inline.style.code;
        }
        None => {}
    }
    s
}

/// A selectable text block made of styled spans.
fn text_block(inlines: &[Inline], base: &SpanStyle, options: &RenderOptions) -> impl Scene + use<> {
    let spans: Scenes = inlines
        .iter()
        .flat_map(|inline| {
            let style = inline_style(inline, base, options);
            let span = span_scene(inline.text.clone(), &style, inline.link.clone().map(MdLink));
            // Spans can't have padding: chips (inline code, tags) get a no-break space of
            // their background on each side instead, which copies as nothing.
            match style.background {
                Some(_) => vec![pad_span(&style, base), span, pad_span(&style, base)],
                None => vec![span],
            }
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

//! Engine-independent markdown document model and parser.
//!
//! [`MdDocument::parse`] turns `CommonMark` (plus GFM tables, task lists, strikethrough,
//! footnotes, callouts and Obsidian `[[wikilinks]]` / `#tags`) into a small tree of
//! [`Block`]s and styled [`Inline`] runs that the scene builder renders.

use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, LinkType, MetadataBlockKind,
    Options, Parser, Tag, TagEnd,
};

/// A parsed markdown document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MdDocument {
    /// The first level-1 heading, if any.
    pub title: Option<String>,
    /// Top-level blocks in source order.
    pub blocks: Vec<Block>,
    /// Every `[[wikilink]]` in the document, in order of appearance (duplicates kept).
    pub links: Vec<WikiLink>,
    /// Every distinct `#tag` (without `#`), in order of first appearance.
    /// Includes tags from a YAML front-matter `tags:` list.
    pub tags: Vec<String>,
    /// Single-line `key: value` properties of the YAML front matter (e.g. `icon: rocket`),
    /// in order, with quotes removed. Lists such as `tags` are not included.
    pub properties: Vec<(String, String)>,
    /// Every heading as `(level, text, slug)`, for outlines.
    pub headings: Vec<Heading>,
}

/// A heading entry of a document outline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    /// 1..=6
    pub level: u8,
    /// Plain heading text.
    pub text: String,
    /// Anchor slug, as used by `[[Page#Heading]]`.
    pub slug: String,
}

/// An Obsidian wikilink: `[[target#heading|alias]]`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WikiLink {
    /// Note name the link points at.
    pub target: String,
    /// Optional heading inside the target note.
    pub heading: Option<String>,
}

/// Block-level element.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    /// `# Heading`
    Heading {
        /// 1..=6
        level: u8,
        /// Styled heading content.
        inlines: Vec<Inline>,
        /// Anchor slug.
        slug: String,
    },
    /// A paragraph of styled text.
    Paragraph(Vec<Inline>),
    /// Fenced or indented code.
    CodeBlock {
        /// Info-string language, e.g. `rust`.
        lang: Option<String>,
        /// Raw code (trailing newline trimmed).
        code: String,
    },
    /// `> quote`, or a callout like `> [!NOTE]`.
    Quote {
        /// Callout kind, when the quote is a GFM/Obsidian callout.
        callout: Option<Callout>,
        /// Quoted blocks.
        blocks: Vec<Block>,
    },
    /// Ordered (`start` is `Some`) or bullet list.
    List {
        /// First number of an ordered list.
        start: Option<u64>,
        /// The list items.
        items: Vec<ListItem>,
    },
    /// GFM table.
    Table {
        /// Column alignments.
        align: Vec<ColumnAlign>,
        /// Header cells.
        header: Vec<Vec<Inline>>,
        /// Body rows of cells.
        rows: Vec<Vec<Vec<Inline>>>,
    },
    /// `---`
    Rule,
    /// Footnote definition `[^label]: ...`.
    Footnote {
        /// Footnote label.
        label: String,
        /// Footnote content.
        blocks: Vec<Block>,
    },
    /// Raw HTML block, displayed verbatim.
    Html(String),
}

/// A list item.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListItem {
    /// `Some(checked)` for task list items.
    pub task: Option<bool>,
    /// Item content.
    pub blocks: Vec<Block>,
}

/// Callout flavour of a block quote.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Callout {
    /// `[!NOTE]`
    Note,
    /// `[!TIP]`
    Tip,
    /// `[!IMPORTANT]`
    Important,
    /// `[!WARNING]`
    Warning,
    /// `[!CAUTION]`
    Caution,
}

impl Callout {
    /// Human-readable title.
    pub fn title(self) -> &'static str {
        match self {
            Callout::Note => "Note",
            Callout::Tip => "Tip",
            Callout::Important => "Important",
            Callout::Warning => "Warning",
            Callout::Caution => "Caution",
        }
    }
}

/// Table column alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColumnAlign {
    /// Not specified.
    #[default]
    None,
    /// `:--`
    Left,
    /// `:-:`
    Center,
    /// `--:`
    Right,
}

/// Inline style flags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)]
pub struct InlineStyle {
    /// `**strong**`
    pub strong: bool,
    /// `*emphasis*`
    pub emphasis: bool,
    /// `~~strike~~`
    pub strike: bool,
    /// `` `code` ``
    pub code: bool,
}

/// Where an inline run links to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LinkTarget {
    /// `[[Note]]`, `[[Note#Heading]]`
    Wiki(WikiLink),
    /// `#tag` (stored without `#`)
    Tag(String),
    /// `[text](https://…)` or an autolink
    Url(String),
    /// `[^label]`
    Footnote(String),
}

/// A run of text sharing one style and link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inline {
    /// Displayed text.
    pub text: String,
    /// Style flags.
    pub style: InlineStyle,
    /// Link, if the run is clickable.
    pub link: Option<LinkTarget>,
}

impl Inline {
    /// Plain unstyled text.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: InlineStyle::default(),
            link: None,
        }
    }
}

/// Concatenates the text of inline runs.
pub fn inline_text(inlines: &[Inline]) -> String {
    inlines.iter().map(|i| i.text.as_str()).collect()
}

/// Lowercase, dash-separated anchor slug for heading text.
pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut dash = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            if dash && !slug.is_empty() {
                slug.push('-');
            }
            dash = false;
            slug.extend(c.to_lowercase());
        } else {
            dash = true;
        }
    }
    slug
}

/// Splits `[[target#heading]]` destinations.
pub fn parse_wikilink(dest: &str) -> WikiLink {
    match dest.split_once('#') {
        Some((target, heading)) => WikiLink {
            target: target.trim().to_owned(),
            heading: Some(heading.trim().to_owned()).filter(|h| !h.is_empty()),
        },
        None => WikiLink {
            target: dest.trim().to_owned(),
            heading: None,
        },
    }
}

fn is_tag_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '/')
}

/// Splits `text` into plain runs and `#tag` runs (Obsidian rules: a tag starts after
/// whitespace or at the start, and contains at least one non-digit).
pub fn split_tags(text: &str) -> Vec<(&str, Option<&str>)> {
    let mut out = Vec::new();
    let mut plain_start = 0;
    let mut chars = text.char_indices().peekable();
    let mut prev: Option<char> = None;
    while let Some((i, c)) = chars.next() {
        if c == '#' && prev.is_none_or(char::is_whitespace) {
            let start = i + 1;
            let mut end = start;
            while let Some(&(j, n)) = chars.peek() {
                if is_tag_char(n) {
                    end = j + n.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            // Trailing '/' or '-' are not part of a tag.
            let tag = text[start..end].trim_end_matches(['/', '-']);
            if !tag.is_empty() && !tag.chars().all(|c| c.is_ascii_digit()) {
                if plain_start < i {
                    out.push((&text[plain_start..i], None));
                }
                let tag_end = start + tag.len();
                out.push((&text[i..tag_end], Some(tag)));
                plain_start = tag_end;
            }
            prev = text[..end].chars().next_back();
            continue;
        }
        prev = Some(c);
    }
    if plain_start < text.len() {
        out.push((&text[plain_start..], None));
    }
    out
}

/// Builder frames for nested containers.
enum Frame {
    Root(Vec<Block>),
    Quote(Option<Callout>, Vec<Block>),
    List(Option<u64>, Vec<ListItem>),
    Item(ListItem),
    Footnote(String, Vec<Block>),
    Table {
        align: Vec<ColumnAlign>,
        header: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
        in_head: bool,
    },
    Row(Vec<Vec<Inline>>),
}

impl Frame {
    fn blocks(&mut self) -> Option<&mut Vec<Block>> {
        match self {
            Frame::Root(b) | Frame::Quote(_, b) | Frame::Footnote(_, b) => Some(b),
            Frame::Item(item) => Some(&mut item.blocks),
            _ => None,
        }
    }
}

#[derive(Default)]
struct InlineState {
    inlines: Vec<Inline>,
    style: InlineStyle,
    link: Vec<Option<LinkTarget>>,
    active: bool,
}

impl InlineState {
    fn push(&mut self, text: &str, style: InlineStyle, link: Option<LinkTarget>) {
        if text.is_empty() {
            return;
        }
        if let Some(last) = self.inlines.last_mut()
            && last.style == style
            && last.link == link
        {
            last.text.push_str(text);
            return;
        }
        self.inlines.push(Inline {
            text: text.to_owned(),
            style,
            link,
        });
    }

    fn current_link(&self) -> Option<LinkTarget> {
        self.link.last().cloned().flatten()
    }

    fn text(&mut self, text: &str, tags: &mut Vec<String>, detect_tags: bool) {
        let link = self.current_link();
        if link.is_some() || self.style.code || !detect_tags {
            self.push(text, self.style, link);
            return;
        }
        for (part, tag) in split_tags(text) {
            match tag {
                Some(tag) => {
                    if !tags.iter().any(|t| t == tag) {
                        tags.push(tag.to_owned());
                    }
                    self.push(part, self.style, Some(LinkTarget::Tag(tag.to_owned())));
                }
                None => self.push(part, self.style, None),
            }
        }
    }

    fn take(&mut self) -> Vec<Inline> {
        self.active = false;
        self.link.clear();
        self.style = InlineStyle::default();
        // Trim whitespace at the edges of the block.
        let mut inlines = std::mem::take(&mut self.inlines);
        if let Some(first) = inlines.first_mut() {
            first.text = first.text.trim_start().to_owned();
        }
        if let Some(last) = inlines.last_mut() {
            last.text = last.text.trim_end().to_owned();
        }
        inlines.retain(|i| !i.text.is_empty());
        inlines
    }
}

/// Top-level `key: value` lines of front matter; values are unquoted.
fn front_matter_properties(yaml: &str, properties: &mut Vec<(String, String)>) {
    for line in yaml.lines() {
        if line.starts_with([' ', '\t', '-', '#']) {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if key == "tags" || value.is_empty() || value.starts_with('[') {
            continue;
        }
        let value = value.trim_matches(['"', '\'']);
        properties.push((key.trim().to_owned(), value.to_owned()));
    }
}

fn front_matter_tags(yaml: &str, tags: &mut Vec<String>) {
    // Minimal YAML subset: `tags: [a, b]`, `tags: a b`, or a `tags:` list of `- a`.
    let mut in_list = false;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if in_list {
            if let Some(item) = trimmed.strip_prefix("- ") {
                push_tag(item, tags);
                continue;
            }
            in_list = false;
        }
        if let Some(rest) = trimmed.strip_prefix("tags:") {
            let rest = rest.trim().trim_start_matches('[').trim_end_matches(']');
            if rest.is_empty() {
                in_list = true;
            }
            for item in rest.split([',', ' ']) {
                push_tag(item, tags);
            }
        }
    }
}

fn push_tag(raw: &str, tags: &mut Vec<String>) {
    let tag = raw.trim().trim_matches(['"', '\'']).trim_start_matches('#');
    if !tag.is_empty() && !tags.iter().any(|t| t == tag) {
        tags.push(tag.to_owned());
    }
}

impl MdDocument {
    /// Parses markdown source.
    #[allow(clippy::too_many_lines)]
    pub fn parse(source: &str) -> Self {
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_GFM
            | Options::ENABLE_WIKILINKS
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;

        let mut doc = MdDocument::default();
        let mut stack = vec![Frame::Root(Vec::new())];
        let mut inl = InlineState::default();
        let mut heading: Option<u8> = None;
        let mut code: Option<(Option<String>, String)> = None;
        let mut html: Option<String> = None;
        let mut metadata: Option<String> = None;

        fn push_block(stack: &mut [Frame], block: Block) {
            for frame in stack.iter_mut().rev() {
                if let Some(blocks) = frame.blocks() {
                    blocks.push(block);
                    return;
                }
            }
        }

        /// Flushes loose inline text (tight list items have no paragraph) into a block.
        fn flush_loose(stack: &mut [Frame], inl: &mut InlineState) {
            if inl.active && !inl.inlines.is_empty() {
                let inlines = inl.take();
                if !inlines.is_empty() {
                    push_block(stack, Block::Paragraph(inlines));
                }
            }
            inl.active = false;
        }

        for event in Parser::new_ext(source, options) {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Paragraph => {
                        flush_loose(&mut stack, &mut inl);
                        inl.active = true;
                    }
                    Tag::Heading { level, .. } => {
                        flush_loose(&mut stack, &mut inl);
                        heading = Some(heading_level(level));
                        inl.active = true;
                    }
                    Tag::BlockQuote(kind) => {
                        flush_loose(&mut stack, &mut inl);
                        stack.push(Frame::Quote(kind.map(callout), Vec::new()));
                    }
                    Tag::CodeBlock(kind) => {
                        flush_loose(&mut stack, &mut inl);
                        let lang = match kind {
                            CodeBlockKind::Fenced(info) => info
                                .split_whitespace()
                                .next()
                                .map(str::to_owned)
                                .filter(|s| !s.is_empty()),
                            CodeBlockKind::Indented => None,
                        };
                        code = Some((lang, String::new()));
                    }
                    Tag::HtmlBlock => {
                        flush_loose(&mut stack, &mut inl);
                        html = Some(String::new());
                    }
                    Tag::List(start) => {
                        flush_loose(&mut stack, &mut inl);
                        stack.push(Frame::List(start, Vec::new()));
                    }
                    Tag::Item => {
                        stack.push(Frame::Item(ListItem::default()));
                        inl.active = true;
                    }
                    Tag::FootnoteDefinition(label) => {
                        flush_loose(&mut stack, &mut inl);
                        stack.push(Frame::Footnote(label.to_string(), Vec::new()));
                    }
                    Tag::Table(align) => {
                        stack.push(Frame::Table {
                            align: align
                                .into_iter()
                                .map(|a| match a {
                                    Alignment::None => ColumnAlign::None,
                                    Alignment::Left => ColumnAlign::Left,
                                    Alignment::Center => ColumnAlign::Center,
                                    Alignment::Right => ColumnAlign::Right,
                                })
                                .collect(),
                            header: Vec::new(),
                            rows: Vec::new(),
                            in_head: false,
                        });
                    }
                    Tag::TableHead => {
                        if let Some(Frame::Table { in_head, .. }) = stack.last_mut() {
                            *in_head = true;
                        }
                        stack.push(Frame::Row(Vec::new()));
                    }
                    Tag::TableRow => stack.push(Frame::Row(Vec::new())),
                    Tag::TableCell => inl.active = true,
                    Tag::Emphasis => inl.style.emphasis = true,
                    Tag::Strong => inl.style.strong = true,
                    Tag::Strikethrough => inl.style.strike = true,
                    Tag::Link {
                        link_type,
                        dest_url,
                        ..
                    } => {
                        let target = if let LinkType::WikiLink { .. } = link_type {
                            let link = parse_wikilink(&dest_url);
                            doc.links.push(link.clone());
                            LinkTarget::Wiki(link)
                        } else {
                            LinkTarget::Url(dest_url.to_string())
                        };
                        inl.link.push(Some(target));
                    }
                    Tag::Image { dest_url, .. } => {
                        // Images render as their alt text, linked to the source.
                        inl.link.push(Some(LinkTarget::Url(dest_url.to_string())));
                    }
                    Tag::MetadataBlock(MetadataBlockKind::YamlStyle) => {
                        metadata = Some(String::new());
                    }
                    _ => {}
                },
                Event::End(tag) => match tag {
                    TagEnd::Paragraph => {
                        let inlines = inl.take();
                        if !inlines.is_empty() {
                            push_block(&mut stack, Block::Paragraph(inlines));
                        }
                    }
                    TagEnd::Heading(_) => {
                        let inlines = inl.take();
                        let level = heading.take().unwrap_or(1);
                        let text = inline_text(&inlines);
                        let slug = slugify(&text);
                        if level == 1 && doc.title.is_none() {
                            doc.title = Some(text.clone());
                        }
                        doc.headings.push(Heading {
                            level,
                            text,
                            slug: slug.clone(),
                        });
                        push_block(
                            &mut stack,
                            Block::Heading {
                                level,
                                inlines,
                                slug,
                            },
                        );
                    }
                    TagEnd::BlockQuote(_) => {
                        flush_loose(&mut stack, &mut inl);
                        if let Some(Frame::Quote(callout, blocks)) = stack.pop() {
                            push_block(&mut stack, Block::Quote { callout, blocks });
                        }
                    }
                    TagEnd::CodeBlock => {
                        if let Some((lang, mut text)) = code.take() {
                            if text.ends_with('\n') {
                                text.pop();
                            }
                            push_block(&mut stack, Block::CodeBlock { lang, code: text });
                        }
                    }
                    TagEnd::HtmlBlock => {
                        if let Some(text) = html.take() {
                            push_block(&mut stack, Block::Html(text.trim_end().to_owned()));
                        }
                    }
                    TagEnd::List(_) => {
                        if let Some(Frame::List(start, items)) = stack.pop() {
                            push_block(&mut stack, Block::List { start, items });
                        }
                    }
                    TagEnd::Item => {
                        flush_loose(&mut stack, &mut inl);
                        if let Some(Frame::Item(item)) = stack.pop()
                            && let Some(Frame::List(_, items)) = stack.last_mut()
                        {
                            items.push(item);
                        }
                    }
                    TagEnd::FootnoteDefinition => {
                        flush_loose(&mut stack, &mut inl);
                        if let Some(Frame::Footnote(label, blocks)) = stack.pop() {
                            push_block(&mut stack, Block::Footnote { label, blocks });
                        }
                    }
                    TagEnd::TableCell => {
                        let cell = inl.take();
                        if let Some(Frame::Row(cells)) = stack.last_mut() {
                            cells.push(cell);
                        }
                    }
                    TagEnd::TableHead | TagEnd::TableRow => {
                        if let Some(Frame::Row(cells)) = stack.pop()
                            && let Some(Frame::Table {
                                header,
                                rows,
                                in_head,
                                ..
                            }) = stack.last_mut()
                        {
                            if *in_head {
                                *header = cells;
                                *in_head = false;
                            } else {
                                rows.push(cells);
                            }
                        }
                    }
                    TagEnd::Table => {
                        if let Some(Frame::Table {
                            align,
                            header,
                            rows,
                            ..
                        }) = stack.pop()
                        {
                            push_block(
                                &mut stack,
                                Block::Table {
                                    align,
                                    header,
                                    rows,
                                },
                            );
                        }
                    }
                    TagEnd::Emphasis => inl.style.emphasis = false,
                    TagEnd::Strong => inl.style.strong = false,
                    TagEnd::Strikethrough => inl.style.strike = false,
                    TagEnd::Link | TagEnd::Image => {
                        inl.link.pop();
                    }
                    TagEnd::MetadataBlock(_) => {
                        if let Some(yaml) = metadata.take() {
                            front_matter_tags(&yaml, &mut doc.tags);
                            front_matter_properties(&yaml, &mut doc.properties);
                        }
                    }
                    _ => {}
                },
                Event::Text(text) => {
                    if let Some(yaml) = metadata.as_mut() {
                        yaml.push_str(&text);
                    } else if let Some((_, code)) = code.as_mut() {
                        code.push_str(&text);
                    } else {
                        inl.active = true;
                        inl.text(&text, &mut doc.tags, true);
                    }
                }
                Event::Code(text) => {
                    inl.active = true;
                    let style = InlineStyle {
                        code: true,
                        ..inl.style
                    };
                    let link = inl.current_link();
                    inl.push(&text, style, link);
                }
                Event::InlineMath(text) | Event::DisplayMath(text) => {
                    let style = InlineStyle {
                        code: true,
                        ..inl.style
                    };
                    inl.push(&text, style, None);
                }
                Event::Html(text) => {
                    if let Some(html) = html.as_mut() {
                        html.push_str(&text);
                    }
                }
                Event::InlineHtml(text) => {
                    // `<br>` is the only inline HTML with an obvious plain-text meaning.
                    if text.trim().eq_ignore_ascii_case("<br>")
                        || text.trim().eq_ignore_ascii_case("<br/>")
                    {
                        inl.push("\n", inl.style, None);
                    }
                }
                Event::FootnoteReference(label) => {
                    inl.active = true;
                    inl.push(
                        &format!("[{label}]"),
                        inl.style,
                        Some(LinkTarget::Footnote(label.to_string())),
                    );
                }
                Event::SoftBreak => {
                    let link = inl.current_link();
                    inl.push(" ", inl.style, link);
                }
                Event::HardBreak => inl.push("\n", inl.style, None),
                Event::Rule => {
                    flush_loose(&mut stack, &mut inl);
                    push_block(&mut stack, Block::Rule);
                }
                Event::TaskListMarker(checked) => {
                    if let Some(Frame::Item(item)) = stack.last_mut() {
                        item.task = Some(checked);
                    }
                }
            }
        }
        flush_loose(&mut stack, &mut inl);
        if let Some(Frame::Root(blocks)) = stack.into_iter().next() {
            doc.blocks = blocks;
        }
        doc
    }

    /// Plain-text rendering of the whole document (used for search).
    pub fn plain_text(&self) -> String {
        fn walk(blocks: &[Block], out: &mut String) {
            for block in blocks {
                match block {
                    Block::Heading { inlines, .. } | Block::Paragraph(inlines) => {
                        out.push_str(&inline_text(inlines));
                        out.push('\n');
                    }
                    Block::CodeBlock { code, .. } | Block::Html(code) => {
                        out.push_str(code);
                        out.push('\n');
                    }
                    Block::Quote { blocks, .. } | Block::Footnote { blocks, .. } => {
                        walk(blocks, out);
                    }
                    Block::List { items, .. } => {
                        for item in items {
                            walk(&item.blocks, out);
                        }
                    }
                    Block::Table { header, rows, .. } => {
                        for row in std::iter::once(header).chain(rows) {
                            for cell in row {
                                out.push_str(&inline_text(cell));
                                out.push('\t');
                            }
                            out.push('\n');
                        }
                    }
                    Block::Rule => {}
                }
            }
        }
        let mut out = String::new();
        walk(&self.blocks, &mut out);
        out
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn callout(kind: BlockQuoteKind) -> Callout {
    match kind {
        BlockQuoteKind::Note => Callout::Note,
        BlockQuoteKind::Tip => Callout::Tip,
        BlockQuoteKind::Important => Callout::Important,
        BlockQuoteKind::Warning => Callout::Warning,
        BlockQuoteKind::Caution => Callout::Caution,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wikilinks() {
        let doc = MdDocument::parse(
            "See [[ECS]], [[Rendering#Cameras|the camera]] and [[Scenes and BSN]].",
        );
        assert_eq!(
            doc.links,
            vec![
                WikiLink {
                    target: "ECS".into(),
                    heading: None
                },
                WikiLink {
                    target: "Rendering".into(),
                    heading: Some("Cameras".into())
                },
                WikiLink {
                    target: "Scenes and BSN".into(),
                    heading: None
                },
            ]
        );
        let Block::Paragraph(inlines) = &doc.blocks[0] else {
            panic!("expected paragraph");
        };
        let texts: Vec<_> = inlines.iter().map(|i| i.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "See ",
                "ECS",
                ", ",
                "the camera",
                " and ",
                "Scenes and BSN",
                "."
            ]
        );
        assert!(matches!(inlines[3].link, Some(LinkTarget::Wiki(_))));
    }

    #[test]
    fn tags() {
        let doc = MdDocument::parse("Topics: #ecs, #rendering/2d and issue #42.\n\n`#notatag`");
        assert_eq!(doc.tags, vec!["ecs", "rendering/2d"]);
        assert_eq!(
            split_tags("a #b c#d #1 #e-"),
            vec![
                ("a ", None),
                ("#b", Some("b")),
                (" c#d #1 ", None),
                ("#e", Some("e")),
                ("-", None),
            ]
        );
    }

    #[test]
    fn front_matter() {
        let doc = MdDocument::parse(
            "---\ntags: [beginner, \"ecs\"]\nicon: \"rocket\"\naliases:\n  - a\n---\n# Title\n",
        );
        assert_eq!(doc.tags, vec!["beginner", "ecs"]);
        assert_eq!(
            doc.properties,
            vec![("icon".to_owned(), "rocket".to_owned())]
        );
        assert_eq!(doc.title.as_deref(), Some("Title"));
    }

    #[test]
    fn nested_lists_and_tasks() {
        let doc = MdDocument::parse("- [x] done\n- [ ] todo\n  1. nested\n  2. two\n");
        let Block::List { start: None, items } = &doc.blocks[0] else {
            panic!("expected list: {:?}", doc.blocks);
        };
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].task, Some(true));
        assert_eq!(items[1].task, Some(false));
        assert!(matches!(
            items[1].blocks[1],
            Block::List { start: Some(1), .. }
        ));
    }

    #[test]
    fn blocks() {
        let src = "# Title\n\n> [!TIP]\n> Use **BSN**.\n\n```rust\nfn main() {}\n```\n\n| a | b |\n|:-|-:|\n| 1 | 2 |\n\n---\n\nText[^n]\n\n[^n]: Footnote.\n";
        let doc = MdDocument::parse(src);
        assert!(matches!(doc.blocks[0], Block::Heading { level: 1, .. }));
        assert!(matches!(
            doc.blocks[1],
            Block::Quote {
                callout: Some(Callout::Tip),
                ..
            }
        ));
        assert_eq!(
            doc.blocks[2],
            Block::CodeBlock {
                lang: Some("rust".into()),
                code: "fn main() {}".into()
            }
        );
        let Block::Table { align, rows, .. } = &doc.blocks[3] else {
            panic!("expected table");
        };
        assert_eq!(align, &[ColumnAlign::Left, ColumnAlign::Right]);
        assert_eq!(inline_text(&rows[0][1]), "2");
        assert_eq!(doc.blocks[4], Block::Rule);
        assert!(
            doc.blocks
                .iter()
                .any(|b| matches!(b, Block::Footnote { .. }))
        );
        assert_eq!(slugify("Hello, World!"), "hello-world");
    }
}

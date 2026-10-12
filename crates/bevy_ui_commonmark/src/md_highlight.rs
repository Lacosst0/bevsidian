//! Markdown syntax highlighting for editors: classifies every byte of a source text.
//!
//! This is a fast, line-based approximation of `CommonMark` + GFM + Obsidian syntax (headings,
//! emphasis, code, links, wikilinks, tags, lists, quotes, front matter). It never fails and
//! is meant to be re-run on every edit.

use std::ops::Range;

/// What a highlighted range of markdown source is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MdToken {
    /// Ordinary text.
    Text,
    /// Syntax characters: `#`, `>`, list bullets, `**`, fences, brackets, …
    Marker,
    /// Heading text.
    Heading,
    /// `**strong**` text.
    Strong,
    /// `*emphasis*` text.
    Emphasis,
    /// `~~strikethrough~~` text.
    Strike,
    /// Inline code and code blocks.
    Code,
    /// Link text, wikilinks and bare URLs.
    Link,
    /// The URL part of `[text](url)`.
    Url,
    /// `#tag`
    Tag,
    /// YAML front matter.
    FrontMatter,
    /// Text inside a `>` quote.
    Quote,
}

/// Splits `source` into consecutive ranges covering all of it, each with a token kind.
/// Adjacent ranges of the same kind are merged.
pub fn highlight_markdown(source: &str) -> Vec<(Range<usize>, MdToken)> {
    let mut out = Out::default();
    let mut fence: Option<(u8, usize)> = None;
    let mut front_matter = source.starts_with("---\n") || source.starts_with("---\r\n");
    let mut start = 0;
    let mut first = true;
    while start < source.len() {
        let end = source[start..]
            .find('\n')
            .map_or(source.len(), |i| start + i + 1);
        let line = &source[start..end];
        let content = line.trim_end_matches(['\n', '\r']);
        let trimmed = content.trim_start();
        let indent = content.len() - trimmed.len();

        if front_matter {
            out.push(start..end, MdToken::FrontMatter);
            if !first && content.trim_end() == "---" {
                front_matter = false;
            }
        } else if let Some((ch, len)) = fence {
            // Inside a fenced code block: a fence of the same kind closes it.
            let run = trimmed.bytes().take_while(|&b| b == ch).count();
            if run >= len && trimmed[run..].trim().is_empty() {
                fence = None;
                out.push(start..end, MdToken::Marker);
            } else {
                out.push(start..end, MdToken::Code);
            }
        } else if let Some(open) = fence_open(trimmed) {
            fence = Some(open);
            out.push(start..end, MdToken::Marker);
        } else {
            block_line(source, start, end, content, indent, &mut out);
        }
        first = false;
        start = end;
    }
    out.ranges
}

/// The fence character and length if `line` opens a fenced code block.
fn fence_open(line: &str) -> Option<(u8, usize)> {
    let ch = *line.as_bytes().first()?;
    if ch != b'`' && ch != b'~' {
        return None;
    }
    let run = line.bytes().take_while(|&b| b == ch).count();
    (run >= 3 && !(ch == b'`' && line[run..].contains('`'))).then_some((ch, run))
}

fn block_line(source: &str, start: usize, end: usize, content: &str, indent: usize, out: &mut Out) {
    let mut pos = start + indent;
    let rest = &content[indent..];
    out.push(start..pos, MdToken::Text);

    // Heading: 1–6 `#` and a space.
    let hashes = rest.bytes().take_while(|&b| b == b'#').count();
    if (1..=6).contains(&hashes) && rest[hashes..].starts_with([' ', '\t']) {
        out.push(pos..pos + hashes + 1, MdToken::Marker);
        out.push(pos + hashes + 1..end, MdToken::Heading);
        return;
    }
    // Thematic break.
    let compact: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.len() >= 3
        && (compact.chars().all(|c| c == '-')
            || compact.chars().all(|c| c == '*')
            || compact.chars().all(|c| c == '_'))
    {
        out.push(pos..end, MdToken::Marker);
        return;
    }

    let mut base = MdToken::Text;
    let mut rest = rest;
    // Quote markers (possibly nested).
    while let Some(after) = rest.strip_prefix('>') {
        let len = 1 + usize::from(after.starts_with(' '));
        out.push(pos..pos + len, MdToken::Marker);
        pos += len;
        rest = &rest[len..];
        base = MdToken::Quote;
    }
    // List bullet or number, then an optional task box.
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let bullet = if rest.starts_with(['-', '*', '+']) {
        1
    } else if (1..=9).contains(&digits) && rest[digits..].starts_with(['.', ')']) {
        digits + 1
    } else {
        0
    };
    if bullet > 0 && (rest.len() == bullet || rest[bullet..].starts_with([' ', '\t'])) {
        let mut len = (bullet + 1).min(rest.len());
        for task in ["[ ] ", "[x] ", "[X] "] {
            if rest[len..].starts_with(task) {
                len += task.len();
            }
        }
        out.push(pos..pos + len, MdToken::Marker);
        pos += len;
    }
    inline(source, pos, end, base, out);
}

/// Highlights inline syntax in `source[start..end]`.
fn inline(source: &str, start: usize, end: usize, base: MdToken, out: &mut Out) {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut text_from = start;
    let flush = |out: &mut Out, from: usize, to: usize| out.push(from..to, base);
    while i < end {
        let rest = &source[i..end];
        let prev = (i > start).then(|| bytes[i - 1]);
        let token: Option<Vec<(Range<usize>, MdToken)>> = match bytes[i] {
            b'\\' if i + 1 < end && bytes[i + 1].is_ascii_punctuation() => {
                Some(vec![(i..i + 1, MdToken::Marker), (i + 1..i + 2, base)])
            }
            b'`' => {
                let run = rest.bytes().take_while(|&b| b == b'`').count();
                let fence = &rest[..run];
                rest[run..].find(fence).map(|close| {
                    let body = i + run..i + run + close;
                    vec![
                        (i..i + run, MdToken::Marker),
                        (body.clone(), MdToken::Code),
                        (body.end..body.end + run, MdToken::Marker),
                    ]
                })
            }
            b'[' if rest.starts_with("[[") => rest.find("]]").map(|close| {
                vec![
                    (i..i + 2, MdToken::Marker),
                    (i + 2..i + close, MdToken::Link),
                    (i + close..i + close + 2, MdToken::Marker),
                ]
            }),
            b'!' if rest.starts_with("![") => link(rest, i + 1).map(|mut parts| {
                parts.insert(0, (i..i + 1, MdToken::Marker));
                parts
            }),
            b'[' => link(rest, i),
            b'*' | b'_' | b'~' => delimited(rest, i, prev),
            b'#' if prev.is_none_or(|p| p.is_ascii_whitespace() || p == b'(') => {
                let len = rest[1..]
                    .find(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '/')))
                    .unwrap_or(rest.len() - 1);
                let tag = &rest[1..=len];
                (len > 0 && !tag.chars().all(|c| c.is_ascii_digit()))
                    .then(|| vec![(i..i + 1 + len, MdToken::Tag)])
            }
            b'h' if rest.starts_with("http://") || rest.starts_with("https://") => {
                let len = rest
                    .find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | ')' | ']'))
                    .unwrap_or(rest.len());
                Some(vec![(i..i + len, MdToken::Link)])
            }
            _ => None,
        };
        match token {
            Some(parts) => {
                flush(out, text_from, i);
                let next = parts.last().map_or(i + 1, |(r, _)| r.end).max(i + 1);
                for (range, kind) in parts {
                    out.push(range, kind);
                }
                i = next;
                text_from = i;
            }
            None => i += source[i..].chars().next().map_or(1, char::len_utf8),
        }
    }
    flush(out, text_from, end);
}

/// `[text](url)` starting at `at` (`rest` begins with `[`).
fn link(rest: &str, at: usize) -> Option<Vec<(Range<usize>, MdToken)>> {
    let close = rest.find(']')?;
    let after = &rest[close + 1..];
    if !after.starts_with('(') {
        return None;
    }
    let url_end = after.find(')')?;
    let paren = at + close + 1;
    Some(vec![
        (at..at + 1, MdToken::Marker),
        (at + 1..at + close, MdToken::Link),
        (at + close..paren + 1, MdToken::Marker),
        (paren + 1..paren + url_end, MdToken::Url),
        (paren + url_end..paren + url_end + 1, MdToken::Marker),
    ])
}

/// `**strong**`, `*emphasis*`, `__strong__`, `_emphasis_` or `~~strike~~` at `at`.
fn delimited(rest: &str, at: usize, prev: Option<u8>) -> Option<Vec<(Range<usize>, MdToken)>> {
    let ch = rest.as_bytes()[0];
    let run = rest.bytes().take_while(|&b| b == ch).count();
    let (len, kind) = match (ch, run) {
        (b'~', 2..) => (2, MdToken::Strike),
        (b'~', _) => return None,
        (_, 2..) => (2, MdToken::Strong),
        _ => (1, MdToken::Emphasis),
    };
    // `_` inside words (snake_case) is not emphasis.
    if ch == b'_' && prev.is_some_and(|p| p.is_ascii_alphanumeric()) {
        return None;
    }
    let delim = &rest[..len];
    let body = &rest[len..];
    if body.starts_with(char::is_whitespace) || body.is_empty() {
        return None;
    }
    let close = body.find(delim)?;
    if close == 0 || body[..close].ends_with(char::is_whitespace) {
        return None;
    }
    let body_start = at + len;
    Some(vec![
        (at..body_start, MdToken::Marker),
        (body_start..body_start + close, kind),
        (
            body_start + close..body_start + close + len,
            MdToken::Marker,
        ),
    ])
}

#[derive(Default)]
struct Out {
    ranges: Vec<(Range<usize>, MdToken)>,
}

impl Out {
    fn push(&mut self, range: Range<usize>, kind: MdToken) {
        if range.is_empty() {
            return;
        }
        if let Some((last, last_kind)) = self.ranges.last_mut()
            && *last_kind == kind
            && last.end == range.start
        {
            last.end = range.end;
            return;
        }
        self.ranges.push((range, kind));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<(&str, MdToken)> {
        let ranges = highlight_markdown(source);
        // Ranges cover the source exactly, in order.
        let mut at = 0;
        for (range, _) in &ranges {
            assert_eq!(range.start, at, "{ranges:?}");
            at = range.end;
        }
        assert_eq!(at, source.len());
        ranges.into_iter().map(|(r, k)| (&source[r], k)).collect()
    }

    #[test]
    fn blocks() {
        use MdToken::*;
        assert_eq!(
            kinds("---\nicon: x\n---\n# Title\n"),
            vec![
                ("---\nicon: x\n---\n", FrontMatter),
                ("# ", Marker),
                ("Title\n", Heading)
            ]
        );
        assert_eq!(
            kinds("```rust\nlet a = 1;\n```\nok"),
            vec![
                ("```rust\n", Marker),
                ("let a = 1;\n", Code),
                ("```\n", Marker),
                ("ok", Text)
            ]
        );
        assert_eq!(
            kinds("- [x] done\n> quote"),
            vec![
                ("- [x] ", Marker),
                ("done\n", Text),
                ("> ", Marker),
                ("quote", Quote)
            ]
        );
    }

    #[test]
    fn inlines() {
        use MdToken::*;
        assert_eq!(
            kinds("a **b** *c* `d` [[E]] #tag snake_case_x"),
            vec![
                ("a ", Text),
                ("**", Marker),
                ("b", Strong),
                ("**", Marker),
                (" ", Text),
                ("*", Marker),
                ("c", Emphasis),
                ("*", Marker),
                (" ", Text),
                ("`", Marker),
                ("d", Code),
                ("`", Marker),
                (" ", Text),
                ("[[", Marker),
                ("E", Link),
                ("]]", Marker),
                (" ", Text),
                ("#tag", Tag),
                (" snake_case_x", Text),
            ]
        );
        assert_eq!(
            kinds("[docs](https://bevy.org) 2 * 3 #1"),
            vec![
                ("[", Marker),
                ("docs", Link),
                ("](", Marker),
                ("https://bevy.org", Url),
                (")", Marker),
                (" 2 * 3 #1", Text),
            ]
        );
        // Multi-byte text is kept on char boundaries.
        assert_eq!(kinds("é *ü*")[1], ("*", Marker));
    }
}

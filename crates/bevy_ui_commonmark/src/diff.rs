//! Line diffs between the variants of a code block (see [`Block::CodeVariants`]).
//!
//! A variant is written either as its whole code (diffed against the base line by line) or as
//! a `diff` block: every line of the base prefixed with ` ` (kept), `-` (removed) or `+`
//! (added). Either way it becomes a list of [`DiffLine`]s; an added line that resembles a
//! removed line next to it (an edit of it) is marked changed.
//!
//! [`Block::CodeVariants`]: crate::Block::CodeVariants

/// What happened to a line of a variant, compared with the base.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineKind {
    /// In both.
    #[default]
    Same,
    /// Only in the variant.
    Added,
    /// Only in the base (shown, but not part of the variant's code).
    Removed,
    /// An edit of a removed line next to it.
    Changed,
}

/// A line shown for a variant.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: LineKind,
    pub text: String,
}

/// The lines of `variant` against `base`: a longest-common-subsequence line diff.
pub fn diff(base: &str, variant: &str) -> Vec<DiffLine> {
    let a: Vec<&str> = base.lines().collect();
    let b: Vec<&str> = variant.lines().collect();
    // common[i][j]: length of the longest common subsequence of a[i..] and b[j..].
    let mut common = vec![vec![0_u32; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            common[i][j] = if a[i] == b[j] {
                common[i + 1][j + 1] + 1
            } else {
                common[i + 1][j].max(common[i][j + 1])
            };
        }
    }
    let mut lines = Vec::with_capacity(a.len().max(b.len()));
    let line = |kind, text: &str| DiffLine {
        kind,
        text: text.to_owned(),
    };
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            lines.push(line(LineKind::Same, a[i]));
            i += 1;
            j += 1;
        } else if j < b.len() && (i == a.len() || common[i][j + 1] >= common[i + 1][j]) {
            lines.push(line(LineKind::Added, b[j]));
            j += 1;
        } else {
            lines.push(line(LineKind::Removed, a[i]));
            i += 1;
        }
    }
    pair_changes(order_removals_first(lines))
}

/// The lines of a `diff`-style variant: ` ` kept, `-` removed, `+` added (a line without a
/// marker is kept as it is).
pub fn parse(text: &str) -> Vec<DiffLine> {
    let lines = text
        .lines()
        .map(|line| {
            let (kind, rest) = match line.chars().next() {
                Some('+') => (LineKind::Added, &line[1..]),
                Some('-') => (LineKind::Removed, &line[1..]),
                Some(' ') => (LineKind::Same, &line[1..]),
                _ => (LineKind::Same, line),
            };
            DiffLine {
                kind,
                text: rest.to_owned(),
            }
        })
        .collect();
    pair_changes(lines)
}

/// The variant's own code: its lines without the removed ones.
pub fn code(lines: &[DiffLine]) -> String {
    lines
        .iter()
        .filter(|l| l.kind != LineKind::Removed)
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Everything shown for a variant, removed lines included, one per line.
pub fn shown(lines: &[DiffLine]) -> String {
    lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Within each run of changes, removals come before additions (as diffs read).
fn order_removals_first(lines: Vec<DiffLine>) -> Vec<DiffLine> {
    let mut out = Vec::with_capacity(lines.len());
    let mut run: Vec<DiffLine> = Vec::new();
    for line in lines {
        if line.kind == LineKind::Same {
            run.sort_by_key(|l| l.kind != LineKind::Removed);
            out.append(&mut run);
            out.push(line);
        } else {
            run.push(line);
        }
    }
    run.sort_by_key(|l| l.kind != LineKind::Removed);
    out.append(&mut run);
    out
}

/// In each run of removals and additions, an added line that resembles a removed one (an
/// edit of it) is a change; the others stay added.
fn pair_changes(mut lines: Vec<DiffLine>) -> Vec<DiffLine> {
    let mut start = 0;
    while start < lines.len() {
        if lines[start].kind == LineKind::Same {
            start += 1;
            continue;
        }
        let end = (start..lines.len())
            .find(|&i| lines[i].kind == LineKind::Same)
            .unwrap_or(lines.len());
        let removed: Vec<usize> = (start..end)
            .filter(|&i| lines[i].kind == LineKind::Removed)
            .collect();
        let mut added: Vec<usize> = (start..end)
            .filter(|&i| lines[i].kind == LineKind::Added)
            .collect();
        for r in removed {
            let best = added
                .iter()
                .enumerate()
                .map(|(k, &a)| (k, similarity(&lines[r].text, &lines[a].text)))
                .max_by(|x, y| x.1.total_cmp(&y.1));
            if let Some((k, score)) = best
                && score >= SIMILAR
            {
                lines[added.remove(k)].kind = LineKind::Changed;
            }
        }
        start = end;
    }
    lines
}

/// How alike two lines must be for one to count as an edit of the other.
const SIMILAR: f32 = 0.5;

/// Share of the longer line covered by the start and end the two lines have in common
/// (1 for equal lines, near 1 for a small edit, low for unrelated lines).
fn similarity(a: &str, b: &str) -> f32 {
    let (a, b) = (a.trim(), b.trim());
    let longest = a.chars().count().max(b.chars().count());
    if longest == 0 {
        return 1.0;
    }
    let prefix = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
    let suffix = a
        .chars()
        .rev()
        .zip(b.chars().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let shortest = a.chars().count().min(b.chars().count());
    (prefix + suffix).min(shortest) as f32 / longest as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(lines: &[DiffLine]) -> Vec<LineKind> {
        lines.iter().map(|l| l.kind).collect()
    }

    #[test]
    fn diffs_mark_added_removed_and_changed_lines() {
        use LineKind::{Added, Changed, Removed, Same};
        let lines = diff("a\nlet x = 1;\nc\nd", "a\nlet x = 2;\nc\nd\ne");
        assert_eq!(kinds(&lines), [Same, Removed, Changed, Same, Same, Added]);
        assert_eq!(code(&lines), "a\nlet x = 2;\nc\nd\ne");
        assert_eq!(shown(&lines), "a\nlet x = 1;\nlet x = 2;\nc\nd\ne");
        // A line replaced by an unrelated one is removed and added, not changed.
        assert_eq!(kinds(&diff("old()", "brand_new_line")), [Removed, Added]);
        assert_eq!(kinds(&diff("a\nb", "a")), [Same, Removed]);
        assert_eq!(kinds(&diff("", "x")), [Added]);
    }

    #[test]
    fn diff_blocks_parse_their_markers() {
        use LineKind::{Added, Changed, Removed, Same};
        let lines = parse(" fn main() {\n-    run(1);\n+    // Now twice.\n+    run(2);\n }");
        assert_eq!(
            kinds(&lines),
            [Same, Removed, Added, Changed, Same],
            "the edit, not the comment"
        );
        assert_eq!(
            code(&lines),
            "fn main() {\n    // Now twice.\n    run(2);\n}"
        );
    }
}

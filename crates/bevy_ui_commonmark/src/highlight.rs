//! A tiny, dependency-free syntax highlighter for code blocks.
//!
//! It is a lexer, not a parser: good enough to colour keywords, strings, comments, numbers,
//! types and macros in Rust, TOML, shell, JS and WGSL snippets.

/// Token class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// Anything else.
    Plain,
    /// Language keyword.
    Keyword,
    /// String or char literal.
    String,
    /// Comment.
    Comment,
    /// Number, boolean or constant.
    Number,
    /// Capitalised identifier or macro call.
    Type,
}

const RUST: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "type", "unsafe", "use",
    "where", "while",
];
const WGSL: &[&str] = &[
    "fn",
    "let",
    "var",
    "const",
    "struct",
    "return",
    "if",
    "else",
    "for",
    "loop",
    "while",
    "uniform",
    "storage",
    "read",
    "write",
    "private",
    "workgroup",
    "override",
    "alias",
];
const JS: &[&str] = &[
    "const", "let", "var", "function", "return", "if", "else", "for", "while", "import", "export",
    "from", "await", "async", "new", "class", "extends", "this", "of", "in",
];
const SHELL: &[&str] = &[
    "if", "then", "else", "fi", "for", "do", "done", "case", "esac", "export", "cd", "cargo",
    "rustup", "git", "echo",
];

fn keywords(lang: &str) -> (&'static [&'static str], &'static str) {
    match lang {
        "rust" | "rs" => (RUST, "//"),
        "wgsl" => (WGSL, "//"),
        "js" | "javascript" | "ts" | "typescript" => (JS, "//"),
        "sh" | "bash" | "shell" | "console" => (SHELL, "#"),
        "toml" | "ini" | "yaml" | "yml" => (&[], "#"),
        _ => (&[], ""),
    }
}

/// Splits `code` into coloured tokens. Unknown languages yield one plain token.
pub fn highlight<'a>(code: &'a str, lang: Option<&str>) -> Vec<(&'a str, TokenKind)> {
    let lang = lang.unwrap_or("").to_ascii_lowercase();
    let (kws, line_comment) = keywords(&lang);
    if kws.is_empty() && line_comment.is_empty() {
        return vec![(code, TokenKind::Plain)];
    }
    let toml = matches!(lang.as_str(), "toml" | "ini" | "yaml" | "yml");
    let bytes = code.as_bytes();
    let mut out: Vec<(&str, TokenKind)> = Vec::new();
    let mut push = |s: &'a str, k: TokenKind| {
        if s.is_empty() {
            return;
        }
        if let Some(last) = out.last_mut()
            && last.1 == k
        {
            // Merge adjacent tokens of the same kind (they are contiguous slices).
            let start = last.0.as_ptr() as usize - code.as_ptr() as usize;
            let end = s.as_ptr() as usize - code.as_ptr() as usize + s.len();
            last.0 = &code[start..end];
            return;
        }
        out.push((s, k));
    };
    let mut i = 0;
    let len = bytes.len();
    while i < len {
        let c = bytes[i];
        let rest = &code[i..];
        // Comments
        if !line_comment.is_empty() && rest.starts_with(line_comment) {
            let end = rest.find('\n').map_or(len, |n| i + n);
            push(&code[i..end], TokenKind::Comment);
            i = end;
            continue;
        }
        if rest.starts_with("/*") && line_comment == "//" {
            let end = rest.find("*/").map_or(len, |n| i + n + 2);
            push(&code[i..end], TokenKind::Comment);
            i = end;
            continue;
        }
        // Strings
        if c == b'"' || (c == b'\'' && !matches!(lang.as_str(), "rust" | "rs")) {
            let mut j = i + 1;
            while j < len && bytes[j] != c {
                if bytes[j] == b'\\' {
                    j += 1;
                }
                if bytes[j.min(len - 1)] == b'\n' && c == b'\'' {
                    break;
                }
                j += 1;
            }
            let end = (j + 1).min(len);
            push(&code[i..end], TokenKind::String);
            i = end;
            continue;
        }
        // Numbers
        if c.is_ascii_digit() {
            let mut j = i;
            while j < len
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'.')
            {
                // Stop at range operators like `0..10`.
                if bytes[j] == b'.' && bytes.get(j + 1) == Some(&b'.') {
                    break;
                }
                j += 1;
            }
            push(&code[i..j], TokenKind::Number);
            i = j;
            continue;
        }
        // Identifiers
        if c.is_ascii_alphabetic() || c == b'_' {
            let mut j = i;
            while j < len
                && (bytes[j].is_ascii_alphanumeric()
                    || bytes[j] == b'_'
                    || (toml && bytes[j] == b'-'))
            {
                j += 1;
            }
            let word = &code[i..j];
            let kind = if kws.contains(&word) {
                TokenKind::Keyword
            } else if matches!(word, "true" | "false" | "None" | "Some" | "Ok" | "Err") {
                TokenKind::Number
            } else if bytes.get(j) == Some(&b'!') && line_comment == "//" {
                j += 1;
                TokenKind::Type
            } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                TokenKind::Type
            } else if toml && rest_is_key(&code[j..]) {
                TokenKind::Keyword
            } else {
                TokenKind::Plain
            };
            push(&code[i..j], kind);
            i = j;
            continue;
        }
        // TOML table headers
        if toml && c == b'[' && (i == 0 || bytes[i - 1] == b'\n') {
            let end = rest.find('\n').map_or(len, |n| i + n);
            push(&code[i..end], TokenKind::Type);
            i = end;
            continue;
        }
        let ch_len = rest.chars().next().map_or(1, char::len_utf8);
        push(&code[i..i + ch_len], TokenKind::Plain);
        i += ch_len;
    }
    out
}

fn rest_is_key(rest: &str) -> bool {
    let line = rest.split('\n').next().unwrap_or("");
    line.trim_start().starts_with('=')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust() {
        let tokens = highlight("fn main() { println!(\"hi\"); // c\n}", Some("rust"));
        assert_eq!(tokens[0], ("fn", TokenKind::Keyword));
        assert!(tokens.contains(&("println!", TokenKind::Type)));
        assert!(tokens.contains(&("\"hi\"", TokenKind::String)));
        assert!(tokens.contains(&("// c", TokenKind::Comment)));
        let joined: String = tokens.iter().map(|t| t.0).collect();
        assert_eq!(joined, "fn main() { println!(\"hi\"); // c\n}");
    }

    #[test]
    fn toml() {
        let tokens = highlight("[dependencies]\nbevy = \"0.20\"", Some("toml"));
        assert_eq!(tokens[0], ("[dependencies]", TokenKind::Type));
        assert!(tokens.contains(&("bevy", TokenKind::Keyword)));
    }

    #[test]
    fn unknown() {
        assert_eq!(highlight("x", None), vec![("x", TokenKind::Plain)]);
    }
}

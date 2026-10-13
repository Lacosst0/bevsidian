//! Docs references: rustdoc-style links in notes (`` [`Commands`] ``) open Bevy's API docs
//! on docs.rs.
//!
//! [`url`] finds the item in an index of Bevy's documented items (`data/bevy-docs.txt`,
//! built by `tools/docs_index.py`): its exact page, or a member's anchor on its type's page
//! (`Commands::spawn`). Anything else opens a docs.rs search for it.

use std::{collections::HashMap, sync::OnceLock};

/// Bevy's docs, at the version the notes are written for.
pub const BASE: &str = "https://docs.rs/bevy/0.20/bevy/";

/// `kind path` lines, e.g. `struct ecs::system::Commands`.
const INDEX: &str = include_str!("../data/bevy-docs.txt");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Item {
    kind: &'static str,
    /// Path below `bevy::`, e.g. `ecs::system::Commands`.
    path: &'static str,
}

/// Items by name (the last segment of their path).
fn index() -> &'static HashMap<&'static str, Vec<Item>> {
    static INDEX_BY_NAME: OnceLock<HashMap<&'static str, Vec<Item>>> = OnceLock::new();
    INDEX_BY_NAME.get_or_init(|| {
        let mut by_name: HashMap<&'static str, Vec<Item>> = HashMap::new();
        for line in INDEX.lines() {
            let Some((kind, path)) = line.split_once(' ') else {
                continue;
            };
            let name = path.rsplit("::").next().unwrap_or(path);
            by_name.entry(name).or_default().push(Item { kind, path });
        }
        by_name
    })
}

/// How well an item answers a bare name: types and traits before functions and macros, and
/// derive macros last (`Component` means the trait); then the shortest path.
fn rank(item: &Item) -> (u8, usize, &'static str) {
    let kind = match item.kind {
        "struct" | "enum" | "trait" | "union" | "type" | "primitive" => 0,
        "fn" | "constant" | "static" => 1,
        "macro" => 2,
        _ => 3,
    };
    (kind, item.path.matches("::").count(), item.path)
}

/// The item a path names: `Commands`, `ecs::system::Commands` or `bevy::ecs::system::Commands`
/// (a suffix of its full path), `bsn!` (a macro), `run_once()` (a function).
fn find(path: &str) -> Option<Item> {
    let (path, kinds): (&str, &[&str]) = if let Some(path) = path.strip_suffix('!') {
        (path, &["macro"])
    } else if let Some(path) = path.strip_suffix("()") {
        (path, &["fn"])
    } else {
        (path, &[])
    };
    let path = path.strip_prefix("bevy::").unwrap_or(path);
    let name = path.rsplit("::").next()?;
    index()
        .get(name)?
        .iter()
        .filter(|item| kinds.is_empty() || kinds.contains(&item.kind))
        .filter(|item| item.path == path || item.path.ends_with(&format!("::{path}")))
        .min_by_key(|item| rank(item))
        .copied()
}

fn page(item: Item) -> String {
    let (modules, name) = item.path.rsplit_once("::").unwrap_or(("", item.path));
    let folder = if modules.is_empty() {
        String::new()
    } else {
        format!("{}/", modules.replace("::", "/"))
    };
    format!("{BASE}{folder}{}.{name}.html", item.kind)
}

/// The docs.rs page of a Rust path as written in a note: the item's page, or a search.
pub fn url(path: &str) -> String {
    page_of(path).unwrap_or_else(|| {
        let name = path
            .trim()
            .trim_matches('`')
            .rsplit("::")
            .next()
            .unwrap_or(path);
        format!("{BASE}?search={}", name.trim_end_matches(['!', '(', ')']))
    })
}

/// The docs.rs page of the item a Rust path names, if the index has it.
pub fn page_of(path: &str) -> Option<String> {
    let path = path.trim().trim_matches('`');
    if let Some(item) = find(path) {
        return Some(page(item));
    }
    // A member of a type: `Commands::spawn`, `Val::Px`.
    if let Some((owner, member)) = path.rsplit_once("::")
        && let Some(item) = find(owner)
        && matches!(item.kind, "struct" | "enum" | "trait" | "union")
    {
        let member = member.trim_end_matches("()");
        let anchor = if item.kind == "enum" && member.starts_with(char::is_uppercase) {
            "variant"
        } else {
            "method"
        };
        return Some(format!("{}#{anchor}.{member}", page(item)));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_find_their_pages() {
        assert_eq!(
            url("Commands"),
            format!("{BASE}ecs/system/struct.Commands.html")
        );
        assert_eq!(
            url("`bevy::ecs::system::Commands`"),
            format!("{BASE}ecs/system/struct.Commands.html")
        );
        assert_eq!(
            url("Component"),
            format!("{BASE}ecs/component/trait.Component.html")
        );
        assert_eq!(url("bsn!"), format!("{BASE}scene/macro.bsn.html"));
        assert_eq!(
            url("DefaultPlugins"),
            format!("{BASE}struct.DefaultPlugins.html")
        );
        assert!(url("StandardMaterial").ends_with("/struct.StandardMaterial.html"));
    }

    #[test]
    fn members_link_to_anchors() {
        assert_eq!(
            url("Commands::spawn"),
            format!("{BASE}ecs/system/struct.Commands.html#method.spawn")
        );
        assert!(url("Val::Px").ends_with("/enum.Val.html#variant.Px"));
    }

    #[test]
    fn unknown_names_search() {
        assert_eq!(url("NoSuchThing"), format!("{BASE}?search=NoSuchThing"));
    }
}

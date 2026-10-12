//! The note vault: parsed notes, resolved links, backlinks and tags.

use std::{collections::BTreeMap, sync::Arc};

use bevy::prelude::*;
use bevy_ui_commonmark::{MdDocument, WikiLink};

/// Index of a note in [`Vault::notes`].
pub type NoteId = usize;

/// One markdown note.
#[derive(Debug)]
pub struct Note {
    /// File name without extension; what `[[links]]` refer to.
    pub name: String,
    /// Path relative to the vault root, e.g. `guides/ECS.md`.
    pub path: String,
    /// Folder relative to the vault root (empty for the root).
    pub folder: String,
    /// Parsed content.
    pub doc: MdDocument,
    /// Lowercase plain text, for search.
    pub search_text: String,
    /// Distinct notes this note links to.
    pub outgoing: Vec<NoteId>,
    /// Distinct notes linking to this note.
    pub backlinks: Vec<NoteId>,
    /// `[[links]]` whose target does not exist.
    pub unresolved: Vec<String>,
}

/// All notes, shared cheaply between systems and UI closures.
#[derive(Resource, Clone, Debug, Deref)]
pub struct Vault(pub Arc<VaultData>);

/// Vault contents.
#[derive(Debug, Default)]
pub struct VaultData {
    /// Notes sorted by path.
    pub notes: Vec<Note>,
    /// Tag → notes, sorted by tag.
    pub tags: BTreeMap<String, Vec<NoteId>>,
    /// The note opened on start.
    pub home: NoteId,
}

impl VaultData {
    /// Builds a vault from `(path, markdown)` pairs.
    pub fn from_sources<'a>(sources: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut notes: Vec<Note> = sources
            .into_iter()
            .map(|(path, source)| {
                let doc = MdDocument::parse(source);
                let file = path.rsplit('/').next().unwrap_or(path);
                let name = file.strip_suffix(".md").unwrap_or(file).to_owned();
                let folder = path
                    .rsplit_once('/')
                    .map(|(folder, _)| folder.to_owned())
                    .unwrap_or_default();
                let search_text = format!("{name}\n{}", doc.plain_text()).to_lowercase();
                Note {
                    name,
                    path: path.to_owned(),
                    folder,
                    doc,
                    search_text,
                    outgoing: Vec::new(),
                    backlinks: Vec::new(),
                    unresolved: Vec::new(),
                }
            })
            .collect();
        notes.sort_by(|a, b| a.path.cmp(&b.path));

        let mut vault = VaultData { notes, ..default() };
        for id in 0..vault.notes.len() {
            let mut outgoing = Vec::new();
            let mut unresolved = Vec::new();
            for link in &vault.notes[id].doc.links {
                match vault.resolve(link) {
                    Some(target) if target != id => {
                        if !outgoing.contains(&target) {
                            outgoing.push(target);
                        }
                    }
                    Some(_) => {}
                    None => {
                        if !unresolved.contains(&link.target) {
                            unresolved.push(link.target.clone());
                        }
                    }
                }
            }
            for &target in &outgoing {
                vault.notes[target].backlinks.push(id);
            }
            vault.notes[id].outgoing = outgoing;
            vault.notes[id].unresolved = unresolved;
            for tag in vault.notes[id].doc.tags.clone() {
                vault.tags.entry(tag).or_default().push(id);
            }
        }
        vault.home = vault
            .find("Welcome")
            .or_else(|| vault.find("README"))
            .unwrap_or(0);
        vault
    }

    /// Finds a note by name or path, case-insensitively.
    pub fn find(&self, name: &str) -> Option<NoteId> {
        let name = name.trim();
        let name = name.strip_suffix(".md").unwrap_or(name);
        self.notes
            .iter()
            .position(|n| n.name.eq_ignore_ascii_case(name))
            .or_else(|| {
                self.notes.iter().position(|n| {
                    n.path
                        .strip_suffix(".md")
                        .is_some_and(|p| p.eq_ignore_ascii_case(name))
                })
            })
    }

    /// Resolves a wikilink to a note.
    pub fn resolve(&self, link: &WikiLink) -> Option<NoteId> {
        self.find(&link.target)
    }

    /// Notes matching a search query and optional tag, best matches first.
    pub fn search(&self, query: &str, tag: Option<&str>) -> Vec<NoteId> {
        let query = query.trim().to_lowercase();
        let terms: Vec<&str> = query.split_whitespace().collect();
        let mut hits: Vec<(usize, NoteId)> = self
            .notes
            .iter()
            .enumerate()
            .filter(|(_, note)| tag.is_none_or(|t| note.doc.tags.iter().any(|nt| nt == t)))
            .filter_map(|(id, note)| {
                if terms.is_empty() {
                    return Some((0, id));
                }
                let name = note.name.to_lowercase();
                let mut score = 0;
                for term in &terms {
                    if name.contains(term) {
                        score += 10;
                    } else if note.search_text.contains(term) {
                        score += 1;
                    } else {
                        return None;
                    }
                }
                Some((score, id))
            })
            .collect();
        hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        hits.into_iter().map(|(_, id)| id).collect()
    }

    /// A short excerpt of the note around the first search term.
    pub fn excerpt(&self, id: NoteId, query: &str) -> Option<String> {
        let term = query.split_whitespace().next()?.to_lowercase();
        let text = self.notes[id].doc.plain_text();
        let lower = text.to_lowercase();
        let at = lower.find(&term)?;
        let start = text[..at]
            .char_indices()
            .rev()
            .nth(30)
            .map_or(0, |(i, _)| i);
        let end = text[at..]
            .char_indices()
            .nth(term.chars().count() + 50)
            .map_or(text.len(), |(i, _)| at + i);
        let snippet = text[start..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        Some(format!(
            "{}{snippet}{}",
            if start > 0 { "…" } else { "" },
            if end < text.len() { "…" } else { "" }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vault() -> VaultData {
        VaultData::from_sources([
            (
                "Welcome.md",
                "# Welcome\nSee [[ECS]] and [[Missing]]. #beginner",
            ),
            (
                "guides/ECS.md",
                "# ECS\nBack to [[welcome]]. #ecs #beginner",
            ),
            ("guides/Rendering.md", "# Rendering\n[[guides/ECS#Systems]]"),
        ])
    }

    #[test]
    fn links_and_backlinks() {
        let v = vault();
        let welcome = v.find("Welcome").unwrap();
        let ecs = v.find("ecs").unwrap();
        let rendering = v.find("Rendering").unwrap();
        assert_eq!(v.notes[welcome].outgoing, vec![ecs]);
        assert_eq!(v.notes[welcome].unresolved, vec!["Missing".to_owned()]);
        assert_eq!(v.notes[ecs].backlinks, vec![welcome, rendering]);
        assert_eq!(v.notes[ecs].folder, "guides");
        assert_eq!(v.home, welcome);
    }

    #[test]
    fn tags_and_search() {
        let v = vault();
        assert_eq!(v.tags["beginner"].len(), 2);
        assert_eq!(v.search("", Some("ecs")), vec![v.find("ECS").unwrap()]);
        assert_eq!(v.search("rendering", None)[0], v.find("Rendering").unwrap());
        assert_eq!(v.search("nothing-matches", None), Vec::<NoteId>::new());
        assert!(
            v.excerpt(v.find("ECS").unwrap(), "back")
                .unwrap()
                .contains("Back to")
        );
    }
}

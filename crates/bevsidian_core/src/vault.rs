//! The note vault: parsed notes, resolved links, backlinks and tags.

use std::{collections::BTreeMap, sync::Arc};

use bevy::prelude::*;
use bevy_radioactive::view::Signal;
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
    /// Icon (a character of the icon font): the `icon:` front-matter property, or a page.
    pub icon: char,
    /// Markdown source.
    pub source: String,
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
///
/// Notes can be edited ([`Vault::save`]), so this is a signal: reactive UI regions that
/// read it with [`Vault::get`] re-render when a note changes.
#[derive(Resource, Clone)]
pub struct Vault(Signal<Arc<VaultData>>);

impl Vault {
    pub fn new(data: VaultData) -> Self {
        Self(Signal::new(Arc::new(data)))
    }

    /// The current contents (tracked: inside a reactive region, re-runs it on change).
    pub fn get(&self) -> Arc<VaultData> {
        self.0.get()
    }

    /// The current contents, untracked (for systems).
    pub fn get_untracked(&self) -> Arc<VaultData> {
        self.0.get_untracked()
    }

    /// Replaces the source of note `id` and rebuilds links, backlinks and tags.
    /// Returns whether anything changed.
    pub fn save(&self, id: NoteId, source: &str) -> bool {
        let old = self.get_untracked();
        if old.notes.get(id).is_none_or(|n| n.source == source) {
            return false;
        }
        let mut data = VaultData::from_sources(old.notes.iter().enumerate().map(|(i, n)| {
            (
                n.path.as_str(),
                if i == id { source } else { n.source.as_str() },
            )
        }));
        data.home = old.home;
        data.revision = old.revision + 1;
        self.0.set(Arc::new(data));
        true
    }
}

/// Characters of context before and after a search term in an [`Excerpt`].
const EXCERPT_BEFORE: usize = 16;
const EXCERPT_AFTER: usize = 60;

/// A search result's context: the matched term with some text around it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Excerpt {
    pub before: String,
    pub term: String,
    pub after: String,
}

/// A reading path through the vault: a note with `path: true` in its front matter, whose
/// links are the path's steps, in order (e.g. a numbered list of `[[links]]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotePath {
    /// The note describing the path.
    pub note: NoteId,
    /// The notes it goes through, in order.
    pub steps: Vec<NoteId>,
}

/// Vault contents.
#[derive(Debug, Default)]
pub struct VaultData {
    /// Notes sorted by path.
    pub notes: Vec<Note>,
    /// Tag → notes, sorted by tag.
    pub tags: BTreeMap<String, Vec<NoteId>>,
    /// Reading paths, in note order.
    pub paths: Vec<NotePath>,
    /// The note opened on start.
    pub home: NoteId,
    /// Bumped by every saved edit.
    pub revision: u64,
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
                let icon = doc
                    .properties
                    .iter()
                    .find(|(key, _)| key == "icon")
                    .and_then(|(_, value)| crate::icons::find(value))
                    .unwrap_or(crate::icons::DEFAULT);
                Note {
                    name,
                    icon,
                    source: source.to_owned(),
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
        let is_path = |note: &Note| {
            note.doc
                .properties
                .iter()
                .any(|(key, value)| key == "path" && value == "true")
        };
        let path_notes: Vec<NoteId> = (0..vault.notes.len())
            .filter(|&id| is_path(&vault.notes[id]))
            .collect();
        vault.paths = path_notes
            .iter()
            .map(|&note| {
                let mut steps = Vec::new();
                for link in &vault.notes[note].doc.links {
                    if let Some(step) = vault.resolve(link)
                        && step != note
                        && !path_notes.contains(&step)
                        && !steps.contains(&step)
                    {
                        steps.push(step);
                    }
                }
                NotePath { note, steps }
            })
            .collect();
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

    /// The reading path note `id` describes, if it is one.
    pub fn path(&self, id: NoteId) -> Option<&NotePath> {
        self.paths.iter().find(|path| path.note == id)
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

    /// A short excerpt of the note around the first search term: a few words before it, the
    /// term as written in the note, and what follows.
    pub fn excerpt(&self, id: NoteId, query: &str) -> Option<Excerpt> {
        let term = query.split_whitespace().next()?.to_lowercase();
        let text = self.notes[id].doc.plain_text();
        // Lowercasing can change byte lengths: only map positions back when it did not.
        let lower = text.to_lowercase();
        if lower.len() != text.len() {
            return None;
        }
        let at = lower.find(&term)?;
        let end_of_term = at + term.len();
        // Up to a few words before the term (it stays visible in a narrow list), from a word's
        // start.
        let mut start = text[..at]
            .char_indices()
            .rev()
            .nth(EXCERPT_BEFORE)
            .map_or(0, |(i, _)| i);
        if start > 0 {
            start = text[start..at]
                .find(char::is_whitespace)
                .map_or(at, |space| start + space + 1);
        }
        let end = text[end_of_term..]
            .char_indices()
            .nth(EXCERPT_AFTER)
            .map_or(text.len(), |(i, _)| end_of_term + i);
        let squash = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        // Keep the spaces around the term (squashing drops them).
        let space_before = text[..at].ends_with(char::is_whitespace);
        let space_after = text[end_of_term..].starts_with(char::is_whitespace);
        let mut before = squash(&text[start..at]);
        if space_before && !before.is_empty() {
            before.push(' ');
        }
        if start > 0 {
            before.insert(0, '…');
        }
        let mut after = squash(&text[end_of_term..end]);
        if space_after && !after.is_empty() {
            after.insert(0, ' ');
        }
        if end < text.len() {
            after.push('…');
        }
        Some(Excerpt {
            before,
            term: text[at..end_of_term].to_owned(),
            after,
        })
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
    fn paths_list_their_links_in_order() {
        let v = VaultData::from_sources([
            ("A.md", "# A"),
            ("B.md", "# B"),
            (
                "Tour.md",
                "---\npath: true\n---\n# Tour\n1. [[B]]\n2. [[A]]\n3. [[B]] again, [[Tour]], [[Gone]]",
            ),
        ]);
        let tour = v.find("Tour").unwrap();
        assert_eq!(
            v.paths,
            vec![NotePath {
                note: tour,
                steps: vec![v.find("B").unwrap(), v.find("A").unwrap()],
            }]
        );
        assert!(v.path(tour).is_some());
        assert!(v.path(v.find("A").unwrap()).is_none());
    }

    #[test]
    fn tags_and_search() {
        let v = vault();
        assert_eq!(v.tags["beginner"].len(), 2);
        assert_eq!(v.search("", Some("ecs")), vec![v.find("ECS").unwrap()]);
        assert_eq!(v.search("rendering", None)[0], v.find("Rendering").unwrap());
        assert_eq!(v.search("nothing-matches", None), Vec::<NoteId>::new());
        let excerpt = v.excerpt(v.find("ECS").unwrap(), "back").unwrap();
        assert_eq!(excerpt.term, "Back", "as written in the note");
        assert!(excerpt.after.starts_with(" to"));
    }
}

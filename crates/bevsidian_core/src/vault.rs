//! The note vault: parsed notes, resolved links, backlinks and tags.

use std::{collections::BTreeMap, sync::Arc};

use bevy::prelude::*;
use bevy_radioactive::view::Signal;
use bevy_ui_commonmark::{MdDocument, WikiLink};

use crate::map::WorldMap;

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
    /// How each step is taken, parallel to `steps`.
    pub kinds: Vec<StepKind>,
    /// Where it sits among the map's routes (the `quest:` property).
    pub quest: Quest,
    /// What a game that takes it has, for the map's checklist (the `feature:` property, or
    /// the note's name).
    pub label: String,
    /// The colour it is drawn in on the map (the `color:` property, a name from the map's
    /// palette), if it names one.
    pub color: Option<String>,
}

/// The kind of route a reading path is on the map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Quest {
    /// The main quest (`quest: main`): always on the map.
    Main,
    /// A feature a game may have (`quest: feature`): offered in the map's checklist.
    Feature,
    /// Something to get better at (`quest: mastery`): offered after the features.
    Mastery,
    /// A path that is not a route.
    #[default]
    Other,
}

/// How a step of a reading path is taken: a link followed by `(optional)` or `(extra)` in the
/// path's note marks the step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    /// On the way (solid line).
    Core,
    /// Worth a detour, but can be skipped (dashed line).
    Optional,
    /// Bonus material for the curious (dotted line).
    Extra,
}

impl NotePath {
    /// The lines joining the steps, as indices into `steps`: core steps in a chain, and each
    /// optional or extra step hanging off the core step before it.
    pub fn edges(&self) -> Vec<(usize, usize, StepKind)> {
        let mut anchor = None;
        let mut edges = Vec::new();
        for (i, &kind) in self.kinds.iter().enumerate() {
            if let Some(from) = anchor {
                edges.push((from, i, kind));
            }
            if kind == StepKind::Core {
                anchor = Some(i);
            }
        }
        edges
    }
}

/// The links of a path note's lines, in order, with how each step is marked.
fn path_links(source: &str) -> Vec<(String, StepKind)> {
    let mut links = Vec::new();
    for line in source.lines() {
        let lower = line.to_lowercase();
        let kind = if lower.contains("(optional)") {
            StepKind::Optional
        } else if lower.contains("(extra)") {
            StepKind::Extra
        } else {
            StepKind::Core
        };
        let mut rest = line;
        while let Some((_, after)) = rest.split_once("[[") {
            let Some((inner, tail)) = after.split_once("]]") else {
                break;
            };
            let target = inner.split(['|', '#']).next().unwrap_or(inner);
            links.push((target.to_owned(), kind));
            rest = tail;
        }
    }
    links
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
    /// The hand-drawn map of the graph (the note with `map: true`).
    pub map: Option<WorldMap>,
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
                let (mut steps, mut kinds) = (Vec::new(), Vec::new());
                for (target, kind) in path_links(&vault.notes[note].source) {
                    if let Some(step) = vault.find(&target)
                        && step != note
                        && !path_notes.contains(&step)
                        && !steps.contains(&step)
                    {
                        steps.push(step);
                        kinds.push(kind);
                    }
                }
                let property = |key: &str| {
                    vault.notes[note]
                        .doc
                        .properties
                        .iter()
                        .find(|(k, _)| k == key)
                        .map(|(_, v)| v.trim().to_owned())
                };
                let quest = match property("quest").as_deref() {
                    Some("main") => Quest::Main,
                    Some("feature") => Quest::Feature,
                    Some("mastery") => Quest::Mastery,
                    _ => Quest::Other,
                };
                NotePath {
                    note,
                    steps,
                    kinds,
                    quest,
                    label: property("feature").unwrap_or_else(|| vault.notes[note].name.clone()),
                    color: property("color"),
                }
            })
            .collect();
        vault.map = vault
            .notes
            .iter()
            .position(|note| {
                note.doc
                    .properties
                    .iter()
                    .any(|(key, value)| key == "map" && value == "true")
            })
            .map(|id| WorldMap::parse(id, &vault.notes[id].source));
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

    /// Whether note `id` describes the vault rather than teaching (a reading path or the
    /// map): the graph leaves it out.
    pub fn is_meta(&self, id: NoteId) -> bool {
        self.path(id).is_some() || self.map.as_ref().is_some_and(|m| m.note == id)
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
                kinds: vec![StepKind::Core; 2],
                quest: Quest::Other,
                label: "Tour".into(),
                color: None,
            }]
        );
        assert!(v.path(tour).is_some());
        assert!(v.is_meta(tour));
        assert!(v.path(v.find("A").unwrap()).is_none());
    }

    #[test]
    fn optional_and_extra_steps_hang_off_the_core_route() {
        let v = VaultData::from_sources([
            ("A.md", "# A"),
            ("B.md", "# B"),
            ("C.md", "# C"),
            ("D.md", "# D"),
            (
                "Tour.md",
                "---\npath: true\n---\n# Tour\n1. [[A]]\n2. [[B#Heading|b]] (optional)\n3. [[C]] (extra)\n4. [[D]]",
            ),
        ]);
        let path = v.path(v.find("Tour").unwrap()).unwrap();
        assert_eq!(
            path.kinds,
            vec![
                StepKind::Core,
                StepKind::Optional,
                StepKind::Extra,
                StepKind::Core
            ]
        );
        assert_eq!(
            path.edges(),
            vec![
                (0, 1, StepKind::Optional),
                (0, 2, StepKind::Extra),
                (0, 3, StepKind::Core)
            ]
        );
    }

    #[test]
    fn quests_and_the_map() {
        let v = VaultData::from_sources([
            ("A.md", "# A"),
            (
                "Run.md",
                "---\npath: true\nquest: feature\nfeature: Running\n---\n# Run\n\nGo fast,\nvery fast.\n\n1. [[A]]",
            ),
            (
                "Map.md",
                "---\nmap: true\n---\n# Map\n```map\nA @ 1, 2\n```",
            ),
        ]);
        let run = v.path(v.find("Run").unwrap()).unwrap();
        assert_eq!(run.quest, Quest::Feature);
        assert_eq!(run.label, "Running");
        let map = v.map.as_ref().unwrap();
        assert_eq!(map.note, v.find("Map").unwrap());
        assert_eq!(map.place("A").unwrap().at, bevy::math::Vec2::new(1.0, 2.0));
        assert!(v.is_meta(map.note));
        assert!(!v.is_meta(v.find("A").unwrap()));
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

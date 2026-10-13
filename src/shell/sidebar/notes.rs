//! The note tree: folders, their notes, and under the open note its headings. While searching
//! or filtering by tag it becomes a flat list of results with excerpts.

use std::collections::{BTreeMap, BTreeSet};

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Signal, effect, memo, signal, view};

use bevsidian_core::{
    Vault,
    nav::Nav,
    vault::{NoteId, VaultData},
};
use bevsidian_ui::prelude::*;

use super::FollowSelection;

/// Row heights: mouse, touch.
const ROW: [f32; 2] = [32.0, 40.0];
const SUB_ROW: [f32; 2] = [28.0, 36.0];

pub fn notes(nav: Nav, vault: Vault) -> impl Scene {
    // Notes whose headings are shown. Opening a note shows its headings and folds the rest.
    let expanded: Signal<BTreeSet<NoteId>> = signal(BTreeSet::new);
    // Folders folded by the user (folders start open).
    let folded: Signal<BTreeSet<String>> = signal(BTreeSet::new);
    let (current, follow) = (nav.current.clone(), expanded.clone());
    let key = {
        let (query, tag, touch, vault) = (
            nav.query.clone(),
            nav.tag.clone(),
            nav.touch.clone(),
            vault.clone(),
        );
        memo(move || (query.get(), tag.get(), touch.get(), vault.get().revision))
    };
    view! {
        Node { flex_direction: FlexDirection::Column, flex_shrink: 0.0 }
        effect(move |_: &mut Commands| follow.set(BTreeSet::from([current.get()])))
        Children [
            match $key {
                (query, tag, touch, _) => {
                    @{list(&vault.get_untracked(), &query, tag.as_deref(), touch, &nav, &expanded, &folded)}
                },
            }
        ]
    }
}

/// The tree, or the search results.
fn list(
    data: &VaultData,
    query: &str,
    tag: Option<&str>,
    touch: bool,
    nav: &Nav,
    expanded: &Signal<BTreeSet<NoteId>>,
    closed: &Signal<BTreeSet<String>>,
) -> Box<dyn Scene> {
    let searching = !query.trim().is_empty() || tag.is_some();
    let rows: Vec<Box<dyn Scene>> = if searching {
        let ids = data.search(query, tag);
        if ids.is_empty() {
            vec![Box::new(empty("No matching notes"))]
        } else {
            ids.into_iter()
                .map(|id| Box::new(result_row(id, data, query, touch, nav)) as Box<dyn Scene>)
                .collect()
        }
    } else {
        let tree = Folder::of(data);
        folder_contents(&tree, data, touch, nav, expanded, closed)
    };
    let title = if searching { "Results" } else { "Notes" };
    Box::new(bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(1) }
        Children [ @{group_label(title)} -- {rows} ]
    })
}

/// A folder of the vault, built from note paths.
#[derive(Default)]
struct Folder {
    /// Path from the vault root, e.g. `guides/ecs`.
    path: String,
    name: String,
    folders: BTreeMap<String, Folder>,
    notes: Vec<NoteId>,
}

impl Folder {
    fn of(data: &VaultData) -> Self {
        let mut root = Folder::default();
        for (id, note) in data.notes.iter().enumerate() {
            let mut folder = &mut root;
            for part in note.folder.split('/').filter(|p| !p.is_empty()) {
                let path = if folder.path.is_empty() {
                    part.to_owned()
                } else {
                    format!("{}/{part}", folder.path)
                };
                folder = folder
                    .folders
                    .entry(part.to_owned())
                    .or_insert_with(|| Folder {
                        path,
                        name: part.to_owned(),
                        ..default()
                    });
            }
            folder.notes.push(id);
        }
        root
    }
}

/// Subfolders first, then notes, like a file browser.
fn folder_contents(
    folder: &Folder,
    data: &VaultData,
    touch: bool,
    nav: &Nav,
    expanded: &Signal<BTreeSet<NoteId>>,
    closed: &Signal<BTreeSet<String>>,
) -> Vec<Box<dyn Scene>> {
    let folders = folder
        .folders
        .values()
        .map(|sub| Box::new(folder_row(sub, data, touch, nav, expanded, closed)) as Box<dyn Scene>);
    let notes = folder
        .notes
        .iter()
        .map(|&id| Box::new(note_row(id, data, touch, nav, expanded)) as Box<dyn Scene>);
    folders.chain(notes).collect()
}

fn folder_row(
    folder: &Folder,
    data: &VaultData,
    touch: bool,
    nav: &Nav,
    expanded: &Signal<BTreeSet<NoteId>>,
    closed: &Signal<BTreeSet<String>>,
) -> impl Scene + use<> {
    let children = folder_contents(folder, data, touch, nav, expanded, closed);
    let path = folder.path.clone();
    let (is_open, toggle) = set_toggle(closed.clone(), path.clone(), true);
    let (row_toggle, shown) = (toggle.clone(), is_open.clone());
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            @{item()}
            Node { height: px(ROW[usize::from(touch)]) }
            on(move |_: On<Activate>| row_toggle())
            Children [
                @{item_icon("folder")}
                --
                @{item_label(&folder.name)}
                --
                @{disclosure(is_open.clone(), toggle.clone())}
            ]
            --
            @{tree_children()}
            Node { display: $(if shown() { Display::Flex } else { Display::None }) }
            Children [ {children} ]
        ]
    }
}

fn note_row(
    id: NoteId,
    data: &VaultData,
    touch: bool,
    nav: &Nav,
    expanded: &Signal<BTreeSet<NoteId>>,
) -> impl Scene + use<> {
    let note = &data.notes[id];
    // A lone level-1 heading is the title: it is not part of the outline.
    let titles = note.doc.headings.iter().filter(|h| h.level == 1).count();
    let outline: Vec<_> = note
        .doc
        .headings
        .iter()
        .filter(|h| h.level > 1 || titles > 1)
        .collect();
    let top = outline.iter().map(|h| h.level).min().unwrap_or(1);
    let headings: Vec<Box<dyn Scene>> = outline
        .iter()
        .map(|heading| {
            let (nav, text, slug) = (nav.clone(), heading.text.clone(), heading.slug.clone());
            let (section, current) = (nav.section.clone(), nav.current.clone());
            let indent = 8.0 + 12.0 * f32::from(heading.level - top);
            Box::new(view! {
                @{sub_item(None, &heading.text)}
                Node { height: px(SUB_ROW[usize::from(touch)]), padding: UiRect::new(px(indent), px(8), px(0), px(0)) }
                // The section being read.
                Selected($($current == id && $section.as_deref() == Some(slug.as_str())))
                FollowSelection
                on(move |_: On<Activate>| nav.open(id, Some(text.clone())))
            }) as Box<dyn Scene>
        })
        .collect();
    let has_outline = !headings.is_empty();
    let (is_open, toggle) = set_toggle(expanded.clone(), id, false);
    let chevron: Vec<Box<dyn Scene>> = has_outline
        .then(|| Box::new(disclosure(is_open.clone(), toggle)) as Box<dyn Scene>)
        .into_iter()
        .collect();
    let (current, open) = (nav.current.clone(), nav.clone());
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            @{item()}
            Node { height: px(ROW[usize::from(touch)]) }
            Selected($($current == id))
            FollowSelection
            on(move |_: On<Activate>| open.open(id, None))
            Children [
                @{item_icon_char(note.icon)}
                --
                @{item_label(&note.name)}
                --
                {chevron}
            ]
            --
            @{tree_children()}
            Node { display: $(if has_outline && is_open() { Display::Flex } else { Display::None }) }
            Children [ {headings} ]
        ]
    }
}

/// A search result: the note, and under it where the query matched.
fn result_row(
    id: NoteId,
    data: &VaultData,
    query: &str,
    touch: bool,
    nav: &Nav,
) -> impl Scene + use<> {
    let note = &data.notes[id];
    let excerpt: Vec<Box<dyn Scene>> = data
        .excerpt(id, query)
        .map(|text| {
            Box::new(bsn! {
                @{tinted(&text, size::XS, FontWeight::NORMAL, Tint::fixed(color::MUTED_FOREGROUND))}
                Node { margin: UiRect::left(px(24)) }
            }) as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    let (current, open) = (nav.current.clone(), nav.clone());
    let row = px(ROW[usize::from(touch)]);
    view! {
        @{item()}
        Node {
            height: Val::Auto,
            min_height: row,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(8), px(6)),
            row_gap: px(2),
            overflow: Overflow::clip(),
        }
        Selected($($current == id))
        on(move |_: On<Activate>| open.open(id, None))
        Children [
            Node { column_gap: px(8), align_items: AlignItems::Center, min_width: px(0) }
            Pickable::IGNORE
            Children [ @{item_icon_char(note.icon)} -- @{item_label(&note.name)} ]
            --
            {excerpt}
        ]
    }
}

/// Reads and flips the membership of `key` in a set signal. With `inverted`, membership means
/// closed (for things that start open).
fn set_toggle<K>(
    set: Signal<BTreeSet<K>>,
    key: K,
    inverted: bool,
) -> (
    impl Fn() -> bool + Clone + Send + Sync + 'static,
    impl Fn() + Clone + Send + Sync + 'static,
)
where
    K: Ord + Clone + Send + Sync + 'static,
{
    let (read, write, k) = (set.clone(), set, key.clone());
    (
        move || read.with(|s| s.contains(&key)) != inverted,
        move || {
            write.update(|s| {
                if !s.remove(&k) {
                    s.insert(k.clone());
                }
            });
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_nest_and_keep_notes_in_path_order() {
        let data = VaultData::from_sources([
            ("Welcome.md", "# Welcome"),
            ("guides/ECS.md", "# ECS"),
            ("guides/advanced/Stepping.md", "# Stepping"),
            ("guides/Assets.md", "# Assets"),
        ]);
        let root = Folder::of(&data);
        let name = |id: NoteId| data.notes[id].name.as_str();
        assert_eq!(
            root.notes.iter().map(|&id| name(id)).collect::<Vec<_>>(),
            ["Welcome"]
        );
        let guides = &root.folders["guides"];
        assert_eq!(guides.path, "guides");
        assert_eq!(
            guides.notes.iter().map(|&id| name(id)).collect::<Vec<_>>(),
            ["Assets", "ECS"]
        );
        let advanced = &guides.folders["advanced"];
        assert_eq!(advanced.path, "guides/advanced");
        assert_eq!(advanced.notes.len(), 1);
    }

    #[test]
    fn an_empty_vault_has_an_empty_tree() {
        let root = Folder::of(&VaultData::from_sources([]));
        assert!(root.notes.is_empty() && root.folders.is_empty());
    }
}

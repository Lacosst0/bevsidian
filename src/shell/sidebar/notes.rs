//! The note tree: folders, their notes, and under the open note its headings. While searching
//! or filtering by tag it becomes a flat list of results with excerpts.

use std::collections::{BTreeMap, BTreeSet};

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, effect, view};

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

/// Notes whose headings are shown, and open folders (folders start closed).
#[derive(Clone, Default)]
struct Tree {
    expanded: Watch<Local<'static, BTreeSet<NoteId>>>,
    opened: Watch<Local<'static, BTreeSet<String>>>,
}

/// Opening a note shows its headings, folds the other notes and opens its folders; `followed` is
/// the note that last did.
#[component]
pub fn notes(
    nav: Watch<Res<Nav>>,
    vault: Watch<Res<Vault>>,
    expanded: Watch<Local<BTreeSet<NoteId>>>,
    opened: Watch<Local<BTreeSet<String>>>,
    followed: Watch<Local<Option<NoteId>>>,
) -> impl Scene {
    let tree = Tree {
        expanded: expanded.clone(),
        opened: opened.clone(),
    };
    view! {
        Node { flex_direction: FlexDirection::Column, flex_shrink: 0.0 }
        effect(move |_: &mut Commands| {
            let current = nav.with(|nav| nav.current);
            if followed.set_if_neq(Some(current)) {
                expanded.set(BTreeSet::from([current]));
                let folder = vault.with_untracked(|vault| {
                    vault.notes.get(current).map(|note| note.folder.clone())
                });
                if let Some(folder) = folder {
                    let mut open = opened.get_untracked();
                    let mut path = String::new();
                    for part in folder.split('/').filter(|p| !p.is_empty()) {
                        if !path.is_empty() {
                            path.push('/');
                        }
                        path.push_str(part);
                        open.insert(path.clone());
                    }
                    opened.set_if_neq(open);
                }
            }
        })
        Children [
            match ($nav.query, $nav.tag, $nav.touch, $vault.revision) {
                (query, tag, touch, _) => {
                    @{vault.with_untracked(|vault| list(vault, &query, tag.as_deref(), touch, &tree))}
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
    tree: &Tree,
) -> Box<dyn Scene> {
    let searching = !query.trim().is_empty() || tag.is_some();
    let rows: Vec<Box<dyn Scene>> = if searching {
        let ids = data.search(query, tag);
        if ids.is_empty() {
            vec![Box::new(empty("No matching notes"))]
        } else {
            ids.into_iter()
                .map(|id| Box::new(result_row(id, data, query, touch)) as Box<dyn Scene>)
                .collect()
        }
    } else {
        let root = Folder::of(data);
        // At the top, notes (Welcome) come before the folders.
        let notes = root
            .notes
            .iter()
            .map(|&id| Box::new(note_row(id, data, touch, tree)) as Box<dyn Scene>);
        let folders = root
            .folders
            .values()
            .map(|sub| Box::new(folder_row(sub, data, touch, tree)) as Box<dyn Scene>);
        notes.chain(folders).collect()
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
    tree: &Tree,
) -> Vec<Box<dyn Scene>> {
    let folders = folder
        .folders
        .values()
        .map(|sub| Box::new(folder_row(sub, data, touch, tree)) as Box<dyn Scene>);
    let notes = folder
        .notes
        .iter()
        .map(|&id| Box::new(note_row(id, data, touch, tree)) as Box<dyn Scene>);
    folders.chain(notes).collect()
}

fn folder_row(folder: &Folder, data: &VaultData, touch: bool, tree: &Tree) -> impl Scene + use<> {
    let children = folder_contents(folder, data, touch, tree);
    let shown = member(tree.opened.clone(), folder.path.clone(), false);
    let toggle = shown.clone();
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            @{item()}
            Node { height: px(ROW[usize::from(touch)]) }
            on(move |_: On<Activate>, mut commands: Commands| toggle.update(&mut commands, |was| !was))
            Children [
                @{item_icon("folder")}
                --
                @{item_label(&folder.name)}
                --
                @{disclosure(shown.clone())}
            ]
            --
            @{tree_children()}
            Node { display: $(if $shown { Display::Flex } else { Display::None }) }
            Children [ {children} ]
        ]
    }
}

/// A note, and under it (while expanded) its headings.
#[component]
fn note_row(
    nav: Watch<Res<Nav>>,
    id: NoteId,
    data: &VaultData,
    touch: bool,
    tree: &Tree,
) -> impl Scene {
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
            let (text, slug) = (heading.text.clone(), heading.slug.clone());
            let nav = nav.clone();
            let indent = 8.0 + 12.0 * f32::from(heading.level - top);
            Box::new(view! {
                @{sub_item(None, &heading.text)}
                Node { height: px(SUB_ROW[usize::from(touch)]), padding: UiRect::new(px(indent), px(8), px(0), px(0)) }
                // The section being read.
                Selected($(nav.with(|nav| nav.current == id && nav.section.as_deref() == Some(slug.as_str()))))
                FollowSelection
                on(move |_: On<Activate>, mut n: ResMut<Nav>| n.open(id, Some(text.clone())))
            }) as Box<dyn Scene>
        })
        .collect();
    let has_outline = !headings.is_empty();
    let outlined = member(tree.expanded.clone(), id, false);
    let chevron: Vec<Box<dyn Scene>> = has_outline
        .then(|| Box::new(disclosure(outlined.clone())) as Box<dyn Scene>)
        .into_iter()
        .collect();
    let (icon, name) = (note.icon, note.name.clone());
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            @{item()}
            Node { height: px(ROW[usize::from(touch)]) }
            Selected($nav.current == id)
            FollowSelection
            on(move |_: On<Activate>, mut n: ResMut<Nav>| n.open(id, None))
            Children [
                @{item_icon_char(icon)}
                --
                @{item_label(&name)}
                --
                {chevron}
            ]
            --
            @{tree_children()}
            Node { display: $(if has_outline && $outlined { Display::Flex } else { Display::None }) }
            Children [ {headings} ]
        ]
    }
}

/// A search result: the note, and under it where the query matched.
#[component]
fn result_row(
    nav: Watch<Res<Nav>>,
    id: NoteId,
    data: &VaultData,
    query: &str,
    touch: bool,
) -> impl Scene {
    let note = &data.notes[id];
    // The excerpt on one line, cut at the end, with the matched term brighter.
    let excerpt: Vec<Box<dyn Scene>> = data
        .excerpt(id, query)
        .map(|excerpt| {
            let normal = insert(font(size::XS, FontWeight::NORMAL));
            let strong = insert(font(size::XS, FontWeight::MEDIUM));
            let rest = insert(font(size::XS, FontWeight::NORMAL));
            let (before, term, after) = (
                Text::new(excerpt.before),
                TextSpan::new(excerpt.term),
                TextSpan::new(excerpt.after),
            );
            Box::new(bsn! {
                before
                @{normal}
                TextColor(color::MUTED_FOREGROUND)
                TextLayout { linebreak: LineBreak::NoWrap }
                Node { margin: UiRect::left(px(24)), min_width: px(0), overflow: Overflow::clip() }
                Pickable::IGNORE
                Children [
                    term @{strong} TextColor(color::FOREGROUND)
                    --
                    after @{rest} TextColor(color::MUTED_FOREGROUND)
                ]
            }) as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    let (icon, name) = (note.icon, note.name.clone());
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
        Selected($nav.current == id)
        on(move |_: On<Activate>, mut n: ResMut<Nav>| n.open(id, None))
        Children [
            Node { column_gap: px(8), align_items: AlignItems::Center, min_width: px(0) }
            Pickable::IGNORE
            Children [ @{item_icon_char(icon)} -- @{item_label(&name)} ]
            --
            {excerpt}
        ]
    }
}

/// Whether `key` is in a set of the tree's state, as a binding; with `inverted`, being in
/// the set means closed (for things that start open).
fn member<K>(set: Watch<Local<'static, BTreeSet<K>>>, key: K, inverted: bool) -> Binding<bool>
where
    K: Ord + Clone + Send + Sync + 'static,
{
    let k = key.clone();
    Binding::local_part(
        set,
        move |set: &BTreeSet<K>| set.contains(&key) != inverted,
        move |set, open| {
            if open == inverted {
                set.remove(&k);
            } else {
                set.insert(k.clone());
            }
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

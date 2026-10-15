//! The note page: the open note rendered from markdown (or the editor), in a centred column.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, view};
use bevy_ui_commonmark::{MdTheme, RenderOptions, markdown_scene};
use bevy_ui_textselect::SelectableText;

use bevsidian_core::{
    Vault, VaultData,
    nav::{LayoutMode, Nav, NotePane},
    vault::NoteId,
};
use bevsidian_ui::prelude::*;

#[component]
pub fn note_page(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>, theme: MdTheme) -> impl Scene {
    view! {
        Node { flex_grow: 1.0, flex_basis: px(0), min_width: px(0), min_height: px(0) }
        Children [
        Node {
            flex_grow: 1.0,
            min_width: px(0),
            overflow: Overflow { x: OverflowAxis::Clip, y: OverflowAxis::Scroll },
            justify_content: JustifyContent::Center,
        }
        NotePane
        Children [
            Node {
                width: Val::Percent(100.0),
                max_width: px(784),
                min_width: px(0),
                padding: $(if $nav.mode == LayoutMode::Compact {
                    UiRect::new(px(20), px(20), px(24), px(140))
                } else {
                    UiRect::new(px(48), px(48), px(40), px(120))
                }),
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Start,
            }
            Children [
                if $nav.editing {
                    Node { position_type: PositionType::Relative, width: Val::Percent(100.0) }
                    crate::editor::EditorSlot($nav.current)
                } else {
                    let theme = theme.clone();
                    Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0) }
                    Children [
                        // Re-rendered when the note, the text size, the layout or the note's
                        // content changes.
                        match ($nav.current, $nav.text_size, $nav.mode, $vault.revision) {
                            (id, size, mode, _) => {
                                let theme = theme.clone();
                                @{vault.with_untracked(|vault| note_body(id, size, mode, vault, &theme))}
                            },
                        }
                        --
                        @{path_pager()}
                    ]
                }
            ]
        ]
        --
        @{back_to_top()}
        --
        @{crate::overlays::find::find_bar()}
        ]
    }
}

/// Where the current note is on the followed reading path.
#[derive(Clone, Debug, PartialEq)]
struct PathPlace {
    path: NoteId,
    name: String,
    /// The note's step (from 1); `None` on the path's own note.
    step: Option<usize>,
    steps: usize,
    previous: Option<(NoteId, String)>,
    next: Option<(NoteId, String)>,
}

/// Where the current note is on the followed path, if it is on it.
fn place(nav: &Nav, vault: &VaultData) -> Option<PathPlace> {
    let path = vault.path(nav.path?)?;
    let current = nav.current;
    let step = path.steps.iter().position(|&s| s == current);
    if step.is_none() && current != path.note {
        return None;
    }
    let named = |id: NoteId| (id, vault.notes[id].name.clone());
    Some(PathPlace {
        path: path.note,
        name: vault.notes[path.note].name.clone(),
        step: step.map(|s| s + 1),
        steps: path.steps.len(),
        previous: step
            .map(|s| s.checked_sub(1).map_or(path.note, |p| path.steps[p]))
            .map(named),
        next: path
            .steps
            .get(step.map_or(0, |s| s + 1))
            .copied()
            .map(named),
    })
}

/// At the end of a note on the followed path: the path, the step, and links to the previous
/// and next notes (from the path's own note, a link to its first step).
#[component]
fn path_pager(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>) -> impl Scene {
    view! {
        Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0) }
        Children [
            match $(place(&$nav, &$vault)) {
                Some(place) => { @{pager(&place)} },
                None => {},
            }
        ]
    }
}

fn pager(place: &PathPlace) -> impl Scene + use<> {
    let path = place.path;
    let where_ = match place.step {
        Some(number) => format!("Step {number} of {}", place.steps),
        None => format!("{} steps", place.steps),
    };
    let link = |direction: PageDirection, label: &str, target: &Option<(NoteId, String)>| {
        let Some((id, title)) = target.clone() else {
            // Keeps the other link on its side.
            return Box::new(bsn! { Node { flex_grow: 1.0, flex_basis: px(0) } }) as Box<dyn Scene>;
        };
        Box::new(bsn! {
            @{page_link(direction, label, &title)}
            on(move |_: On<Activate>, mut n: ResMut<Nav>| n.open(id, None))
        }) as Box<dyn Scene>
    };
    let previous = link(PageDirection::Previous, "Previous", &place.previous);
    let next_label = if place.step.is_some() {
        "Next"
    } else {
        "Start"
    };
    let next = link(PageDirection::Next, next_label, &place.next);
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(12), margin: UiRect::top(px(48)) }
        Children [
            @{separator()}
            --
            Node { align_items: AlignItems::Center, column_gap: px(8), min_height: px(28) }
            Children [
                @{text_button(Variant::Ghost, Size::Xs, &place.name)}
                on(move |_: On<Activate>, mut n: ResMut<Nav>| n.open(path, None))
                --
                @{caption(&where_)}
                --
                Node { flex_grow: 1.0 }
                --
                @{icon_button(Variant::Ghost, Size::IconXs, "x")}
                on(|_: On<Activate>, mut n: ResMut<Nav>| n.path = None)
            ]
            --
            Node { column_gap: px(12) }
            Children [ @{previous} -- @{next} ]
        ]
    }
}

/// A round button floating at the bottom right once the note is scrolled down; it eases back
/// to the top.
#[component]
fn back_to_top(nav: Watch<Res<Nav>>) -> impl Scene {
    let shadow = shadow::popover();
    view! {
        Node {
            position_type: PositionType::Absolute,
            right: $(if $nav.mode == LayoutMode::Compact { px(16) } else { px(24) }),
            bottom: $(if $nav.mode == LayoutMode::Compact { px(16) } else { px(24) }),
        }
        Pickable::IGNORE
        Children [
            if $nav.scrolled {
                let shadow = shadow.clone();
                @{icon_button(Variant::Outline, Size::IconLg, "arrow-up")}
                Tooltip::new("Back to top")
                Surface { idle: color::CARD, hover: color::ACCENT, selected: color::ACCENT }
                Node { border_radius: BorderRadius::all(px(radius::FULL)) }
                shadow
                on(|_: On<Activate>, mut panes: Query<&mut ScrollPosition, With<NotePane>>| {
                    // The pane's smooth scrolling eases to the new position.
                    for mut position in &mut panes {
                        position.0 = Vec2::ZERO;
                    }
                })
            }
        ]
    }
}

/// The note's markdown, with its icon before the title (in its route's colour). Notes without a level-1 heading get
/// their name as the title, like Obsidian.
fn note_body(
    id: NoteId,
    text_size: u32,
    mode: LayoutMode,
    vault: &VaultData,
    theme: &MdTheme,
) -> Box<dyn Scene> {
    let mut theme = theme.clone();
    theme.base_size = text_size as f32;
    if mode == LayoutMode::Compact {
        theme = theme.compact();
    }
    let note = &vault.notes[id];
    let icon = note.icon;
    // In the colour of the note's route on the map.
    let tint = bevsidian_graph::note_colors(vault)
        .get(&id)
        .copied()
        .unwrap_or(color::FOREGROUND);
    let resolve = |link: &bevy_ui_commonmark::WikiLink| vault.resolve(link).is_some();
    let badge = move |title_size: f32, line: f32| -> Box<dyn Scene> {
        Box::new(note_icon(icon, tint, title_size, line))
    };
    let options = RenderOptions {
        theme: &theme,
        resolve: &resolve,
        title_badge: Some(&badge),
        code_preview: Some(&bevsidian_preview::code_preview),
        icon: Some(&crate::theme::icon),
        variant_picker: Some(&crate::theme::variant_picker),
    };
    let body = markdown_scene(&note.doc, &options);
    let title: Vec<Box<dyn Scene>> = note
        .doc
        .title
        .is_none()
        .then(|| {
            let size = theme.heading_size(1);
            let row = Node {
                align_items: AlignItems::FlexStart,
                column_gap: px(size * 0.5),
                margin: UiRect::bottom(px(16)),
                ..default()
            };
            let name = label(&note.name, size, FontWeight::SEMIBOLD, color::FOREGROUND);
            Box::new(bsn! {
                row
                Children [
                    // `label` keeps Bevy's default line height, 1.2 × the font size.
                    @{note_icon(icon, tint, size, size * 1.2)}
                    --
                    @{name} SelectableText Pickable::default() Node { flex_shrink: 1.0 }
                ]
            }) as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    Box::new(bsn! {
        Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), min_width: px(0) }
        Children [ {title} -- @{body} ]
    })
}

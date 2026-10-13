//! The note page: the open note rendered from markdown (or the editor), in a centred column.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{memo, view};
use bevy_ui_commonmark::{MdTheme, RenderOptions, markdown_scene};
use bevy_ui_textselect::SelectableText;

use bevsidian_core::{
    Vault,
    nav::{LayoutMode, Nav, NotePane},
    vault::NoteId,
};
use bevsidian_ui::prelude::*;

pub fn note_page(nav: Nav, vault: Vault, theme: MdTheme) -> impl Scene {
    // Re-rendered when the note, the text size, the layout or the note's content changes.
    let key = {
        let (current, size, mode, vault) = (
            nav.current.clone(),
            nav.text_size.clone(),
            nav.mode.clone(),
            vault.clone(),
        );
        memo(move || (current.get(), size.get(), mode.get(), vault.get().revision))
    };
    let (mode, editing, current) = (nav.mode.clone(), nav.editing.clone(), nav.current.clone());
    let render = move |(id, size, mode, _): (NoteId, u32, LayoutMode, u64)| {
        note_body(id, size, mode, &vault, &theme)
    };
    let top = back_to_top(nav.clone());
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
                padding: $(if $mode == LayoutMode::Compact {
                    UiRect::new(px(20), px(20), px(24), px(140))
                } else {
                    UiRect::new(px(48), px(48), px(40), px(120))
                }),
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Start,
            }
            Children [
                if $editing {
                    Node { position_type: PositionType::Relative, width: Val::Percent(100.0) }
                    crate::editor::EditorSlot($current)
                } else {
                    Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0) }
                    Children [
                        let render = render.clone();
                        match $key {
                            key => { @{render(key)} },
                        }
                    ]
                }
            ]
        ]
        --
        @{top}
        ]
    }
}

/// A round button floating at the bottom right once the note is scrolled down; it eases back
/// to the top.
fn back_to_top(nav: Nav) -> impl Scene {
    let (scrolled, mode) = (nav.scrolled.clone(), nav.mode.clone());
    let shadow = shadow::popover();
    view! {
        Node {
            position_type: PositionType::Absolute,
            right: $(if $mode == LayoutMode::Compact { px(16) } else { px(24) }),
            bottom: $(if $mode == LayoutMode::Compact { px(16) } else { px(24) }),
        }
        Pickable::IGNORE
        Children [
            if $scrolled {
                let shadow = shadow.clone();
                @{icon_button(Variant::Outline, Size::IconLg, "arrow-up")}
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

/// The note's markdown, with its icon before the title. Notes without a level-1 heading get
/// their name as the title, like Obsidian.
fn note_body(
    id: NoteId,
    text_size: u32,
    mode: LayoutMode,
    vault: &Vault,
    theme: &MdTheme,
) -> Box<dyn Scene> {
    let vault = vault.get_untracked();
    let mut theme = theme.clone();
    theme.base_size = text_size as f32;
    if mode == LayoutMode::Compact {
        theme = theme.compact();
    }
    let note = &vault.notes[id];
    let icon = note.icon;
    let resolve = |link: &bevy_ui_commonmark::WikiLink| vault.resolve(link).is_some();
    let badge = move |title_size: f32| -> Box<dyn Scene> { Box::new(note_icon(icon, title_size)) };
    let options = RenderOptions {
        theme: &theme,
        resolve: &resolve,
        title_badge: Some(&badge),
        code_preview: Some(&bevsidian_preview::code_preview),
        icon: Some(&crate::theme::icon),
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
                    @{note_icon(icon, size)}
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

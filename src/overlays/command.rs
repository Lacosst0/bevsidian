//! The quick switcher (`Ctrl+O`), a command palette over the notes: type to filter, arrows to
//! move, Enter to open.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::{Vault, VaultData, nav::Nav, vault::NoteId};
use bevsidian_ui::prelude::*;

#[component]
pub fn command_palette(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::COMMAND)
        Pickable::IGNORE
        Children [
            if $nav.switcher {
                @{scrim(|world| world.resource_mut::<Nav>().switcher = false)}
                Node { justify_content: JustifyContent::Center, align_items: AlignItems::Start, padding: UiRect::top(Val::Percent(8.0)) }
                Children [ @{palette()} ]
            }
        ]
    }
}

/// A row of the list: a group label, or a note with its index among the notes (and whether
/// it is the last).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Row {
    Label(&'static str),
    Note {
        index: usize,
        id: NoteId,
        last: bool,
    },
}

/// The switcher's rows. With no query, the notes visited last come first, under their own
/// label.
fn rows(nav: &Nav, vault: &VaultData) -> Vec<Row> {
    let (notes, recent) = nav.switcher_results(vault);
    let mut rows = Vec::new();
    for (index, &id) in notes.iter().enumerate() {
        if index == 0 {
            rows.push(Row::Label(if recent > 0 { "Recent" } else { "Notes" }));
        } else if index == recent {
            rows.push(Row::Label("Notes"));
        }
        let last = index + 1 == notes.len();
        rows.push(Row::Note { index, id, last });
    }
    rows
}

#[component]
fn palette(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>) -> impl Scene {
    let query = Some(Binding::resource(
        nav.clone(),
        |n: &Nav| n.switcher_query.clone(),
        |n, query| n.switcher_query = query,
    ));
    let shadow = shadow::dialog();
    view! {
        Node {
            flex_direction: FlexDirection::Column,
            width: px(560),
            max_width: Val::Percent(94.0),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::XXL)),
            overflow: Overflow::clip(),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        shadow
        Children [
            Node { padding: UiRect::all(px(8)), border: UiRect::bottom(px(1)) }
            BorderColor::all(color::BORDER)
            Children [
                @InputField { @icon: "search", @placeholder: "Search notes…", @value: query, @autofocus: true }
                Node { flex_grow: 1.0, border: UiRect::all(px(0)) }
                BackgroundColor(Color::NONE)
            ]
            --
            Node { flex_direction: FlexDirection::Column, padding: UiRect::all(px(4)), max_height: px(380), overflow: Overflow::scroll_y() }
            bevy::ui_widgets::ScrollArea
            Children [
                #[key(*entry)]
                for entry in $(rows(&$nav, &$vault)) {
                    @{vault.with_untracked(|vault| row(entry, vault))}
                }
                --
                if $($nav.switcher_results(&$vault).0.is_empty()) {
                    @{empty("No notes match")}
                }
            ]
            --
            if !$nav.touch {
                Node { height: px(40), padding: UiRect::horizontal(px(12)), align_items: AlignItems::Center, border: UiRect::top(px(1)) }
                BorderColor::all(color::BORDER)
                Children [ @{shortcuts(&[("↑ ↓", "navigate"), ("↵", "open"), ("Esc", "close")])} ]
            }
        ]
    }
}

#[component]
fn row(nav: Watch<Res<Nav>>, row: Row, vault: &VaultData) -> impl Scene + use<> {
    let (index, id, last) = match row {
        Row::Label(text) => return Box::new(group_label(text)) as Box<dyn Scene>,
        Row::Note { index, id, last } => (index, id, last),
    };
    let note = &vault.notes[id];
    let (icon, name, path) = (note.icon, note.name.clone(), note.path.clone());
    Box::new(view! {
        @{item()}
        Node { height: $(if $nav.touch { px(44) } else { px(36) }) }
        // The keyboard can point past a shrunken list: then the last one is meant.
        Selected($nav.switcher_index == index || (last && $nav.switcher_index > index))
        on(move |_: On<Activate>, mut n: ResMut<Nav>| {
            n.open(id, None);
            n.switcher = false;
        })
        Children [
            @{item_icon_char(icon)}
            --
            @{item_label(&name)}
            --
            @{label(&path, size::XS, FontWeight::NORMAL, color::MUTED_FOREGROUND)}
            TextLayout { linebreak: LineBreak::NoWrap }
            Node { flex_shrink: 0.0 }
        ]
    })
}

//! The quick switcher (`Ctrl+O`), a command palette over the notes: type to filter, arrows to
//! move, Enter to open.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{Memo, memo, view};

use bevsidian_core::{Vault, nav::Nav, vault::NoteId};
use bevsidian_ui::prelude::*;

pub fn command_palette(nav: Nav, vault: Vault) -> impl Scene {
    let open = nav.switcher.clone();
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::COMMAND)
        Pickable::IGNORE
        Children [
            if $open {
                let close = nav.switcher.clone();
                @{scrim(move || close.set(false))}
                Node { justify_content: JustifyContent::Center, align_items: AlignItems::Start, padding: UiRect::top(Val::Percent(8.0)) }
                Children [ @{palette(nav.clone(), vault.clone())} ]
            }
        ]
    }
}

/// A row of the list: a group label, or a note with its index among the notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Row {
    Label(&'static str),
    Note(usize, NoteId),
}

fn palette(nav: Nav, vault: Vault) -> impl Scene {
    let results = {
        let (nav, vault) = (nav.clone(), vault.clone());
        memo(move || {
            // What the results depend on.
            let _ = (
                nav.switcher_query.get(),
                nav.current.get(),
                nav.back.with(Vec::len),
            );
            nav.switcher_results(&vault.get())
        })
    };
    // With no query, the notes visited last come first, under their own label.
    let rows = {
        let results = results.clone();
        memo(move || {
            let (notes, recent) = results.get();
            let mut rows = Vec::new();
            for (i, &id) in notes.iter().enumerate() {
                if i == 0 {
                    rows.push(Row::Label(if recent > 0 { "Recent" } else { "Notes" }));
                } else if i == recent {
                    rows.push(Row::Label("Notes"));
                }
                rows.push(Row::Note(i, id));
            }
            rows
        })
    };
    let query = Some(nav.switcher_query.clone());
    let (emptiness, touch) = (results.clone(), nav.touch.clone());
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
                for entry in $rows {
                    let scene = row(&nav, &vault, &results, entry);
                    @{scene}
                }
                --
                if $(emptiness.with(|(notes, _)| notes.is_empty())) {
                    @{empty("No notes match")}
                }
            ]
            --
            if !$touch {
                Node { height: px(40), padding: UiRect::horizontal(px(12)), align_items: AlignItems::Center, border: UiRect::top(px(1)) }
                BorderColor::all(color::BORDER)
                Children [ @{shortcuts(&[("↑ ↓", "navigate"), ("↵", "open"), ("Esc", "close")])} ]
            }
        ]
    }
}

fn row(nav: &Nav, vault: &Vault, results: &Memo<(Vec<NoteId>, usize)>, row: Row) -> Box<dyn Scene> {
    let (i, id) = match row {
        Row::Label(text) => return Box::new(group_label(text)),
        Row::Note(i, id) => (i, id),
    };
    let (open, index, touch, listed) = (
        nav.clone(),
        nav.switcher_index.clone(),
        nav.touch.clone(),
        results.clone(),
    );
    let data = vault.get_untracked();
    let note = &data.notes[id];
    let (icon, name, path) = (note.icon, note.name.clone(), note.path.clone());
    Box::new(view! {
        @{item()}
        Node { height: $(if $touch { px(44) } else { px(36) }) }
        // The keyboard can point past a shrunken list: then the last one is meant.
        Selected($($index.min(listed.with(|(notes, _)| notes.len()).saturating_sub(1)) == i))
        on(move |_: On<Activate>| {
            open.open(id, None);
            open.switcher.set(false);
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

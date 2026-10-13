//! The quick switcher (`Ctrl+O`), a command palette over the notes: type to filter, arrows to
//! move, Enter to open.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};
use bevy_radioactive::view::{memo, view};

use bevsidian_core::{
    Vault,
    nav::{Nav, SWITCHER_RESULTS},
};
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

fn palette(nav: Nav, vault: Vault) -> impl Scene {
    let results = {
        let (query, vault) = (nav.switcher_query.clone(), vault.clone());
        memo(move || {
            vault
                .get()
                .search(&query.get(), None)
                .into_iter()
                .take(SWITCHER_RESULTS)
                .collect::<Vec<_>>()
        })
    };
    let (index, touch) = (nav.switcher_index.clone(), nav.touch.clone());
    let query = Some(nav.switcher_query.clone());
    let (listing, emptiness) = (results.clone(), results.clone());
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
                @{group_label("Notes")}
                --
                #[key(*entry)]
                for entry in $(listing.get().into_iter().enumerate().collect::<Vec<_>>()) {
                    let (i, id) = entry;
                    let (nav, listed) = (nav.clone(), results.clone());
                    let data = vault.get_untracked();
                    let note = &data.notes[id];
                    @{item()}
                    Node { height: $(if $touch { px(44) } else { px(36) }) }
                    // The keyboard can point past a shrunken list: then the last one is meant.
                    Selected($($index.min(listed.with(Vec::len).saturating_sub(1)) == i))
                    on(move |_: On<Activate>| {
                        nav.open(id, None);
                        nav.switcher.set(false);
                    })
                    Children [
                        @{item_icon_char(note.icon)}
                        --
                        @{item_label(&note.name)}
                        --
                        @{label(&note.path, size::XS, FontWeight::NORMAL, color::MUTED_FOREGROUND)}
                        TextLayout { linebreak: LineBreak::NoWrap }
                        Node { flex_shrink: 0.0 }
                    ]
                }
                --
                if $(emptiness.with(Vec::is_empty)) {
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

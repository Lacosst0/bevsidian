//! The open note's links: notes linking to it (expanded) and notes it links to (folded).

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{Signal, memo, signal, view};

use bevsidian_core::{
    Vault,
    nav::Nav,
    vault::{NoteId, VaultData},
};
use bevsidian_ui::prelude::*;

pub fn links(nav: Nav, vault: Vault) -> impl Scene {
    let backlinks_open = signal(|| true);
    let outgoing_open = signal(|| false);
    let key = {
        let (current, touch, vault) = (nav.current.clone(), nav.touch.clone(), vault.clone());
        memo(move || (current.get(), touch.get(), vault.get().revision))
    };
    view! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(1), margin: UiRect::top(px(16)), flex_shrink: 0.0 }
        Children [
            @{group_label("Links")}
            --
            match $key {
                (id, touch, _) => {
                    let data = vault.get_untracked();
                    @{branch("undo-2", "Backlinks", &data.notes[id].backlinks, &data, touch, &nav, backlinks_open.clone())}
                    --
                    @{branch("arrow-up-right", "Outgoing links", &data.notes[id].outgoing, &data, touch, &nav, outgoing_open.clone())}
                },
            }
        ]
    }
}

/// A folding row with a count, over the linked notes. With no links it is dimmed and does not
/// open.
fn branch(
    icon: &str,
    text: &str,
    notes: &[NoteId],
    data: &VaultData,
    touch: bool,
    nav: &Nav,
    open: Signal<bool>,
) -> impl Scene + use<> {
    let rows: Vec<Box<dyn Scene>> = notes
        .iter()
        .map(|&target| {
            let nav = nav.clone();
            let note = &data.notes[target];
            Box::new(bsn! {
                @{sub_item(Some(note.icon), &note.name)}
                Node { height: px(if touch { 36.0 } else { 28.0 }) }
                on(move |_: On<Activate>| nav.open(target, None))
            }) as Box<dyn Scene>
        })
        .collect();
    let empty = rows.is_empty();
    let (shown, flip, toggle) = (open.clone(), open.clone(), open.clone());
    let chevron: Vec<Box<dyn Scene>> = (!empty)
        .then(|| {
            Box::new(disclosure(
                move || shown.get(),
                move || flip.update(|o| *o = !*o),
            )) as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    view! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            @{item()}
            Node { height: px(if touch { 40.0 } else { 32.0 }) }
            Disabled(empty)
            on(move |_: On<Activate>| toggle.update(|o| *o = !*o))
            Children [
                @{item_icon(icon)}
                --
                @{item_label(text)}
                --
                @{count(notes.len())}
                --
                {chevron}
            ]
            --
            @{tree_children()}
            Node { display: $(if !empty && $open { Display::Flex } else { Display::None }) }
            Children [ {rows} ]
        ]
    }
}

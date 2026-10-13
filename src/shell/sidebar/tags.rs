//! Every tag with its note count; the active one filters the note list.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{memo, view};

use bevsidian_core::{Vault, nav::Nav};
use bevsidian_ui::prelude::*;

pub fn tags(nav: Nav, vault: Vault) -> impl Scene {
    let tags = {
        let vault = vault.clone();
        memo(move || {
            vault
                .get()
                .tags
                .iter()
                .map(|(name, notes)| (name.clone(), notes.len()))
                .collect::<Vec<_>>()
        })
    };
    let (touch, active) = (nav.touch.clone(), nav.tag.clone());
    view! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(1), margin: UiRect::vertical(px(16)), flex_shrink: 0.0 }
        Children [
            if !$tags.is_empty() {
                @{group_label("Tags")}
            }
            --
            #[key(entry.clone())]
            for entry in $tags {
                let (name, notes) = entry;
                let (nav, filter, text) = (nav.clone(), name.clone(), name.clone());
                @{item()}
                Node { height: $(if $touch { px(40) } else { px(32) }) }
                Selected($($active.as_deref() == Some(name.as_str())))
                on(move |_: On<Activate>| nav.toggle_tag(&filter))
                Children [
                    @{hash()}
                    --
                    @{item_label(&text)}
                    --
                    @{count(notes)}
                ]
            }
        ]
    }
}

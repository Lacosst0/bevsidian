//! "What's in your game?": the map's checklist. The reader ticks the features their game has
//! (and what else they want to get better at); the map then shows the main quest and the
//! roads to those features, and leaves the rest in the fog. It opens the first time the map
//! is shown, and from the map's quest log.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{Signal, signal, view};

use bevsidian_core::{
    nav::{Nav, View},
    vault::{NoteId, Quest, Vault},
};
use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, ask_on_first_map);
}

/// The first time the map is shown, and nothing is chosen yet, the checklist asks.
fn ask_on_first_map(nav: Res<Nav>, mut asked: Local<bool>) {
    if *asked || nav.view.get_untracked() != View::Graph {
        return;
    }
    *asked = true;
    if nav.quests.with_untracked(Option::is_none) {
        nav.map_setup.set(true);
    }
}

pub fn quest_dialog(nav: Nav, vault: Vault) -> impl Scene {
    let open = nav.map_setup.clone();
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::DIALOG)
        Pickable::IGNORE
        Children [
            if $open {
                let close = nav.map_setup.clone();
                @{scrim(move || close.set(false))}
                Node { justify_content: JustifyContent::Center, align_items: AlignItems::Start, padding: UiRect::top(Val::Percent(6.0)) }
                Children [ @{panel(nav.clone(), vault.clone())} ]
            }
        ]
    }
}

/// A route offered in the checklist, and whether it is ticked.
struct Choice {
    note: NoteId,
    icon: char,
    label: String,
    ticked: Signal<bool>,
}

fn panel(nav: Nav, vault: Vault) -> impl Scene {
    let data = vault.get_untracked();
    let chosen = nav.quests.get_untracked().unwrap_or_default();
    let choices = |quest: Quest| -> Vec<Choice> {
        data.paths
            .iter()
            .filter(|p| p.quest == quest)
            .map(|p| {
                let ticked = chosen.contains(&p.note);
                Choice {
                    note: p.note,
                    icon: data.notes[p.note].icon,
                    label: p.label.clone(),
                    ticked: signal(move || ticked),
                }
            })
            .collect()
    };
    let (features, mastery) = (choices(Quest::Feature), choices(Quest::Mastery));
    let main = data
        .paths
        .iter()
        .find(|p| p.quest == Quest::Main)
        .map(|p| data.notes[p.note].name.clone())
        .unwrap_or_default();
    // What is ticked, to keep when the reader is done.
    let ticked: Vec<(NoteId, Signal<bool>)> = features
        .iter()
        .chain(&mastery)
        .map(|c| (c.note, c.ticked.clone()))
        .collect();
    let picked = move || -> Vec<NoteId> {
        ticked
            .iter()
            .filter(|(_, t)| t.get_untracked())
            .map(|&(note, _)| note)
            .collect()
    };
    let (start_pick, whole_pick) = (picked.clone(), picked);
    let (close, start, whole) = (nav.clone(), nav.clone(), nav);
    let note = format!("{main}, the main quest, is always on the map.");
    view! {
        @{dialog(600.0)}
        Node { row_gap: px(16) }
        Children [
            Node { flex_direction: FlexDirection::Column, row_gap: px(6) }
            Children [
                @{dialog_header("What's in your game?", move || close.map_setup.set(false))}
                --
                @{muted("Tick what your game has. The map shows the roads to build it, starting from Bevy; everything else waits, folded, one click away.")}
            ]
            --
            @{group(features, "Your game has")}
            --
            @{group(mastery, "You also want to")}
            --
            Node { align_items: AlignItems::Center, column_gap: px(8) }
            Children [
                @{icon("rocket", 14.0, color::MUTED_FOREGROUND)}
                --
                @{caption(&note)}
            ]
            --
            @{dialog_footer()}
            Node { flex_wrap: FlexWrap::Wrap, row_gap: px(8) }
            Children [
                @{text_button(Variant::Outline, Size::Sm, "Show the whole map")}
                on(move |_: On<Activate>| {
                    whole.quests.set(Some(whole_pick()));
                    whole.whole_map.set(true);
                    whole.map_setup.set(false);
                })
                --
                @{text_button(Variant::Primary, Size::Sm, "Show my roads")}
                on(move |_: On<Activate>| {
                    start.quests.set(Some(start_pick()));
                    start.whole_map.set(false);
                    start.map_setup.set(false);
                })
            ]
        ]
    }
}

/// A titled grid of choices, two to a row.
fn group(choices: Vec<Choice>, title: &str) -> impl Scene + use<> {
    let rows: Vec<Box<dyn Scene>> = choices.into_iter().map(choice).collect();
    bsn! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            @{group_label(title)}
            Node { padding: UiRect::ZERO }
            --
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex::<Vec<RepeatedGridTrack>>(2, 1.0),
                column_gap: px(4),
                row_gap: px(2),
            }
            Children [ {rows} ]
        ]
    }
}

/// One route: its checkbox, icon and what it gives the game.
fn choice(choice: Choice) -> Box<dyn Scene> {
    let toggle = choice.ticked.clone();
    Box::new(bsn! {
        @{item()}
        Node { height: px(36), min_width: px(0) }
        on(move |_: On<Activate>| toggle.update(|on| *on = !*on))
        Children [
            @{checkbox(choice.ticked.clone())}
            --
            @{item_icon_char(choice.icon)}
            --
            @{item_label(&choice.label)}
        ]
    })
}

//! "What's in your game?": the map's checklist. The reader ticks the features their game has
//! (and what else they want to get better at); the map then shows the main quest and the
//! roads to those features, and leaves the rest out until the whole map is shown. It opens the first time the map
//! is shown, and from the map's quest log.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::{
    nav::{Nav, View},
    vault::{NoteId, Quest, Vault},
};
use bevsidian_graph::route_color;
use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, ask_on_first_map);
}

/// The first time the map is shown, and nothing is chosen yet, the checklist asks.
fn ask_on_first_map(mut nav: ResMut<Nav>, mut asked: Local<bool>) {
    if *asked || nav.view != View::Graph {
        return;
    }
    *asked = true;
    if nav.quests.is_none() {
        nav.map_setup = true;
    }
}

#[component]
pub fn quest_dialog(nav: Watch<Res<Nav>>, vault: Watch<Res<Vault>>) -> impl Scene {
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0) }
        GlobalZIndex(layer::DIALOG)
        Pickable::IGNORE
        Children [
            if $nav.map_setup {
                @{scrim(close)}
                Node { justify_content: JustifyContent::Center, align_items: AlignItems::Start, padding: UiRect::top(Val::Percent(6.0)) }
                Children [
                    // The routes chosen so far start ticked.
                    @{panel(vault.get_untracked(), nav.with_untracked(|n| n.quests.clone().unwrap_or_default()))}
                ]
            }
        ]
    }
}

fn close(world: &mut World) {
    world.resource_mut::<Nav>().map_setup = false;
}

/// A route offered in the checklist.
struct Choice {
    note: NoteId,
    icon: char,
    /// The route's colour on the map.
    tint: Color,
    label: String,
}

/// `ticked` holds the routes ticked so far, starting with the `chosen` ones.
#[component]
fn panel(
    vault: Vault,
    chosen: Vec<NoteId>,
    #[init(chosen)] ticked: Watch<Local<Vec<NoteId>>>,
) -> impl Scene {
    let choices = |quest: Quest| -> Vec<Choice> {
        vault
            .paths
            .iter()
            .filter(|p| p.quest == quest)
            .map(|p| Choice {
                note: p.note,
                icon: vault.notes[p.note].icon,
                tint: route_color(&vault, p.note),
                label: p.label.clone(),
            })
            .collect()
    };
    let (features, mastery) = (choices(Quest::Feature), choices(Quest::Mastery));
    let main = vault
        .paths
        .iter()
        .find(|p| p.quest == Quest::Main)
        .map(|p| vault.notes[p.note].name.clone())
        .unwrap_or_default();
    // What is ticked, in the checklist's order, to keep when the reader is done.
    let order: Vec<NoteId> = features.iter().chain(&mastery).map(|c| c.note).collect();
    let picked = {
        let ticked = ticked.clone();
        move || -> Vec<NoteId> {
            ticked.with_untracked(|ticked| {
                order
                    .iter()
                    .copied()
                    .filter(|note| ticked.contains(note))
                    .collect()
            })
        }
    };
    let (start_pick, whole_pick) = (picked.clone(), picked);
    let (features, mastery) = (
        group(features, &ticked, "Your game has"),
        group(mastery, &ticked, "You also want to"),
    );
    let note = format!("{main}, the main quest, is always on the map.");
    bsn! {
        @{dialog(600.0)}
        Node { row_gap: px(16) }
        Children [
            Node { flex_direction: FlexDirection::Column, row_gap: px(6) }
            Children [
                @{dialog_header("What's in your game?", close)}
                --
                @{muted("Tick what your game has. The map shows the routes to build it, starting from Bevy; everything else waits on the whole map, one click away.")}
            ]
            --
            @{features}
            --
            @{mastery}
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
                on(move |_: On<Activate>, mut n: ResMut<Nav>| {
                    n.quests = Some(whole_pick());
                    n.whole_map = true;
                    n.map_setup = false;
                })
                --
                @{text_button(Variant::Primary, Size::Sm, "Show my routes")}
                on(move |_: On<Activate>, mut n: ResMut<Nav>| {
                    n.quests = Some(start_pick());
                    n.whole_map = false;
                    n.map_setup = false;
                })
            ]
        ]
    }
}

/// A titled grid of choices, two to a row.
fn group(
    choices: Vec<Choice>,
    ticked: &Watch<Local<'static, Vec<NoteId>>>,
    title: &str,
) -> impl Scene + use<> {
    let rows: Vec<Box<dyn Scene>> = choices
        .into_iter()
        .map(|c| choice(c, ticked.clone()))
        .collect();
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

/// One route: its checkbox, icon (in the route's colour on the map) and what it gives the
/// game.
fn choice(choice: Choice, ticked: Watch<Local<'static, Vec<NoteId>>>) -> Box<dyn Scene> {
    let note = choice.note;
    let checked = Binding::local_part(
        ticked,
        move |ticked: &Vec<NoteId>| ticked.contains(&note),
        move |ticked, on| {
            if on {
                ticked.push(note);
            } else {
                ticked.retain(|&t| t != note);
            }
        },
    );
    let toggle = checked.clone();
    Box::new(bsn! {
        @{item()}
        Node { height: px(36), min_width: px(0) }
        on(move |_: On<Activate>, mut commands: Commands| toggle.update(&mut commands, |on| !on))
        Children [
            @{checkbox(checked.clone())}
            --
            @{icon_char(choice.icon, 16.0, choice.tint)}
            --
            @{item_label(&choice.label)}
        ]
    })
}

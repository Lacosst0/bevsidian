//! Tests of the map against the real vault.

use super::*;
use bevsidian_core::vault::{Quest, StepKind};

/// The real vault, read from disk.
fn real_vault() -> VaultData {
    fn collect(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|e| e == "md") {
                let name = path
                    .strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/../../vault"))
                    .unwrap();
                out.push((
                    name.to_string_lossy().into_owned(),
                    std::fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }
    let mut notes = Vec::new();
    collect(
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../vault")),
        &mut notes,
    );
    VaultData::from_sources(notes.iter().map(|(p, s)| (p.as_str(), s.as_str())))
}

#[test]
fn every_note_of_the_course_is_on_the_map() {
    let vault = real_vault();
    let map = vault.map.as_ref().expect("the vault has a map");
    let layout = home_positions(&vault);
    let name = |slot: usize| vault.notes[layout.notes[slot]].name.as_str();
    assert_eq!(name(layout.root), "Bevy", "the capital is the hub");
    assert_eq!(layout.homes[layout.root], Vec2::ZERO);
    for slot in 0..layout.notes.len() {
        assert!(
            map.place(name(slot)).is_some(),
            "{} is not on the map",
            name(slot)
        );
    }
    // Nothing on the map that is not a note, and the app's own notes stay off it.
    for place in &map.places {
        let id = vault
            .find(&place.name)
            .unwrap_or_else(|| panic!("{} is not a note", place.name));
        assert!(
            layout.notes.contains(&id),
            "{} should not be on the map",
            place.name
        );
    }
    for app in ["Welcome", "Bevsidian", "UI kit"] {
        assert!(!layout.notes.contains(&vault.find(app).unwrap()), "{app}");
    }
    // Every note hangs from the hub, through notes of the map.
    for slot in 0..layout.notes.len() {
        let (mut at, mut steps) = (slot, 0);
        while let Some(up) = layout.parent[at] {
            at = up;
            steps += 1;
            assert!(steps < 8, "{} is too deep or loops", name(slot));
        }
        assert_eq!(at, layout.root, "{}", name(slot));
    }
}

#[test]
fn notes_on_the_map_do_not_crowd_each_other() {
    let vault = real_vault();
    let layout = home_positions(&vault);
    for a in 0..layout.notes.len() {
        for b in (a + 1)..layout.notes.len() {
            let apart = layout.homes[a].distance(layout.homes[b]);
            assert!(
                apart > 110.0,
                "{} and {} are {apart} apart",
                vault.notes[layout.notes[a]].name,
                vault.notes[layout.notes[b]].name
            );
        }
    }
}

#[test]
fn the_routes_make_a_game_map() {
    let vault = real_vault();
    let main: Vec<_> = vault
        .paths
        .iter()
        .filter(|p| p.quest == Quest::Main)
        .collect();
    assert_eq!(main.len(), 1, "one main quest");
    let features = vault
        .paths
        .iter()
        .filter(|p| p.quest == Quest::Feature)
        .count();
    assert!(features >= 8, "a choice of features: {features}");
    for path in &vault.paths {
        let name = &vault.notes[path.note].name;
        assert_ne!(path.quest, Quest::Other, "{name} has no quest");
        assert!(path.steps.len() >= 3, "{name} is too short");
        assert_eq!(
            path.kinds[0],
            StepKind::Core,
            "{name} starts on its main road"
        );
        assert!(!path.summary.is_empty(), "{name} has no summary");
        if path.quest != Quest::Main {
            assert_ne!(&path.label, name, "{name} has no feature label");
        }
    }
    // Roads branch off the ring road: no road but the main quest's leaves the capital.
    let layout = home_positions(&vault);
    let place = |id: NoteId| {
        layout
            .notes
            .iter()
            .position(|&n| n == id)
            .map(|s| layout.homes[s])
    };
    let hub = layout.notes[layout.root];
    let roads = routes::roads(&vault, &place, hub);
    let main = main[0].note;
    for (a, b, uses) in &roads {
        assert!(place(*a).is_some() && place(*b).is_some());
        if *a == hub || *b == hub {
            assert!(uses.iter().all(|(r, _)| *r == main), "{uses:?}");
        }
        // No road is longer than half the map.
        let length = place(*a).unwrap().distance(place(*b).unwrap());
        assert!(
            length < 1300.0,
            "{} to {} is {length} long",
            vault.notes[*a].name,
            vault.notes[*b].name
        );
    }
}

#[test]
fn a_folded_note_unfolds_out_of_its_parent_and_folds_back_in() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        // Every update is one 60 Hz frame.
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(1.0 / 60.0),
        ))
        .insert_resource(Nav::new(99))
        .init_resource::<GraphState>()
        .add_systems(Update, (fold::fold_nodes, motion).chain());
    app.world().resource::<Nav>().view.set(View::Graph);
    let node = |note: usize, rank: usize, home: Vec2, parent: Option<Entity>| {
        (
            GraphNode {
                note,
                radius: 20.0,
                shape: Shape::Circle,
                rank,
                forced: rank == 0,
                step: None,
                parent,
                folds: 0,
                open: false,
                hidden: false,
                tucking: false,
                tuck_to: Vec2::ZERO,
                home,
                arriving: false,
            },
            Transform::from_translation(home.extend(0.0)),
            Visibility::default(),
        )
    };
    let hub = app.world_mut().spawn(node(0, 0, Vec2::ZERO, None)).id();
    let crate_note = app
        .world_mut()
        .spawn(node(1, 4, Vec2::new(300.0, 0.0), Some(hub)))
        .id();
    let run = |app: &mut App, frames: usize| {
        for _ in 0..frames {
            app.update();
        }
    };
    let get = |app: &App, e: Entity| {
        let node = app.world().get::<GraphNode>(e).unwrap();
        let at = app
            .world()
            .get::<Transform>(e)
            .unwrap()
            .translation
            .truncate();
        (node.hidden, node.folds, at)
    };
    run(&mut app, 1);
    assert!(get(&app, crate_note).0, "off every route: folded at first");
    assert_eq!(get(&app, hub).1, 1, "the hub holds it");

    app.world_mut().resource_mut::<GraphState>().fold_toggle = Some(hub);
    run(&mut app, 1);
    let (hidden, _, at) = get(&app, crate_note);
    assert!(!hidden && at.x < 50.0, "it unfolds out of its parent: {at}");
    run(&mut app, 240);
    assert!(
        (get(&app, crate_note).2.x - 300.0).abs() < 1.0,
        "and floats home"
    );

    app.world_mut().resource_mut::<GraphState>().fold_toggle = Some(hub);
    run(&mut app, 1);
    assert!(
        !get(&app, crate_note).0,
        "folding, it first floats back into its parent"
    );
    run(&mut app, 240);
    let (hidden, _, at) = get(&app, crate_note);
    assert!(hidden && at.x < 10.0, "then disappears there: {at}");
    assert_eq!(
        app.world().get::<GraphNode>(crate_note).unwrap().home,
        Vec2::new(300.0, 0.0),
        "its place is kept"
    );

    // With the whole map, a crate still folds (it is beyond the concepts); editing shows it.
    app.world_mut().resource_mut::<GraphState>().whole = true;
    run(&mut app, 2);
    assert!(get(&app, crate_note).0);
    app.world().resource::<Nav>().map_edit.set(true);
    run(&mut app, 2);
    assert!(!get(&app, crate_note).0, "editing shows every note");
}

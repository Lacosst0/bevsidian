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

fn main_steps(vault: &VaultData) -> Vec<NoteId> {
    vault
        .paths
        .iter()
        .find(|p| p.quest == Quest::Main)
        .map(|p| p.steps.clone())
        .unwrap_or_default()
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
}

#[test]
fn notes_on_the_map_do_not_crowd_each_other() {
    let vault = real_vault();
    let layout = home_positions(&vault);
    for a in 0..layout.notes.len() {
        for b in (a + 1)..layout.notes.len() {
            let apart = layout.homes[a].distance(layout.homes[b]);
            assert!(
                apart > 100.0,
                "{} and {} are {apart} apart",
                vault.notes[layout.notes[a]].name,
                vault.notes[layout.notes[b]].name
            );
        }
    }
}

#[test]
fn every_route_is_a_quest_on_the_map() {
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
            "{name} starts with a core step"
        );
        if path.quest != Quest::Main {
            assert_ne!(&path.label, name, "{name} has no feature label");
        }
    }
    // Routes branch off the main quest: no line but the main quest's leaves the capital.
    let layout = home_positions(&vault);
    let place = |id: NoteId| layout.slot(id).map(|s| layout.homes[s]);
    let hub = layout.notes[layout.root];
    let drawn = routes::lines(&vault, &place, hub);
    let main = main[0].note;
    for (a, b, uses) in &drawn.lines {
        assert!(place(*a).is_some() && place(*b).is_some());
        if *a == hub || *b == hub {
            assert!(uses.iter().all(|u| u.route == main), "{uses:?}");
        }
        // No line runs across the map.
        let length = place(*a).unwrap().distance(place(*b).unwrap());
        assert!(
            length < 1000.0,
            "{} to {} is {length} long",
            vault.notes[*a].name,
            vault.notes[*b].name
        );
    }
    // Every route but the main quest leaves it, by a line or at one of its stops.
    let stops = &main_steps(&vault);
    for path in vault.paths.iter().filter(|p| p.quest != Quest::Main) {
        assert!(
            drawn.starts.iter().any(|(route, ..)| *route == path.note)
                || stops.contains(&path.steps[0]),
            "{} does not leave the main quest",
            vault.notes[path.note].name
        );
    }
}

#[test]
fn a_note_that_appears_floats_home() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        // Every update is one 60 Hz frame.
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(1.0 / 60.0),
        ))
        .insert_resource(Nav::new(99))
        .init_resource::<GraphState>()
        .add_systems(Update, (show::show_nodes, motion).chain());
    let node = |note: usize, forced: bool, home: Vec2| {
        (
            GraphNode {
                note,
                radius: 20.0,
                rank: usize::from(!forced),
                forced,
                step: None,
                hidden: false,
                home,
                arriving: false,
            },
            Transform::from_translation(home.extend(0.0)),
            Visibility::default(),
        )
    };
    let hub = app.world_mut().spawn(node(0, true, Vec2::ZERO)).id();
    let other = app
        .world_mut()
        .spawn(node(1, false, Vec2::new(300.0, 0.0)))
        .id();
    let get = |app: &App, e: Entity| {
        let hidden = app.world().get::<GraphNode>(e).unwrap().hidden;
        let at = app
            .world()
            .get::<Transform>(e)
            .unwrap()
            .translation
            .truncate();
        (hidden, at)
    };
    app.update();
    assert!(!get(&app, hub).0, "the hub always shows");
    assert!(get(&app, other).0, "off every route shown: hidden");

    app.world_mut().resource_mut::<GraphState>().whole = true;
    app.update();
    let (hidden, at) = get(&app, other);
    assert!(
        !hidden && at.x < 299.0,
        "with the whole map it appears, sliding in: {at}"
    );
    for _ in 0..240 {
        app.update();
    }
    assert!(
        (get(&app, other).1.x - 300.0).abs() < 1.0,
        "and settles at its place"
    );
}

#[test]
fn zoom_stops_at_its_limits() {
    let mut transform = Transform::default();
    let mut projection = Projection::Orthographic(OrthographicProjection::default_2d());
    for _ in 0..40 {
        zoom_around(
            &mut transform,
            &mut projection,
            Vec2::new(100.0, 50.0),
            0.5,
            4.0,
        );
    }
    assert!(
        (ortho_scale(&projection) - MIN_SCALE).abs() < 1e-5,
        "no closer than the limit"
    );
    for _ in 0..40 {
        zoom_around(
            &mut transform,
            &mut projection,
            Vec2::new(100.0, 50.0),
            2.0,
            4.0,
        );
    }
    assert!(
        (ortho_scale(&projection) - 4.0).abs() < 1e-5,
        "no farther than the map allows"
    );

    // The farthest zoom follows the size of the map.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<GraphState>()
        .add_systems(Update, limit_zoom);
    let viewport = ComputedNode {
        size: Vec2::new(1000.0, 800.0),
        ..default()
    };
    app.world_mut().spawn((GraphViewport, viewport));
    app.world_mut().spawn((
        GraphCamera,
        Transform::default(),
        Projection::Orthographic(OrthographicProjection {
            scale: 9.0,
            ..OrthographicProjection::default_2d()
        }),
    ));
    for x in [-1000.0, 1000.0] {
        app.world_mut().spawn(GraphNode {
            note: 0,
            radius: 20.0,
            rank: 2,
            forced: false,
            step: None,
            hidden: false,
            home: Vec2::new(x, 0.0),
            arriving: false,
        });
    }
    app.update();
    let farthest = app.world().resource::<GraphState>().farthest;
    assert!(
        (farthest - 2180.0 / 1000.0 * ZOOM_OUT_ROOM).abs() < 1e-3,
        "{farthest}"
    );
    let goal = app.world().resource::<GraphState>().camera_goal;
    assert_eq!(
        goal,
        Some((Vec2::ZERO, farthest)),
        "the camera eases back in"
    );
}

#[test]
fn every_note_has_a_line() {
    let vault = real_vault();
    let layout = home_positions(&vault);
    let place = |id: NoteId| layout.slot(id).map(|s| layout.homes[s]);
    let drawn = routes::lines(&vault, &place, layout.notes[layout.root]);
    let on: std::collections::HashSet<NoteId> =
        drawn.lines.iter().flat_map(|(a, b, _)| [*a, *b]).collect();
    let ties = routes::ties(&vault, &place, &layout.notes, &on);
    for &id in &layout.notes {
        assert!(
            on.contains(&id) || ties.iter().any(|&(n, _)| n == id),
            "{} has no line",
            vault.notes[id].name
        );
    }
    for &(note, to) in &ties {
        let length = place(note).unwrap().distance(place(to).unwrap());
        assert!(
            length < 330.0,
            "{} is tied {length} away, to {}",
            vault.notes[note].name,
            vault.notes[to].name
        );
    }
    // Crates hang from the crates they are compared in.
    let tie = |name: &str| {
        let id = vault.find(name).unwrap();
        let (_, to) = ties.iter().find(|&&(n, _)| n == id).unwrap();
        vault.notes[*to].name.clone()
    };
    assert_eq!(tie("bevy_ecs_ldtk"), "Tilemap crates");
    assert_eq!(tie("bevy_kira_audio"), "Audio crates");
    assert_eq!(tie("bevy_steamworks"), "Netcode crates");
}

#[test]
fn related_routes_sit_side_by_side_in_neighbouring_colours() {
    let vault = real_vault();
    let wheel = route_wheel(&vault);
    let at = |name: &str| {
        let id = vault.find(name).unwrap();
        wheel.iter().position(|&r| r == id).unwrap()
    };
    for (a, b) in [
        ("Building UI", "Learn BSN"),
        ("A 2D game", "A 3D game"),
        ("A 3D game", "Make it look good"),
        ("Controls", "Collisions"),
    ] {
        assert_eq!(at(a).abs_diff(at(b)), 1, "{a} and {b}");
    }
    // Every route its own colour.
    let colours: Vec<_> = wheel
        .iter()
        .map(|&r| route_color(&vault, r).to_srgba().to_hex())
        .collect();
    for (i, c) in colours.iter().enumerate() {
        assert!(!colours[..i].contains(c), "{c} twice");
    }
}

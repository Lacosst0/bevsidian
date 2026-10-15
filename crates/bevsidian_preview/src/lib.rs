//! Bevy-in-Bevy: live previews above the code blocks of a note.
//!
//! - `example=NAME` runs an [example](examples) in a [`World`] of its own: the example's
//!   program with the functions the block's code names (see [`example`]), so a note's blocks
//!   can share one example and each run its own part. The world is shown as
//!   - **cards** of its entities and their component values. When the block runs more than
//!     startup code, Bevy's [`Stepping`](bevy::ecs::schedule::Stepping) runs one system at a
//!     time: step forward, play / pause, step back (a replay from the start, as stepping only
//!     goes forward), and the code lines of the system that just ran are highlighted;
//!   - a **game**: its sprites, playing in real time, with pointer input turned into picking
//!     events in its world.
//! - `preview=NAME` spawns a [UI scene](examples::scenes) inside the preview.

mod cards;
pub mod example;
mod examples;
mod game;
mod runtime;
pub mod sound;
pub mod yours;

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_ui_commonmark::{CodeBlockInfo, CodeHighlight, CodePreview, MdCodeGroup};

use bevsidian_core::nav::{Nav, Studio};
use bevsidian_ui::{
    Size, Tooltip, Variant, button, glyph, inset, text_button, tinted_icon, vertical_separator,
};
pub use runtime::PreviewRuntime;

use example::{Action, Kind, Options, View};

pub fn plugin(app: &mut App) {
    app.add_plugins((examples::plugin, yours::plugin))
        .add_observer(start_world_preview)
        .add_systems(
            Update,
            (
                rebuild_previews,
                play_previews,
                yours::play_sounds,
                (show_controls, cards::show_cards, game::draw_games),
            )
                .chain(),
        );
}

/// The preview shown above a code block whose info string names an example
/// (`example=NAME`) or a UI scene (`preview=NAME`).
pub fn code_preview(block: &CodeBlockInfo) -> Option<CodePreview> {
    if let Some(options) = Options::parse(block.info) {
        let example = examples::find(&options.name)?;
        let preview_name = options.name.clone();
        let preview = WorldPreview {
            options,
            code: block.code.to_owned(),
        };
        return Some(match example.view {
            View::Cards { .. } => {
                let stepping = example
                    .functions_for(block.code)
                    .iter()
                    .any(|f| f.kind != Kind::Startup && f.kind != Kind::Observer);
                let buttons = example.actions_for(block.code);
                CodePreview {
                    body: Box::new(cards::cards_frame(preview)),
                    actions: (stepping || !buttons.is_empty()).then(|| {
                        Box::new(controls(Playback::Cards { stepping }, &buttons, None))
                            as Box<dyn Scene>
                    }),
                }
            }
            View::Game { size, background } => {
                let interactive = example
                    .functions_for(block.code)
                    .iter()
                    .any(|f| f.kind == Kind::Observer);
                CodePreview {
                    body: game::game_frame(preview, size, background, interactive),
                    actions: Some(Box::new(controls(
                        Playback::Game,
                        &example.actions_for(block.code),
                        yours::studio_for(&preview_name),
                    ))),
                }
            }
        });
    }
    let name = block
        .info
        .split_whitespace()
        .find_map(|word| word.strip_prefix("preview="))?;
    let scene = examples::scenes::find(name)?;
    Some(CodePreview {
        body: Box::new(scene_frame(scene())),
        actions: None,
    })
}

/// A world preview to start: its example and the code it shows.
#[derive(Component, Clone, Debug, Default)]
struct WorldPreview {
    options: Options,
    code: String,
}

fn start_world_preview(add: On<Add<WorldPreview>>, mut commands: Commands) {
    let entity = add.entity;
    commands.queue(move |world: &mut World| {
        let Some(preview) = world.get::<WorldPreview>(entity).cloned() else {
            return;
        };
        let Some(example) = examples::find(&preview.options.name) else {
            return;
        };
        let setup = (example.setup)(world);
        let runtime = PreviewRuntime::new(example, setup, &preview.code, preview.options.fps);
        if let Ok(mut host) = world.get_entity_mut(entity) {
            host.insert(runtime);
        }
    });
}

/// Starts again the previews built from the reader's model when it changes, so they show the
/// new one.
fn rebuild_previews(
    yours: Res<yours::Yours>,
    previews: Query<(Entity, &WorldPreview)>,
    mut seen: Local<u64>,
    mut commands: Commands,
) {
    if *seen == yours.revision {
        return;
    }
    let first = *seen == 0;
    *seen = yours.revision;
    if first {
        return;
    }
    for (entity, preview) in &previews {
        if yours::studio_for(&preview.options.name) == Some(Studio::Voxels) {
            commands.entity(entity).remove::<PreviewRuntime>();
            let preview = preview.clone();
            commands
                .entity(entity)
                .remove::<WorldPreview>()
                .insert(preview);
        }
    }
}

fn play_previews(time: Res<Time<Real>>, mut previews: Query<&mut PreviewRuntime>) {
    let now = time.elapsed_secs_f64();
    for mut preview in &mut previews {
        if !preview.playing {
            continue;
        }
        if !preview.stepping {
            preview.advance(time.delta_secs_f64());
        } else if now >= preview.next_tick {
            let hold = preview.play_step();
            // Catch up at most one step after a stall.
            preview.next_tick = (preview.next_tick + hold).max(now);
        }
    }
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

/// Parts of the controls that change.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ControlPart {
    #[default]
    Icon,
    PlayIcon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Control {
    Back,
    Play,
    Forward,
    Reset,
}

/// How a preview plays, which decides the controls it gets.
#[derive(Clone, Copy)]
enum Playback {
    /// Cards: play, pause and step through the systems, if there are any to run.
    Cards { stepping: bool },
    /// A game: play and pause.
    Game,
}

/// Play / pause and restart (with stepping, also step back and forward), the block's action
/// buttons, and "Make it yours" where the example draws the reader's sprite or model, in the
/// code block's header.
fn controls(playback: Playback, actions: &[&Action], studio: Option<Studio>) -> impl Scene {
    let stepping = matches!(playback, Playback::Cards { stepping: true });
    let steps: Vec<Box<dyn Scene>> = if stepping {
        vec![
            Box::new(control_button(Control::Back, "step-back", "Step back")),
            Box::new(control_button(Control::Play, "play", "Play / pause")),
            Box::new(control_button(
                Control::Forward,
                "step-forward",
                "Step forward",
            )),
        ]
    } else if matches!(playback, Playback::Cards { .. }) {
        Vec::new()
    } else {
        vec![Box::new(control_button(
            Control::Play,
            "play",
            "Play / pause",
        ))]
    };
    let buttons: Vec<Box<dyn Scene>> = actions
        .iter()
        .enumerate()
        .map(|(index, action)| Box::new(action_button(index, action.label)) as Box<dyn Scene>)
        .collect();
    let make: Vec<Box<dyn Scene>> = studio
        .map(|studio| Box::new(studio_button(studio)) as Box<dyn Scene>)
        .into_iter()
        .collect();
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: px(2) }
        Children [
            {make}
            --
            {buttons}
            --
            {steps}
            --
            @{control_button(Control::Reset, "rotate-ccw", "Restart")}
            --
            @{vertical_separator()}
        ]
    }
}

/// Opens "Make it yours" on the sprite, the model or the sounds this example uses.
fn studio_button(studio: Studio) -> impl Scene + use<> {
    let (icon, tip) = match studio {
        Studio::Sprite => (
            "brush",
            "Draw your own sprite: it plays in every 2D example",
        ),
        Studio::Voxels => (
            "box",
            "Build your own voxel model: it appears in the examples",
        ),
        Studio::Sounds => ("volume-2", "Make your own sounds: the examples play them"),
    };
    bsn! {
        @{button(Variant::Ghost, Size::IconXs)}
        Tooltip::new(tip)
        on(move |_: On<Activate>, nav: Res<Nav>| nav.studio.set(Some(studio)))
        Children [ @{tinted_icon(icon, 14.0, Variant::Ghost.tint())} ]
    }
}

/// A button of the block's actions.
fn action_button(index: usize, label: &str) -> impl Scene + use<> {
    bsn! {
        @{text_button(Variant::Secondary, Size::Xs, label)}
        on(move |activate: On<Activate>,
                 parents: Query<&ChildOf>,
                 groups: Query<(), With<MdCodeGroup>>,
                 children: Query<&Children>,
                 mut previews: Query<&mut PreviewRuntime>| {
            let group = parents.iter_ancestors(activate.entity).find(|&e| groups.contains(e));
            let host = group
                .into_iter()
                .flat_map(|g| children.iter_descendants(g))
                .find(|&e| previews.contains(e));
            if let Some(mut preview) = host.and_then(|e| previews.get_mut(e).ok()) {
                preview.run_action(index);
            }
        })
    }
}

fn control_button(control: Control, icon: &str, tip: &'static str) -> impl Scene + use<> {
    let part = if control == Control::Play {
        ControlPart::PlayIcon
    } else {
        ControlPart::Icon
    };
    bsn! {
        @{button(Variant::Ghost, Size::IconXs)}
        Tooltip::new(tip)
        on(move |activate: On<Activate>,
                 time: Res<Time<Real>>,
                 parents: Query<&ChildOf>,
                 groups: Query<(), With<MdCodeGroup>>,
                 children: Query<&Children>,
                 mut previews: Query<&mut PreviewRuntime>| {
            // The controls and the preview share the code group.
            let group = parents.iter_ancestors(activate.entity).find(|&e| groups.contains(e));
            let host = group
                .into_iter()
                .flat_map(|g| children.iter_descendants(g))
                .find(|&e| previews.contains(e));
            if let Some(mut preview) = host.and_then(|e| previews.get_mut(e).ok()) {
                apply(&mut preview, control, time.elapsed_secs_f64());
            }
        })
        Children [ @{tinted_icon(icon, 14.0, Variant::Ghost.tint())} part ]
    }
}

fn apply(preview: &mut PreviewRuntime, control: Control, now: f64) {
    match control {
        Control::Back => {
            preview.playing = false;
            preview.step_back();
        }
        Control::Forward => {
            preview.playing = false;
            preview.step();
        }
        Control::Play => {
            preview.playing = !preview.playing;
            preview.next_tick = now;
        }
        Control::Reset => {
            preview.playing = false;
            preview.reset();
        }
    }
}

/// Updates the play icon and the code highlight after a preview changes.
fn show_controls(
    previews: Query<(Entity, &PreviewRuntime), Changed<PreviewRuntime>>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    parts: Query<&ControlPart>,
    groups: Query<(), With<MdCodeGroup>>,
    mut texts: Query<&mut Text>,
    mut highlights: Query<&mut CodeHighlight>,
) {
    for (host, preview) in &previews {
        let group = parents.iter_ancestors(host).find(|&e| groups.contains(e));
        for entity in group.into_iter().flat_map(|g| children.iter_descendants(g)) {
            if let (Ok(ControlPart::PlayIcon), Ok(mut text)) =
                (parts.get(entity), texts.get_mut(entity))
            {
                let icon = glyph(if preview.playing { "pause" } else { "play" });
                if text.0 != icon {
                    text.0 = icon;
                }
            }
            if preview.stepping
                && let Ok(mut highlight) = highlights.get_mut(entity)
            {
                highlight.set_if_neq(CodeHighlight(preview.highlight()));
            }
        }
    }
}

/// A UI scene, as it would look in an app.
fn scene_frame(scene: Box<dyn Scene>) -> impl Scene {
    bsn! {
        @{inset()}
        Node { align_items: AlignItems::Stretch }
        Children [ @{scene} ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ecs(code: &str) -> PreviewRuntime {
        let example = examples::find("ecs").unwrap();
        let setup = (example.setup)(&World::new());
        PreviewRuntime::new(example, setup, code, None)
    }

    #[test]
    fn a_sound_an_example_plays_plays_in_the_app() {
        use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
        let mut app = App::new();
        app.insert_resource(bevsidian_core::nav::Nav::new(0))
            .add_systems(Update, yours::play_sounds);
        let example = examples::find("sounds").unwrap();
        let setup = (example.setup)(app.world());
        let mut preview = PreviewRuntime::new(example, setup, "spawn_pads play", None);
        preview.world.spawn((
            AudioPlayer::<AudioSource>(Handle::default()),
            PlaybackSettings::DESPAWN,
        ));
        let host = app.world_mut().spawn(preview).id();
        app.update();
        let mut sounds = app.world_mut().query::<&yours::PreviewSound>();
        assert_eq!(sounds.iter(app.world()).count(), 1, "plays in the app");
        let runtime = app.world_mut().get_mut::<PreviewRuntime>(host).unwrap();
        let world = &mut runtime.into_inner().world;
        let mut left = world.query::<&AudioPlayer<AudioSource>>();
        assert_eq!(
            left.iter(world).count(),
            0,
            "and leaves the example's world"
        );

        // With sound off, it is dropped.
        app.world()
            .resource::<bevsidian_core::nav::Nav>()
            .sound
            .set(false);
        let mut runtime = app.world_mut().get_mut::<PreviewRuntime>(host).unwrap();
        runtime
            .world
            .spawn(AudioPlayer::<AudioSource>(Handle::default()));
        app.update();
        let mut sounds = app.world_mut().query::<&yours::PreviewSound>();
        assert_eq!(sounds.iter(app.world()).count(), 1, "no new sound");
    }

    #[test]
    fn stepping_runs_one_system_at_a_time() {
        let mut preview = ecs("spawn_player regenerate");
        assert!(preview.cards.is_empty());
        preview.step();
        assert_eq!(preview.last, Some(0), "spawn_player ran");
        assert_eq!(preview.cards.len(), 1);
        assert!(preview.cards[0].rows.contains(&"Health(50)".to_owned()));
        preview.step();
        assert_eq!(preview.last, Some(1), "regenerate ran");
        assert!(preview.cards[0].rows.contains(&"Health(51)".to_owned()));
        // Next frame: spawn_player is skipped, regenerate runs again.
        preview.step();
        assert_eq!(preview.frame, 2);
        assert_eq!(preview.last, Some(1));
        assert!(preview.cards[0].rows.contains(&"Health(52)".to_owned()));
        assert_eq!(preview.cards.len(), 1, "only one player");
        // Back replays to the previous step.
        preview.step_back();
        assert!(preview.cards[0].rows.contains(&"Health(51)".to_owned()));
        assert_eq!(preview.steps, 2);
    }

    #[test]
    fn blocks_run_the_functions_they_name() {
        let preview = ecs("fn spawn_player() {}");
        assert!(!preview.stepping, "startup code alone runs at once");
        assert_eq!(preview.cards.len(), 1);
        assert!(ecs("").cards.is_empty());
    }

    #[test]
    fn empty_previews_know_the_cards_to_come() {
        let preview = ecs("spawn_player regenerate");
        assert!(preview.cards.is_empty());
        assert_eq!(preview.placeholder.len(), 1);
        assert_eq!(preview.frame, 0, "the probe frame is undone");
    }

    #[test]
    fn playing_holds_on_spawns() {
        let mut preview = ecs("spawn_player regenerate");
        assert!((preview.play_step() - runtime::HOLD).abs() < f64::EPSILON);
        assert!(
            (preview.play_step() - 0.25).abs() < 1e-9,
            "regenerate at 4 fps"
        );
        assert!(
            (preview.play_step() - 0.25).abs() < 1e-9,
            "spawn_player is skipped"
        );
    }

    #[test]
    fn observers_run_when_a_system_triggers_them() {
        let mut preview = ecs("spawn_player attack on_damage");
        preview.step_frame();
        assert!(preview.cards[0].rows.contains(&"Health(40)".to_owned()));
        preview.step_frame();
        assert!(preview.cards[0].rows.contains(&"Health(30)".to_owned()));
    }

    #[test]
    fn states_follow_the_buttons() {
        let example = examples::find("states").unwrap();
        let setup = (example.setup)(&World::new());
        let code = "spawn_menu despawn_menu spawn_level despawn_level NextState";
        let mut preview = PreviewRuntime::new(example, setup, code, None);
        let shows = |preview: &PreviewRuntime, row: &str| {
            preview
                .cards
                .iter()
                .any(|c| c.rows.iter().any(|r| r.contains(row)))
        };
        assert!(shows(&preview, "MenuScreen"), "starts in the menu");
        preview.run_action(0); // Play
        assert!(shows(&preview, "Level"));
        assert!(!shows(&preview, "MenuScreen"), "OnExit(Menu) ran");
    }

    #[test]
    fn every_example_builds() {
        for name in examples::NAMES {
            let example = examples::find(name).unwrap();
            let setup = (example.setup)(&World::new());
            let everything: String = example
                .functions
                .iter()
                .map(|f| f.name)
                .collect::<Vec<_>>()
                .join(" ");
            let mut preview = PreviewRuntime::new(example, setup, &everything, None);
            for _ in 0..120 {
                preview.advance(1.0 / 60.0);
            }
        }
    }
}

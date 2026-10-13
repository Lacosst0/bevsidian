//! Bevy-in-Bevy: live previews above the code blocks of a note.
//!
//! A code block opts in with `preview=NAME` in its info string; [`demos`] holds the code that
//! runs for each name.
//!
//! - A **world** preview runs the block's systems in a [`World`] of its own and shows its
//!   entities and their component values as cards. With stepping, Bevy's [`Stepping`] runs one
//!   system at a time: step forward, play / pause, step back (which replays from the start, as
//!   stepping only goes forward), and the code lines of the system that just ran are
//!   highlighted.
//! - A **scene** preview spawns the block's UI scene inside the preview.

mod demos;

use std::{ops::Range, time::Duration};

use bevy::{
    ecs::schedule::{
        NodeId, ScheduleBuildSettings, ScheduleLabel, SingleThreadedExecutor, Stepping, SystemKey,
    },
    prelude::*,
    text::FontWeight,
    ui_widgets::Activate,
};
use bevy_ui_commonmark::{CodeBlockInfo, CodeHighlight, MdCodeGroup};

use bevsidian_core::{
    icons,
    theme::{icon_font, palette, ui_font},
};
use demos::{Demo, WorldDemo};

pub fn plugin(app: &mut App) {
    app.add_observer(start_world_preview)
        .add_systems(Update, (play_previews, show_previews).chain());
}

/// The preview shown above a code block whose info string names a demo (`preview=NAME`).
pub fn code_preview(block: &CodeBlockInfo) -> Option<Box<dyn Scene>> {
    let name = block
        .info
        .split_whitespace()
        .find_map(|word| word.strip_prefix("preview="))?;
    Some(match demos::find(name)? {
        Demo::Scene(scene) => Box::new(scene_frame(scene())),
        Demo::World(demo) => Box::new(world_frame(
            WorldPreview {
                name: name.to_owned(),
                code: block.code.to_owned(),
            },
            demo.stepping,
        )),
    })
}

// ---------------------------------------------------------------------------
// The preview world
// ---------------------------------------------------------------------------

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct DemoSchedule;

/// Simulated time per frame: previews run the same way at any display rate.
const FRAME: Duration = Duration::from_nanos(16_666_667);

/// A world preview to start: its demo name and the code it shows.
#[derive(Component, Clone, Debug, Default)]
struct WorldPreview {
    name: String,
    code: String,
}

/// A running world preview.
#[derive(Component)]
pub struct PreviewRuntime {
    demo: WorldDemo,
    world: World,
    /// System keys in run order (to find the system at the stepping cursor).
    keys: Vec<SystemKey>,
    /// Code lines highlighted when each system runs.
    lines: Vec<Vec<Range<usize>>>,
    /// Frames started.
    frame: u32,
    /// Systems run since the start (skipped `Startup` systems aside).
    steps: usize,
    /// The system that ran last.
    last: Option<usize>,
    playing: bool,
    next_tick: f64,
    /// Entities and their components, for the cards.
    cards: Vec<Card>,
}

struct Card {
    title: String,
    rows: Vec<String>,
    /// A 0–1 bar (shown for `Health`).
    bar: Option<f32>,
}

impl PreviewRuntime {
    /// Whether it is playing (and needs frames).
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    fn new(demo: WorldDemo, code: &str) -> Self {
        let lines = demo
            .systems
            .iter()
            .map(|system| {
                let mut lines: Vec<Range<usize>> = code_lines(code, system).into_iter().collect();
                for (_, extra) in demo.also.iter().filter(|(s, _)| s == system) {
                    lines.extend(code_lines(code, extra));
                }
                lines
            })
            .collect();
        let mut runtime = Self {
            demo,
            world: World::new(),
            keys: Vec::new(),
            lines,
            frame: 0,
            steps: 0,
            last: None,
            playing: false,
            next_tick: 0.0,
            cards: Vec::new(),
        };
        runtime.reset();
        runtime
    }

    /// Builds a fresh world. Without stepping, its first frame runs at once.
    fn reset(&mut self) {
        let mut world = World::new();
        world.insert_resource(AppTypeRegistry::default());
        world.insert_resource(Time::<()>::default());
        let mut schedule = Schedule::new(DemoSchedule);
        schedule.set_executor(SingleThreadedExecutor::new());
        // One system per step: each run applies its own commands at the end.
        schedule.set_build_settings(ScheduleBuildSettings {
            auto_insert_apply_deferred: false,
            ..default()
        });
        (self.demo.build)(&mut world, &mut schedule);
        if let Err(error) = schedule.initialize(&mut world) {
            error!("preview schedule: {error}");
        }
        self.keys = schedule
            .systems()
            .map(|systems| systems.map(|(key, _)| key).collect())
            .unwrap_or_default();
        debug_assert_eq!(self.keys.len(), self.demo.systems.len());
        world.add_schedule(schedule);
        let mut stepping = Stepping::new();
        stepping.add_schedule(DemoSchedule);
        if self.demo.stepping {
            stepping.enable();
        }
        world.insert_resource(stepping);
        self.world = world;
        self.frame = 0;
        self.steps = 0;
        self.last = None;
        if !self.demo.stepping {
            self.frame = 1;
            self.world.resource_mut::<Time>().advance_by(FRAME);
            self.world.run_schedule(DemoSchedule);
        }
        self.snapshot();
    }

    /// Index of the system the stepping cursor points at (the next to run).
    fn next_index(&self) -> usize {
        match self.world.resource::<Stepping>().cursor() {
            Some((_, NodeId::System(key))) => self.keys.iter().position(|k| *k == key),
            _ => None,
        }
        .unwrap_or(0)
    }

    /// Runs the next system, skipping `Startup` systems after the first frame.
    fn step(&mut self) {
        if self.keys.is_empty() {
            return;
        }
        for _ in 0..=self.keys.len() {
            let index = self.next_index();
            if index == 0 {
                self.frame += 1;
                self.world.resource_mut::<Time>().advance_by(FRAME);
            }
            self.world.resource_mut::<Stepping>().step_frame();
            if let Err(error) = self.world.run_system_cached(Stepping::begin_frame) {
                error!("preview stepping: {error}");
                return;
            }
            self.world.run_schedule(DemoSchedule);
            let skipped = self.frame > 1 && self.demo.startup.contains(&self.demo.systems[index]);
            if !skipped {
                self.last = Some(index);
                self.steps += 1;
                break;
            }
        }
        self.snapshot();
    }

    /// Runs the rest of the current frame.
    fn step_frame(&mut self) {
        self.step();
        while self.next_index() != 0 {
            self.step();
        }
    }

    /// Goes back one step: stepping only runs forward, so this replays from the start.
    fn step_back(&mut self) {
        let target = self.steps.saturating_sub(1);
        self.reset();
        while self.steps < target {
            self.step();
        }
    }

    fn control(&mut self, control: Control, now: f64) {
        match control {
            Control::Back => {
                self.playing = false;
                self.step_back();
            }
            Control::Forward => {
                self.playing = false;
                self.step();
            }
            Control::Play => {
                self.playing = !self.playing;
                self.next_tick = now;
            }
            Control::Reset => {
                self.playing = false;
                self.reset();
            }
        }
    }

    /// Reads the entities and component values shown on the cards.
    fn snapshot(&mut self) {
        let registry = self.world.resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        let mut cards = Vec::new();
        for entity in self.world.iter_entities() {
            let mut rows = Vec::new();
            let mut bar = None;
            for registration in registry.iter() {
                let Some(reflect) = registration.data::<ReflectComponent>() else {
                    continue;
                };
                let Some(value) = reflect.reflect(entity) else {
                    continue;
                };
                let table = registration.type_info().type_path_table();
                let name = table.short_path();
                if registration.type_id() == std::any::TypeId::of::<Transform>()
                    && let Some(transform) = entity.get::<Transform>()
                {
                    let t = transform.translation;
                    rows.push(format!("Transform ({:.0}, {:.0}, {:.0})", t.x, t.y, t.z));
                    continue;
                }
                let text = format!("{value:?}").replace(table.path(), name);
                if name == "Health" {
                    bar = text
                        .trim_start_matches("Health(")
                        .trim_end_matches(')')
                        .parse::<f32>()
                        .ok()
                        .map(|hp| (hp / 100.0).clamp(0.0, 1.0));
                }
                rows.push(text);
            }
            if !rows.is_empty() {
                rows.sort();
                cards.push(Card {
                    title: format!("Entity {}", entity.id().index()),
                    rows,
                    bar,
                });
            }
        }
        self.cards = cards;
    }

    fn status(&self) -> String {
        if !self.demo.stepping {
            let count = self.cards.len();
            let plural = if count == 1 { "entity" } else { "entities" };
            return format!("{count} {plural} after one frame");
        }
        match self.last {
            None => "Not started: step or play".to_owned(),
            Some(index) => format!("Frame {} · ran {}", self.frame, self.demo.systems[index]),
        }
    }
}

/// Lines of `fn name` (up to its closing brace), or else the first line naming it.
fn code_lines(code: &str, name: &str) -> Option<Range<usize>> {
    let lines: Vec<&str> = code.lines().collect();
    let names = |line: &str, word: &str| {
        line.match_indices(word).any(|(at, _)| {
            let before = line[..at].chars().next_back();
            let after = line[at + word.len()..].chars().next();
            !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
                && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
        })
    };
    if let Some(start) = lines.iter().position(|l| names(l, &format!("fn {name}"))) {
        let mut depth = 0i32;
        for (i, line) in lines.iter().enumerate().skip(start) {
            for c in line.chars() {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if depth <= 0 && line.contains('}') {
                return Some(start..i + 1);
            }
        }
        return Some(start..lines.len());
    }
    let line = lines.iter().position(|l| names(l, name))?;
    Some(line..line + 1)
}

fn start_world_preview(
    add: On<Add<WorldPreview>>,
    previews: Query<&WorldPreview>,
    mut commands: Commands,
) {
    let Ok(preview) = previews.get(add.entity) else {
        return;
    };
    if let Some(Demo::World(demo)) = demos::find(&preview.name) {
        commands
            .entity(add.entity)
            .insert(PreviewRuntime::new(demo, &preview.code));
    }
}

fn play_previews(time: Res<Time<Real>>, mut previews: Query<&mut PreviewRuntime>) {
    let now = time.elapsed_secs_f64();
    for mut preview in &mut previews {
        if preview.playing && now >= preview.next_tick {
            preview.step_frame();
            let period = 1.0 / f64::from(preview.demo.fps);
            // Catch up at most one frame after a stall.
            preview.next_tick = (preview.next_tick + period).max(now);
        }
    }
}

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------

/// Parts of a world preview that change.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PreviewPart {
    #[default]
    Cards,
    Status,
    PlayIcon,
    /// A control's icon that never changes.
    Icon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Control {
    Back,
    Play,
    Forward,
    Reset,
}

const PREVIEW_BG: Color = Color::srgb(0.09, 0.09, 0.1);

fn frame_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(10.0),
        padding: UiRect::all(Val::Px(12.0)),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        min_width: Val::Px(0.0),
        overflow: Overflow::clip(),
        ..default()
    }
}

fn scene_frame(scene: Box<dyn Scene>) -> impl Scene {
    let node = frame_node();
    // Shows the scene root's layout box, e.g. the width a patch sets.
    let outline = Outline::new(Val::Px(1.0), Val::ZERO, palette::ACCENT.with_alpha(0.6));
    bsn! {
        node
        BackgroundColor(PREVIEW_BG)
        BorderColor::all(palette::BORDER)
        Children [
            Node { padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)), min_height: Val::Px(40.0) }
            Children [ @{scene} outline ]
        ]
    }
}

fn world_frame(preview: WorldPreview, stepping: bool) -> impl Scene {
    let node = frame_node();
    let status = Text::new("");
    let controls: Vec<Box<dyn Scene>> = if stepping {
        [
            (Control::Back, "step-back"),
            (Control::Play, "play"),
            (Control::Forward, "step-forward"),
            (Control::Reset, "rotate-ccw"),
        ]
        .into_iter()
        .map(|(control, icon)| Box::new(control_button(control, icon)) as Box<dyn Scene>)
        .collect()
    } else {
        Vec::new()
    };
    bsn! {
        node
        preview
        BackgroundColor(PREVIEW_BG)
        BorderColor::all(palette::BORDER)
        Children [
            status @{ui_font(11.5, FontWeight::MEDIUM)} TextColor(palette::MUTED) PreviewPart::Status
            --
            // Tall enough for a card, so the controls stay put when the first one appears.
            Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(8.0), row_gap: Val::Px(8.0), min_height: Val::Px(104.0) }
            PreviewPart::Cards
            --
            Node { column_gap: Val::Px(6.0) }
            Children [ {controls} ]
        ]
    }
}

fn control_button(control: Control, icon: &str) -> impl Scene + use<> {
    let glyph = Text::new(icons::find(icon).unwrap_or(icons::DEFAULT).to_string());
    let node = Node {
        width: Val::Px(36.0),
        height: Val::Px(30.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    };
    let part = if control == Control::Play {
        PreviewPart::PlayIcon
    } else {
        PreviewPart::Icon
    };
    bsn! {
        node
        BackgroundColor(palette::BG_HOVER)
        bevy::ui_widgets::Button
        bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
        on(move |activate: On<Activate>,
                 time: Res<Time<Real>>,
                 parents: Query<&ChildOf>,
                 mut previews: Query<&mut PreviewRuntime>| {
            let host = parents
                .iter_ancestors(activate.entity)
                .find(|&e| previews.contains(e));
            if let Some(mut preview) = host.and_then(|e| previews.get_mut(e).ok()) {
                preview.control(control, time.elapsed_secs_f64());
            }
        })
        Children [
            glyph @{icon_font(16.0)} TextColor(palette::TEXT) Pickable::IGNORE part
        ]
    }
}

/// Updates cards, status, the play icon and the code highlight after a preview changes.
fn show_previews(
    previews: Query<(Entity, &PreviewRuntime), Changed<PreviewRuntime>>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    parts: Query<&PreviewPart>,
    groups: Query<(), With<MdCodeGroup>>,
    mut texts: Query<&mut Text>,
    mut highlights: Query<&mut CodeHighlight>,
    mut commands: Commands,
) {
    for (host, preview) in &previews {
        for entity in children.iter_descendants(host) {
            match parts.get(entity) {
                Ok(PreviewPart::Cards) => {
                    commands.entity(entity).despawn_children();
                    spawn_cards(&mut commands, entity, &preview.cards);
                }
                Ok(PreviewPart::Status) => {
                    if let Ok(mut text) = texts.get_mut(entity) {
                        text.0 = preview.status();
                    }
                }
                Ok(PreviewPart::PlayIcon) => {
                    if let Ok(mut text) = texts.get_mut(entity) {
                        let icon = if preview.playing { "pause" } else { "play" };
                        text.0 = icons::find(icon).unwrap_or(icons::DEFAULT).to_string();
                    }
                }
                Ok(PreviewPart::Icon) | Err(_) => {}
            }
        }
        if !preview.demo.stepping {
            continue;
        }
        let lines = preview
            .last
            .map(|index| preview.lines[index].clone())
            .unwrap_or_default();
        let group = parents.iter_ancestors(host).find(|&e| groups.contains(e));
        for entity in group.into_iter().flat_map(|g| children.iter_descendants(g)) {
            if let Ok(mut highlight) = highlights.get_mut(entity) {
                highlight.set_if_neq(CodeHighlight(lines.clone()));
            }
        }
    }
}

fn spawn_cards(commands: &mut Commands, parent: Entity, cards: &[Card]) {
    let font = |size: f32, weight: FontWeight| TextFont {
        font: bevy::text::FontSource::families("Inter, sans-serif"),
        font_size: size.into(),
        weight,
        ..default()
    };
    if cards.is_empty() {
        commands.spawn((
            Text::new("No entities yet"),
            font(13.0, FontWeight::NORMAL),
            TextColor(palette::FAINT),
            ChildOf(parent),
        ));
        return;
    }
    for card in cards {
        let id = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(3.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    min_width: Val::Px(140.0),
                    ..default()
                },
                BackgroundColor(palette::BG_SIDEBAR),
                ChildOf(parent),
            ))
            .id();
        commands.spawn((
            Text::new(card.title.clone()),
            font(11.0, FontWeight::BOLD),
            TextColor(palette::MUTED),
            ChildOf(id),
        ));
        for row in &card.rows {
            commands.spawn((
                Text::new(row.clone()),
                font(13.0, FontWeight::MEDIUM),
                TextColor(palette::TEXT),
                ChildOf(id),
            ));
        }
        if let Some(fill) = card.bar {
            let track = commands
                .spawn((
                    Node {
                        height: Val::Px(6.0),
                        margin: UiRect::top(Val::Px(3.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(palette::BORDER),
                    ChildOf(id),
                ))
                .id();
            commands.spawn((
                Node {
                    width: Val::Percent(fill * 100.0),
                    ..default()
                },
                BackgroundColor(palette::ACCENT),
                ChildOf(track),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = "fn a() {\n    x();\n}\n\nfn main() {\n    run(a);\n    b();\n}";

    #[test]
    fn code_lines_find_functions_and_mentions() {
        assert_eq!(code_lines(CODE, "a"), Some(0..3));
        assert_eq!(code_lines(CODE, "main"), Some(4..8));
        assert_eq!(code_lines(CODE, "b"), Some(6..7));
        assert_eq!(code_lines(CODE, "missing"), None);
    }

    fn world_demo(name: &str) -> WorldDemo {
        match demos::find(name) {
            Some(Demo::World(demo)) => demo,
            _ => panic!("no world demo {name}"),
        }
    }

    #[test]
    fn stepping_runs_one_system_at_a_time() {
        let mut preview = PreviewRuntime::new(world_demo("regenerate"), "");
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
    fn observers_run_when_a_system_triggers_them() {
        let mut preview = PreviewRuntime::new(world_demo("damage"), "");
        preview.step_frame();
        assert!(preview.cards[0].rows.contains(&"Health(40)".to_owned()));
        preview.step_frame();
        assert!(preview.cards[0].rows.contains(&"Health(30)".to_owned()));
    }

    #[test]
    fn without_stepping_one_frame_runs_at_once() {
        let preview = PreviewRuntime::new(world_demo("spawn-player"), "");
        assert_eq!(preview.cards.len(), 1);
        assert!(
            preview.cards[0]
                .rows
                .iter()
                .any(|row| row.starts_with("Transform"))
        );
    }
}

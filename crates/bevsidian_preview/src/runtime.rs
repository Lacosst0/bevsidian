//! A preview's own [`World`]: an example's program with the block's functions, run one system
//! at a time with Bevy's [`Stepping`] or in real time.

use std::{ops::Range, time::Duration};

use bevy::{
    ecs::schedule::{
        NodeId, ScheduleBuildSettings, ScheduleLabel, SingleThreadedExecutor, Stepping, SystemKey,
    },
    prelude::*,
};

use crate::example::{Example, Function, Kind, Setup, Step, View, code_lines};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct PreviewSchedule;

/// Simulated time per stepped frame: stepping runs the same way at any display rate.
const FRAME: Duration = Duration::from_nanos(16_666_667);

/// Longest real-time frame: a stall (or a hidden tab) does not jump the game ahead.
const MAX_FRAME: f64 = 1.0 / 20.0;

/// How long (seconds) playback stays on a step that spawns or despawns entities.
pub const HOLD: f64 = 1.0;

/// A running preview.
#[derive(Component)]
pub struct PreviewRuntime {
    setup: Setup,
    functions: Vec<&'static Function>,
    pub(crate) view: View,
    /// Whether it runs one system at a time (cards) rather than whole frames (games).
    pub(crate) stepping: bool,
    fps: f32,
    pub(crate) world: World,
    /// System keys in run order (to find the system at the stepping cursor).
    keys: Vec<SystemKey>,
    /// The systems, in run order.
    systems: Vec<&'static Function>,
    /// Code lines highlighted when each system runs.
    lines: Vec<Vec<Range<usize>>>,
    /// Frames started.
    pub(crate) frame: u32,
    /// Systems run since the start (skipped `Startup` systems aside).
    pub(crate) steps: usize,
    /// The system that ran last.
    pub(crate) last: Option<usize>,
    pub(crate) playing: bool,
    pub(crate) next_tick: f64,
    /// Entities and their components, for the cards.
    pub(crate) cards: Vec<Card>,
    /// The cards after the first frame: laid out invisibly while there are none, so the
    /// preview keeps its size.
    pub(crate) placeholder: Vec<Card>,
}

#[derive(Clone)]
pub(crate) struct Card {
    pub title: String,
    /// Components as `Debug` text, e.g. `Health(50)`, sorted.
    pub rows: Vec<String>,
}

impl PreviewRuntime {
    /// Whether it is playing (and needs frames).
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// The example's program with the functions `code` names, shown with `code`.
    pub(crate) fn new(example: &Example, setup: Setup, code: &str, fps: Option<f32>) -> Self {
        let functions = example.functions_for(code);
        let systems: Vec<_> = functions
            .iter()
            .copied()
            .filter(|f| f.kind != Kind::Observer)
            .collect();
        let lines = systems
            .iter()
            .map(|system| {
                let mut lines: Vec<Range<usize>> =
                    code_lines(code, system.name).into_iter().collect();
                for (_, extra) in example.also.iter().filter(|(s, _)| *s == system.name) {
                    lines.extend(code_lines(code, extra));
                }
                lines
            })
            .collect();
        let (stepping, default_fps) = match example.view {
            // Startup code alone runs at once: there is nothing to step through.
            View::Cards { fps } => (systems.iter().any(|s| s.kind != Kind::Startup), fps),
            View::Game { .. } => (false, 60.0),
        };
        let mut runtime = Self {
            setup,
            functions,
            view: example.view,
            stepping,
            fps: fps.unwrap_or(default_fps),
            world: World::new(),
            keys: Vec::new(),
            systems,
            lines,
            frame: 0,
            steps: 0,
            last: None,
            playing: false,
            next_tick: 0.0,
            cards: Vec::new(),
            placeholder: Vec::new(),
        };
        runtime.reset();
        if runtime.stepping {
            runtime.step_frame();
            runtime.placeholder = std::mem::take(&mut runtime.cards);
            runtime.reset();
        } else {
            runtime.placeholder = runtime.cards.clone();
        }
        runtime
    }

    /// Builds a fresh world. Without stepping, its first frame runs at once.
    pub(crate) fn reset(&mut self) {
        let mut world = World::new();
        world.insert_resource(AppTypeRegistry::default());
        world.insert_resource(Time::<()>::default());
        (self.setup)(&mut world);
        let mut schedule = Schedule::new(PreviewSchedule);
        schedule.set_executor(SingleThreadedExecutor::new());
        // One system per step: each run applies its own commands at the end.
        schedule.set_build_settings(ScheduleBuildSettings {
            auto_insert_apply_deferred: false,
            ..default()
        });
        for (index, function) in self.functions.iter().enumerate() {
            (function.add)(&mut world, &mut schedule, Step(index));
            if index > 0 {
                schedule.configure_sets(Step(index).after(Step(index - 1)));
            }
        }
        if let Err(error) = schedule.initialize(&mut world) {
            error!("preview schedule: {error}");
        }
        self.keys = schedule
            .systems()
            .map(|systems| systems.map(|(key, _)| key).collect())
            .unwrap_or_default();
        debug_assert_eq!(self.keys.len(), self.systems.len());
        world.add_schedule(schedule);
        let mut stepping = Stepping::new();
        stepping.add_schedule(PreviewSchedule);
        if self.stepping {
            stepping.enable();
        }
        world.insert_resource(stepping);
        self.world = world;
        self.frame = 0;
        self.steps = 0;
        self.last = None;
        if !self.stepping {
            self.advance(FRAME.as_secs_f64());
        }
        self.snapshot();
    }

    /// Runs a whole frame `seconds` long (without stepping).
    pub(crate) fn advance(&mut self, seconds: f64) {
        self.frame += 1;
        self.world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f64(seconds.min(MAX_FRAME)));
        self.world.run_schedule(PreviewSchedule);
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
    pub(crate) fn step(&mut self) {
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
            self.world.run_schedule(PreviewSchedule);
            let skipped = self.frame > 1 && self.systems[index].kind == Kind::Startup;
            if !skipped {
                self.last = Some(index);
                self.steps += 1;
                break;
            }
        }
        self.snapshot();
    }

    /// Runs the rest of the current frame.
    pub(crate) fn step_frame(&mut self) {
        self.step();
        while self.next_index() != 0 {
            self.step();
        }
    }

    /// Runs the next system while playing and returns how long (in seconds) to show it:
    /// a frame's share at the playing rate, or longer when it ran a `Startup` system or
    /// spawned or despawned entities, so those changes can be seen.
    pub(crate) fn play_step(&mut self) -> f64 {
        let titles = |cards: &[Card]| cards.iter().map(|c| c.title.clone()).collect::<Vec<_>>();
        let before = titles(&self.cards);
        self.step();
        let startup = self
            .last
            .is_some_and(|index| self.systems[index].kind == Kind::Startup);
        if startup || titles(&self.cards) != before {
            return HOLD;
        }
        let per_frame = self
            .systems
            .iter()
            .filter(|s| s.kind != Kind::Startup)
            .count();
        1.0 / (f64::from(self.fps) * per_frame.max(1) as f64)
    }

    /// Goes back one step: stepping only runs forward, so this replays from the start.
    pub(crate) fn step_back(&mut self) {
        let target = self.steps.saturating_sub(1);
        self.reset();
        while self.steps < target {
            self.step();
        }
    }

    /// Triggers `event` in the preview world (e.g. pointer input over a game).
    pub(crate) fn trigger<'a, E: Event<Trigger<'a>: Default>>(&mut self, event: E) {
        self.world.trigger(event);
        self.world.flush();
    }

    /// Code lines of the system that ran last.
    pub(crate) fn highlight(&self) -> Vec<Range<usize>> {
        self.last
            .map(|index| self.lines[index].clone())
            .unwrap_or_default()
    }

    /// Reads the entities and component values shown on the cards.
    fn snapshot(&mut self) {
        if !matches!(self.view, View::Cards { .. }) {
            return;
        }
        let registry = self.world.resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        let mut cards = Vec::new();
        for entity in self.world.iter_entities() {
            let mut rows = Vec::new();
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
                rows.push(format!("{value:?}").replace(table.path(), name));
            }
            if !rows.is_empty() {
                rows.sort();
                cards.push(Card {
                    title: format!("Entity {}", entity.id().index()),
                    rows,
                });
            }
        }
        self.cards = cards;
    }
}

//! `Time and timers.md`: a repeating timer that counts ticks, and a stopwatch of frames.

use bevy::prelude::*;

use crate::example::{Example, Log, Setup, View, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: &[
        startup!(spawn_clock),
        update!(tick_clock),
        update!(show_time),
    ],
    tick: None,
    actions: &[],
    also: &[],
    view: View::Cards { fps: 20.0 },
};

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>();
        registry.write().register::<Name>();
        registry.write().register::<Ticks>();
    })
}

#[derive(Component)]
struct Clock(Timer);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Ticks(u32);

fn spawn_clock(mut commands: Commands) {
    commands.spawn((
        Name::new("Clock"),
        Clock(Timer::from_seconds(0.5, TimerMode::Repeating)),
        Ticks(0),
    ));
}

fn tick_clock(time: Res<Time>, mut clocks: Query<(&mut Clock, &mut Ticks)>) {
    for (mut clock, mut ticks) in &mut clocks {
        // `tick` adds this frame's time; `just_finished` is true on the frame it wraps.
        if clock.0.tick(time.delta()).just_finished() {
            ticks.0 += 1;
        }
    }
}

fn show_time(time: Res<Time>, mut log: ResMut<Log>) {
    if ((time.elapsed_secs() * 60.0).round() as u32).is_multiple_of(15) {
        log.say(format!(
            "elapsed {:.2}s, last frame {:.1}ms",
            time.elapsed_secs(),
            time.delta_secs() * 1000.0
        ));
    }
}

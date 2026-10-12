//! Optional frame-rate readout for checking performance on real devices.
//!
//! Open the web build with `#fps` at the end of the URL (or run natively with
//! `BEVSIDIAN_FPS=1`) to show frames per second, the frame interval and the time spent in
//! the app's own schedules. Idle gaps (the app only renders when something happens) are left
//! out of the averages.

use bevy::{platform::time::Instant, prelude::*, text::FontWeight};

use crate::ui::palette;

pub fn plugin(app: &mut App) {
    if !enabled() {
        return;
    }
    app.init_resource::<FrameStats>()
        .add_systems(Startup, spawn_readout)
        .add_systems(First, begin_frame)
        .add_systems(Last, (end_frame, update_readout).chain());
}

fn enabled() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .is_some_and(|hash| hash.contains("fps"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var_os("BEVSIDIAN_FPS").is_some()
    }
}

#[derive(Resource)]
struct FrameStats {
    /// Start of the current update.
    begin: Instant,
    /// Start of the previous update.
    previous: Option<Instant>,
    /// Smoothed interval between consecutive frames (ms).
    interval: f32,
    /// Smoothed time from `First` to `Last` of the main schedule (ms).
    work: f32,
    /// Worst interval since the last readout refresh (ms).
    worst: f32,
    last_refresh: Instant,
}

impl Default for FrameStats {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            begin: now,
            previous: None,
            interval: 0.0,
            work: 0.0,
            worst: 0.0,
            last_refresh: now,
        }
    }
}

/// Root of the readout.
#[derive(Component, Clone, Copy, Debug, Default)]
struct FpsText;

fn spawn_readout(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            bottom: Val::Px(8.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GlobalZIndex(200),
        Pickable::IGNORE,
        children![(
            FpsText,
            Text::new("–"),
            TextFont {
                font: bevy::text::FontSource::families("JetBrains Mono, monospace"),
                font_size: 12.0.into(),
                weight: FontWeight::NORMAL,
                ..default()
            },
            TextColor(palette::TEXT),
            Pickable::IGNORE,
        )],
    ));
}

fn begin_frame(mut stats: ResMut<FrameStats>) {
    let now = Instant::now();
    if let Some(previous) = stats.previous {
        let interval = (now - previous).as_secs_f32() * 1000.0;
        // Longer gaps are idle time in reactive mode, not slow frames.
        if interval < 250.0 {
            stats.interval = if stats.interval == 0.0 {
                interval
            } else {
                stats.interval * 0.9 + interval * 0.1
            };
            stats.worst = stats.worst.max(interval);
        }
    }
    stats.previous = Some(now);
    stats.begin = now;
}

fn end_frame(mut stats: ResMut<FrameStats>) {
    let work = stats.begin.elapsed().as_secs_f32() * 1000.0;
    stats.work = stats.work * 0.9 + work * 0.1;
}

fn update_readout(mut stats: ResMut<FrameStats>, mut text: Single<&mut Text, With<FpsText>>) {
    if stats.last_refresh.elapsed().as_secs_f32() < 0.5 || stats.interval == 0.0 {
        return;
    }
    stats.last_refresh = Instant::now();
    text.0 = format!(
        "{:.0} fps  {:.1} ms (worst {:.0})  main {:.1} ms",
        1000.0 / stats.interval,
        stats.interval,
        stats.worst,
        stats.work,
    );
    stats.worst = 0.0;
}

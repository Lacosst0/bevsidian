//! Frame-rate readout and `VSync` setting.
//!
//! The readout is turned on in Settings ("Show FPS"), by opening the web build with `#fps`
//! at the end of the URL, or natively with `BEVSIDIAN_FPS=1`. It shows:
//! - `fps` and the frame interval (idle gaps are left out: the app only renders when
//!   something happens),
//! - `cpu`: time from the start of a frame's update to the end of its rendering, which is
//!   the work a frame costs,
//! - on the web, `display`: how often the browser offers frames to this page
//!   (`requestAnimationFrame`). The app cannot draw faster than that; if `fps` matches it
//!   and `cpu` is well below the interval, the frame rate is limited by the browser.

use std::{
    fmt::Write,
    sync::{Arc, Mutex},
};

use bevy::{
    platform::time::Instant,
    prelude::*,
    render::{Render, RenderApp, RenderSystems},
    text::FontWeight,
    window::{PresentMode, PrimaryWindow},
};

use crate::{nav::Nav, ui::palette};

pub fn plugin(app: &mut App) {
    let render_end = RenderEnd::default();
    app.init_resource::<FrameStats>()
        .insert_resource(render_end.clone())
        .add_systems(Startup, spawn_readout)
        .add_systems(First, begin_frame)
        .add_systems(Last, update_readout)
        .add_systems(Update, apply_vsync);
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .insert_resource(render_end)
            .add_systems(Render, mark_render_end.after(RenderSystems::Cleanup));
    }
}

/// Whether the readout was asked for in the URL (`#fps`) or environment.
pub fn requested() -> bool {
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

/// When the render world finished the last frame (shared by both worlds).
#[derive(Resource, Clone, Default)]
struct RenderEnd(Arc<Mutex<Option<Instant>>>);

fn mark_render_end(end: Res<RenderEnd>) {
    if let Ok(mut end) = end.0.lock() {
        *end = Some(Instant::now());
    }
}

#[derive(Resource)]
struct FrameStats {
    /// Start of the previous update.
    previous: Option<Instant>,
    /// Smoothed interval between consecutive frames (ms).
    interval: f32,
    /// Smoothed CPU time of a whole frame, update and render (ms).
    cpu: f32,
    /// Worst interval since the last readout refresh (ms).
    worst: f32,
    last_refresh: Instant,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            previous: None,
            interval: 0.0,
            cpu: 0.0,
            worst: 0.0,
            last_refresh: Instant::now(),
        }
    }
}

fn smooth(average: f32, sample: f32) -> f32 {
    if average == 0.0 {
        sample
    } else {
        average * 0.9 + sample * 0.1
    }
}

fn begin_frame(mut stats: ResMut<FrameStats>, render_end: Res<RenderEnd>) {
    let now = Instant::now();
    if let Some(previous) = stats.previous {
        let interval = (now - previous).as_secs_f32() * 1000.0;
        // Longer gaps are idle time in reactive mode, not slow frames.
        if interval < 250.0 {
            stats.interval = smooth(stats.interval, interval);
            stats.worst = stats.worst.max(interval);
        }
        let end = render_end.0.lock().ok().and_then(|mut end| end.take());
        if let Some(end) = end
            && end > previous
        {
            stats.cpu = smooth(stats.cpu, (end - previous).as_secs_f32() * 1000.0);
        }
    }
    stats.previous = Some(now);
}

/// The readout's text.
#[derive(Component, Clone, Copy, Debug, Default)]
struct FpsText;

/// Root of the readout.
#[derive(Component, Clone, Copy, Debug, Default)]
struct FpsReadout;

fn spawn_readout(mut commands: Commands) {
    commands.spawn((
        FpsReadout,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            // Bottom left: the graph's zoom buttons sit bottom right.
            left: Val::Px(8.0),
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

fn update_readout(
    nav: Res<Nav>,
    mut stats: ResMut<FrameStats>,
    mut root: Single<&mut Node, With<FpsReadout>>,
    mut text: Single<&mut Text, With<FpsText>>,
    mut probing: Local<bool>,
) {
    let shown = nav.show_fps.get_untracked();
    let display = if shown { Display::Flex } else { Display::None };
    if root.display != display {
        root.display = display;
    }
    if *probing != shown {
        *probing = shown;
        display_probe(shown);
    }
    if !shown || stats.last_refresh.elapsed().as_secs_f32() < 0.5 || stats.interval == 0.0 {
        return;
    }
    stats.last_refresh = Instant::now();
    let mut line = format!(
        "{:.0} fps  {:.1} ms (worst {:.0})  cpu {:.1} ms",
        1000.0 / stats.interval,
        stats.interval,
        stats.worst,
        stats.cpu,
    );
    if let Some(hz) = display_hz() {
        let _ = write!(line, "  display {hz:.0} Hz");
    }
    text.0 = line;
    stats.worst = 0.0;
}

/// Starts or stops a tiny `requestAnimationFrame` loop that measures how often the browser
/// offers frames to the page. It only runs while the readout is shown.
fn display_probe(on: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        let script = if on {
            "(() => { if (window.__bvsProbe) return; window.__bvsProbe = true; \
             let last = 0, n = 0, acc = 0; \
             const tick = (t) => { if (!window.__bvsProbe) return; \
               if (last) { acc += t - last; n++; \
                 if (acc >= 500) { window.__bvsHz = 1000 * n / acc; n = 0; acc = 0; } } \
               last = t; requestAnimationFrame(tick); }; \
             requestAnimationFrame(tick); })()"
        } else {
            "window.__bvsProbe = false"
        };
        let _ = js_sys::eval(script);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = on;
    }
}

fn display_hz() -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        js_sys::Reflect::get(&window, &"__bvsHz".into())
            .ok()?
            .as_f64()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Applies the `VSync` setting (desktop only: browsers always present in sync with the
/// display).
fn apply_vsync(nav: Res<Nav>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mode = if nav.vsync.get_untracked() {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    for mut window in &mut windows {
        if window.present_mode != mode {
            window.present_mode = mode;
        }
    }
}

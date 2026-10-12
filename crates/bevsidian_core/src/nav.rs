//! Navigation state (current note, history, view, filters) and the systems that drive it.

use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    ui_widgets::{ScrollArea, ScrollIntoView},
};
use bevy_radioactive::coat::{Signal, signal};
use bevy_ui_commonmark::{LinkTarget, MdFootnote, MdHeading, MdLinkActivated};
use bevy_ui_textselect::{Modifiers, TextSelection};

use crate::vault::{NoteId, Vault};

pub fn plugin(app: &mut App) {
    app.add_observer(on_link).add_systems(
        Update,
        (
            shortcuts,
            bind_search_inputs,
            reset_scroll_on_navigation,
            scroll_to_anchor,
        ),
    );
}

/// Main content view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    /// Reading view of the current note.
    #[default]
    Note,
    /// Graph of all notes.
    Graph,
}

/// Reactive navigation state, shared by the UI (through `coat!` signals) and systems.
#[derive(Resource, Clone)]
pub struct Nav {
    pub current: Signal<NoteId>,
    pub view: Signal<View>,
    pub query: Signal<String>,
    pub tag: Signal<Option<String>>,
    pub back: Signal<Vec<NoteId>>,
    pub forward: Signal<Vec<NoteId>>,
    /// Heading slug or footnote label to scroll to once the note is rendered.
    pub anchor: Signal<Option<Anchor>>,
    /// Quick switcher state.
    pub switcher: Signal<bool>,
    pub switcher_query: Signal<String>,
    pub switcher_index: Signal<usize>,
    /// Window width class.
    pub mode: Signal<LayoutMode>,
    /// Files drawer open (phones).
    pub left_open: Signal<bool>,
    /// Outline/backlinks drawer open (tablets and phones).
    pub right_open: Signal<bool>,
    /// The last input was a touch.
    pub touch: Signal<bool>,
    /// Some text is selected.
    pub has_selection: Signal<bool>,
    /// The on-screen keyboard is wanted.
    pub keyboard: Signal<bool>,
    /// Reading text size in logical px.
    pub text_size: Signal<u32>,
    /// Settings panel open.
    pub settings: Signal<bool>,
    /// Scroll speed in percent of the default.
    pub scroll_speed: Signal<u32>,
    /// Show the frame-rate readout.
    pub show_fps: Signal<bool>,
    /// Frame-rate cap while animating; 0 = as fast as the display allows.
    pub max_fps: Signal<u32>,
    /// Wait for the display's refresh (desktop only; browsers always do).
    pub vsync: Signal<bool>,
    /// Refresh rate the browser offers the page (measured while Settings or the FPS readout
    /// is open; web only).
    pub display_hz: Signal<Option<u32>>,
}

/// Choices for [`Nav::max_fps`], in the order the setting cycles through them.
pub const MAX_FPS_CHOICES: [u32; 6] = [30, 60, 90, 120, 144, 0];

/// Smallest and largest reading text size.
pub const TEXT_SIZES: std::ops::RangeInclusive<u32> = 13..=24;

/// Slowest and fastest scroll speed (percent), and the step between them.
pub const SCROLL_SPEEDS: std::ops::RangeInclusive<u32> = 50..=300;
pub const SCROLL_SPEED_STEP: i32 = 25;

/// Wheel and fling distance at 100 % scroll speed, relative to the platform default.
pub const BASE_SCROLL_FACTOR: f32 = 1.15;

/// A scroll target inside the current note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Anchor {
    Heading(String),
    Footnote(String),
}

impl Nav {
    pub fn new(home: NoteId) -> Self {
        Self {
            current: signal(|| home),
            view: signal(View::default),
            query: signal(String::new),
            tag: signal(|| None),
            back: signal(Vec::new),
            forward: signal(Vec::new),
            anchor: signal(|| None),
            switcher: signal(|| false),
            switcher_query: signal(String::new),
            switcher_index: signal(|| 0),
            mode: signal(LayoutMode::default),
            left_open: signal(|| false),
            right_open: signal(|| false),
            touch: signal(|| false),
            has_selection: signal(|| false),
            keyboard: signal(|| false),
            text_size: signal(|| 16),
            settings: signal(|| false),
            scroll_speed: signal(|| 100),
            show_fps: signal(|| false),
            max_fps: signal(|| 0),
            vsync: signal(|| true),
            display_hz: signal(|| None),
        }
    }

    /// Opens a note, recording history.
    pub fn open(&self, id: NoteId, heading: Option<String>) {
        let current = self.current.get_untracked();
        if current != id {
            self.back.update(|b| b.push(current));
            self.forward.update(Vec::clear);
            self.current.set(id);
        }
        self.view.set_if_neq(View::Note);
        self.close_drawers();
        self.anchor
            .set(heading.map(|h| Anchor::Heading(bevy_ui_commonmark::slugify(&h))));
    }

    pub fn go_back(&self) {
        if let Some(prev) = self.back.update(Vec::pop) {
            let current = self.current.get_untracked();
            self.forward.update(|f| f.push(current));
            self.current.set(prev);
            self.view.set_if_neq(View::Note);
            self.close_drawers();
        }
    }

    pub fn go_forward(&self) {
        if let Some(next) = self.forward.update(Vec::pop) {
            let current = self.current.get_untracked();
            self.back.update(|b| b.push(current));
            self.current.set(next);
            self.view.set_if_neq(View::Note);
            self.close_drawers();
        }
    }

    pub fn close_drawers(&self) {
        self.left_open.set_if_neq(false);
        self.right_open.set_if_neq(false);
    }

    /// Opens one drawer, closing the other.
    pub fn toggle_drawer(&self, left: bool) {
        let (this, other) = if left {
            (&self.left_open, &self.right_open)
        } else {
            (&self.right_open, &self.left_open)
        };
        other.set_if_neq(false);
        this.set(!this.get_untracked());
    }

    /// Changes the reading text size by `delta` px.
    pub fn change_text_size(&self, delta: i32) {
        let next = self
            .text_size
            .get_untracked()
            .saturating_add_signed(delta)
            .clamp(*TEXT_SIZES.start(), *TEXT_SIZES.end());
        self.text_size.set_if_neq(next);
    }

    /// Changes the scroll speed by `steps` steps.
    pub fn change_scroll_speed(&self, steps: i32) {
        let next = self
            .scroll_speed
            .get_untracked()
            .saturating_add_signed(steps * SCROLL_SPEED_STEP)
            .clamp(*SCROLL_SPEEDS.start(), *SCROLL_SPEEDS.end());
        self.scroll_speed.set_if_neq(next);
    }

    /// Moves the frame-rate cap one choice up or down.
    pub fn change_max_fps(&self, steps: i32) {
        let current = self.max_fps.get_untracked();
        let index = MAX_FPS_CHOICES
            .iter()
            .position(|c| *c == current)
            .unwrap_or(MAX_FPS_CHOICES.len() - 1);
        let next = index
            .saturating_add_signed(steps as isize)
            .min(MAX_FPS_CHOICES.len() - 1);
        self.max_fps.set_if_neq(MAX_FPS_CHOICES[next]);
    }

    /// Multiplier for wheel deltas and touch flings.
    pub fn scroll_factor(&self) -> f32 {
        BASE_SCROLL_FACTOR * self.scroll_speed.get_untracked() as f32 / 100.0
    }

    pub fn show(&self, view: View) {
        self.view.set_if_neq(view);
    }

    pub fn clear_tag(&self) {
        self.tag.set(None);
    }

    pub fn toggle_graph(&self) {
        let next = match self.view.get_untracked() {
            View::Note => View::Graph,
            View::Graph => View::Note,
        };
        self.view.set(next);
    }

    /// Toggles a tag filter on the file list.
    pub fn toggle_tag(&self, tag: &str) {
        let next = if self.tag.get_untracked().as_deref() == Some(tag) {
            None
        } else {
            Some(tag.to_owned())
        };
        let filtering = next.is_some();
        self.tag.set(next);
        // On phones the file list lives in a drawer: show the filtered list.
        if filtering && self.mode.get_untracked().left_drawer() {
            self.right_open.set_if_neq(false);
            self.left_open.set_if_neq(true);
        }
    }

    pub fn open_switcher(&self) {
        self.switcher_query.set(String::new());
        self.switcher_index.set(0);
        self.switcher.set(true);
    }
}

fn on_link(
    link: On<MdLinkActivated>,
    nav: Res<Nav>,
    vault: Res<Vault>,
    time: Res<Time<Real>>,
    mut touch: ResMut<bevy_ui_touchscroll::TouchScroll>,
) {
    // The tap only stopped a fling.
    if touch.take_caught(time.elapsed_secs_f64()) {
        return;
    }
    match &link.target {
        LinkTarget::Wiki(wiki) => {
            if let Some(id) = vault.resolve(wiki) {
                nav.open(id, wiki.heading.clone());
            } else if wiki.target.is_empty()
                && let Some(heading) = &wiki.heading
            {
                // `[[#Heading]]` links inside the current note.
                nav.anchor
                    .set(Some(Anchor::Heading(bevy_ui_commonmark::slugify(heading))));
            } else {
                warn!("no note named {:?}", wiki.target);
            }
        }
        LinkTarget::Tag(tag) => nav.toggle_tag(tag),
        LinkTarget::Footnote(label) => nav.anchor.set(Some(Anchor::Footnote(label.clone()))),
        LinkTarget::Url(url) => open_url(url),
    }
}

/// Opens an external URL in a new browser tab (web) or logs it (native).
pub fn open_url(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        info!("open {url} in your browser");
    }
}

/// Marks a text input whose value is mirrored into a [`Nav`] string signal.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchInput {
    #[default]
    Sidebar,
    Switcher,
}

fn bind_search_inputs(
    inputs: Query<(&EditableText, &SearchInput), Changed<EditableText>>,
    nav: Res<Nav>,
) {
    for (text, kind) in &inputs {
        let value = text.value().to_string();
        match kind {
            SearchInput::Sidebar => {
                nav.query.set_if_neq(value);
            }
            SearchInput::Switcher => {
                if nav.switcher_query.set_if_neq(value) {
                    nav.switcher_index.set(0);
                }
            }
        }
    }
}

fn shortcuts(
    mut keys_in: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    nav: Res<Nav>,
    vault: Res<Vault>,
    mut focus: ResMut<InputFocus>,
    mut selection: ResMut<TextSelection>,
) {
    let mut modifiers = Modifiers::before_frame(&keys);
    if mouse.just_pressed(MouseButton::Back) {
        nav.go_back();
    }
    if mouse.just_pressed(MouseButton::Forward) {
        nav.go_forward();
    }
    for input in keys_in.read() {
        modifiers.update(input);
        if input.state != ButtonState::Pressed {
            continue;
        }
        let command = modifiers.ctrl || modifiers.super_key;
        let alt = modifiers.alt;
        match input.key_code {
            KeyCode::KeyO if command => nav.open_switcher(),
            KeyCode::KeyG if command => nav.toggle_graph(),
            KeyCode::Comma if command => nav.settings.set(!nav.settings.get_untracked()),
            KeyCode::Equal | KeyCode::NumpadAdd if command => nav.change_text_size(1),
            KeyCode::Minus | KeyCode::NumpadSubtract if command => nav.change_text_size(-1),
            KeyCode::ArrowLeft if alt => nav.go_back(),
            KeyCode::ArrowRight if alt => nav.go_forward(),
            KeyCode::Escape => {
                if nav.switcher.get_untracked() {
                    nav.switcher.set(false);
                    focus.clear();
                } else if nav.settings.get_untracked() {
                    nav.settings.set(false);
                } else if nav.left_open.get_untracked() || nav.right_open.get_untracked() {
                    nav.close_drawers();
                } else {
                    selection.clear();
                }
            }
            KeyCode::ArrowDown | KeyCode::ArrowUp if nav.switcher.get_untracked() => {
                let count = vault
                    .search(&nav.switcher_query.get_untracked(), None)
                    .len()
                    .max(1);
                let index = nav.switcher_index.get_untracked();
                let next = if input.key_code == KeyCode::ArrowDown {
                    (index + 1) % count
                } else {
                    (index + count - 1) % count
                };
                nav.switcher_index.set(next);
            }
            KeyCode::Enter | KeyCode::NumpadEnter if nav.switcher.get_untracked() => {
                let results = vault.search(&nav.switcher_query.get_untracked(), None);
                if let Some(id) = results.get(nav.switcher_index.get_untracked()) {
                    nav.open(*id, None);
                }
                nav.switcher.set(false);
                focus.clear();
            }
            _ => {}
        }
    }
}

/// The scrollable container of the reading view.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(ScrollArea)]
pub struct NotePane;

fn reset_scroll_on_navigation(
    nav: Res<Nav>,
    mut last: Local<Option<NoteId>>,
    mut panes: Query<
        (
            &mut ScrollPosition,
            Option<&mut bevy_ui_touchscroll::SmoothScroll>,
        ),
        With<NotePane>,
    >,
) {
    let current = nav.current.get_untracked();
    if *last != Some(current) {
        *last = Some(current);
        for (mut scroll, smooth) in &mut panes {
            *scroll = ScrollPosition::default();
            // A new note starts at the top: jump, don't scroll there.
            if let Some(mut smooth) = smooth {
                smooth.snap(Vec2::ZERO);
            }
        }
    }
}

fn scroll_to_anchor(
    nav: Res<Nav>,
    headings: Query<(Entity, &MdHeading)>,
    footnotes: Query<(Entity, &MdFootnote)>,
    mut commands: Commands,
    mut wait: Local<u8>,
) {
    let Some(anchor) = nav.anchor.get_untracked() else {
        *wait = 0;
        return;
    };
    let target = match &anchor {
        Anchor::Heading(slug) => headings
            .iter()
            .find(|(_, h)| &h.slug == slug)
            .map(|(e, _)| e),
        Anchor::Footnote(label) => footnotes
            .iter()
            .find(|(_, f)| &f.0 == label)
            .map(|(e, _)| e),
    };
    // Give the freshly spawned note a couple of frames to lay out.
    *wait += 1;
    if let Some(entity) = target
        && *wait > 2
    {
        commands.trigger(ScrollIntoView { entity });
        nav.anchor.set(None);
    } else if *wait > 30 {
        nav.anchor.set(None);
    }
}

/// Width class of the window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutMode {
    /// Both sidebars inline (≥ 1100 px).
    #[default]
    Wide,
    /// Files inline, outline/backlinks as a drawer (≥ 760 px).
    Medium,
    /// Phone: both sidebars are drawers.
    Compact,
}

impl LayoutMode {
    pub fn from_width(width: f32) -> Self {
        if width >= 1100.0 {
            Self::Wide
        } else if width >= 760.0 {
            Self::Medium
        } else {
            Self::Compact
        }
    }

    /// Whether the files sidebar is a drawer.
    pub fn left_drawer(self) -> bool {
        self == Self::Compact
    }

    /// Whether the outline/backlinks sidebar is a drawer.
    pub fn right_drawer(self) -> bool {
        self != Self::Wide
    }
}

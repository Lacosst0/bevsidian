//! Navigation state (current note, history, view, filters) and the systems that drive it.

use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::InputFocus,
    prelude::*,
    ui::UiGlobalTransform,
    ui_widgets::{ScrollArea, ScrollIntoView},
};
use bevy_ui_commonmark::{LinkTarget, MdFootnote, MdHeading, MdLinkActivated};
use bevy_ui_textselect::{Modifiers, TextSelection};

use crate::vault::{NoteId, Vault, VaultData};

pub fn plugin(app: &mut App) {
    app.add_observer(on_link).add_systems(
        Update,
        (
            shortcuts,
            reset_switcher_index,
            follow_paths,
            mark_visited,
            reset_scroll_on_navigation,
            scroll_to_anchor,
            track_section.after(reset_scroll_on_navigation),
            track_scrolled.after(reset_scroll_on_navigation),
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

/// What "Make it yours" edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Studio {
    /// A 16 × 16 pixel sprite, drawn in the 2D examples.
    Sprite,
    /// An 8 × 8 × 8 voxel model.
    Voxels,
    /// Sound effects: what a hit, a bounce, a pop and a coin sound like.
    Sounds,
}

/// Navigation state, shared by systems and the UI: views watch it (`Watch<Res<Nav>>`) and
/// handlers change it through `ResMut<Nav>`. Systems that run every frame only write a field
/// when its value changes, so watchers wake only on real changes.
#[derive(Resource, Clone, Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent switches of the interface, each watched on its own"
)]
pub struct Nav {
    pub current: NoteId,
    pub view: View,
    pub query: String,
    pub tag: Option<String>,
    pub back: Vec<NoteId>,
    pub forward: Vec<NoteId>,
    /// Heading slug or footnote label to scroll to once the note is rendered.
    pub anchor: Option<Anchor>,
    /// Slug of the heading whose section is at the top of the reading pane: where the reader
    /// is in the current note (`None` above the first heading, and in the graph).
    pub section: Option<String>,
    /// The reading pane is scrolled far enough down to offer "back to top".
    pub scrolled: bool,
    /// The current note's links popover is open.
    pub links: bool,
    /// The reading path being followed (its note): the map lights its line and numbers its
    /// steps, and its notes link to the previous and next step.
    pub path: Option<NoteId>,
    /// The routes chosen in the map's checklist (their path notes), beside the main quest:
    /// the map shows only their lines. `None` until the reader has chosen.
    pub quests: Option<Vec<NoteId>>,
    /// The map shows every note and line, whatever routes are chosen.
    pub whole_map: bool,
    /// The map's checklist ("What's in your game?") is open.
    pub map_setup: bool,
    /// The map is being edited: notes and objects can be moved, reshaped, added and removed.
    pub map_edit: bool,
    /// "Make it yours" is open, on its pixel sprite or voxel model.
    pub studio: Option<Studio>,
    /// The examples play sounds.
    pub sound: bool,
    /// Notes the reader has opened, in the order first opened (the map lights their stars).
    pub visited: Vec<NoteId>,
    /// Find in the note (`Ctrl+F`): open, what to find, which match is current and how
    /// many there are.
    pub find: bool,
    pub find_query: String,
    pub find_index: usize,
    pub find_count: usize,
    /// Quick switcher state.
    pub switcher: bool,
    pub switcher_query: String,
    pub switcher_index: usize,
    /// Window width class.
    pub mode: LayoutMode,
    /// The sidebar drawer is open (phones).
    pub drawer: bool,
    /// The inline sidebar is collapsed (tablets and desktops).
    pub sidebar_collapsed: bool,
    /// The last input was a touch.
    pub touch: bool,
    /// Some text is selected.
    pub has_selection: bool,
    /// The on-screen keyboard is wanted.
    pub keyboard: bool,
    /// Reading text size in logical px.
    pub text_size: u32,
    /// Settings panel open.
    pub settings: bool,
    /// Scroll speed in percent of the default.
    pub scroll_speed: u32,
    /// Show the frame-rate readout.
    pub show_fps: bool,
    /// Frame-rate cap while animating; 0 = as fast as the display allows.
    pub max_fps: u32,
    /// Wait for the display's refresh (desktop only; browsers always do).
    pub vsync: bool,
    /// Refresh rate the browser offers the page (measured while Settings or the FPS readout
    /// is open; web only).
    pub display_hz: Option<u32>,
    /// The current note is open in the editor.
    pub editing: bool,
}

/// How many notes the quick switcher lists.
pub const SWITCHER_RESULTS: usize = 12;

/// How many recently visited notes the quick switcher lists first.
pub const SWITCHER_RECENT: usize = 5;

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
            current: home,
            view: View::default(),
            query: String::new(),
            tag: None,
            back: Vec::new(),
            forward: Vec::new(),
            anchor: None,
            section: None,
            scrolled: false,
            links: false,
            path: None,
            quests: None,
            whole_map: false,
            map_setup: false,
            map_edit: false,
            studio: None,
            sound: true,
            visited: Vec::new(),
            find: false,
            find_query: String::new(),
            find_index: 0,
            find_count: 0,
            switcher: false,
            switcher_query: String::new(),
            switcher_index: 0,
            mode: LayoutMode::default(),
            drawer: false,
            sidebar_collapsed: false,
            touch: false,
            has_selection: false,
            keyboard: false,
            text_size: 16,
            settings: false,
            scroll_speed: 100,
            show_fps: false,
            max_fps: 0,
            vsync: true,
            display_hz: None,
            editing: false,
        }
    }

    /// The quick switcher's results: notes matching the query, or with no query, the notes
    /// visited last (most recent first, not the open one), then the others. Returns them with
    /// how many lead as recent.
    pub fn switcher_results(&self, vault: &VaultData) -> (Vec<NoteId>, usize) {
        let mut results = vault.search(&self.switcher_query, None);
        let mut recent = 0;
        if self.switcher_query.trim().is_empty() {
            let mut visited: Vec<NoteId> = Vec::new();
            for &id in self.back.iter().rev() {
                if id != self.current && !visited.contains(&id) && visited.len() < SWITCHER_RECENT {
                    visited.push(id);
                }
            }
            recent = visited.len();
            results.retain(|id| !visited.contains(id));
            visited.extend(results);
            results = visited;
        }
        results.truncate(SWITCHER_RESULTS);
        (results, recent)
    }

    /// The highlighted switcher row: the keyboard can point past a list that shrank as the
    /// query grew, and then the last one is meant.
    pub fn switcher_choice(&self, count: usize) -> usize {
        self.switcher_index.min(count.saturating_sub(1))
    }

    /// Opens a note, recording history.
    pub fn open(&mut self, id: NoteId, heading: Option<String>) {
        if self.current != id {
            self.back.push(self.current);
            self.forward.clear();
            self.current = id;
        }
        self.view = View::Note;
        self.close_overlays();
        self.anchor = heading.map(|h| Anchor::Heading(bevy_ui_commonmark::slugify(&h)));
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.back.pop() {
            self.forward.push(self.current);
            self.current = prev;
            self.view = View::Note;
            self.close_overlays();
        }
    }

    pub fn go_forward(&mut self) {
        if let Some(next) = self.forward.pop() {
            self.back.push(self.current);
            self.current = next;
            self.view = View::Note;
            self.close_overlays();
        }
    }

    /// Opens find in the note (in the reading view).
    pub fn open_find(&mut self) {
        if self.view == View::Note && !self.editing {
            self.find = true;
        }
    }

    /// Moves to the next (`1`) or previous (`-1`) match of find, wrapping around.
    pub fn step_find(&mut self, by: isize) {
        let count = self.find_count;
        if count > 0 {
            let index = self.find_index.min(count - 1);
            self.find_index = (index.cast_signed() + by)
                .rem_euclid(count.cast_signed())
                .cast_unsigned();
        }
    }

    /// Closes the drawer and the links popover (after navigating).
    pub fn close_overlays(&mut self) {
        self.drawer = false;
        self.links = false;
    }

    /// Shows or hides the sidebar: the drawer on phones, the inline column elsewhere.
    pub fn toggle_sidebar(&mut self) {
        if self.mode.drawer() {
            self.drawer = !self.drawer;
        } else {
            self.sidebar_collapsed = !self.sidebar_collapsed;
        }
    }

    /// Changes the reading text size by `delta` px.
    pub fn change_text_size(&mut self, delta: i32) {
        self.text_size = self
            .text_size
            .saturating_add_signed(delta)
            .clamp(*TEXT_SIZES.start(), *TEXT_SIZES.end());
    }

    /// Changes the scroll speed by `steps` steps.
    pub fn change_scroll_speed(&mut self, steps: i32) {
        self.scroll_speed = self
            .scroll_speed
            .saturating_add_signed(steps * SCROLL_SPEED_STEP)
            .clamp(*SCROLL_SPEEDS.start(), *SCROLL_SPEEDS.end());
    }

    /// Moves the frame-rate cap one choice up or down.
    pub fn change_max_fps(&mut self, steps: i32) {
        let index = MAX_FPS_CHOICES
            .iter()
            .position(|c| *c == self.max_fps)
            .unwrap_or(MAX_FPS_CHOICES.len() - 1);
        let next = index
            .saturating_add_signed(steps as isize)
            .min(MAX_FPS_CHOICES.len() - 1);
        self.max_fps = MAX_FPS_CHOICES[next];
    }

    /// Multiplier for wheel deltas and touch flings.
    pub fn scroll_factor(&self) -> f32 {
        BASE_SCROLL_FACTOR * self.scroll_speed as f32 / 100.0
    }

    /// Opens or closes the editor for the current note (switching to the note view).
    pub fn toggle_editing(&mut self) {
        let editing = self.view == View::Note && self.editing;
        self.view = View::Note;
        self.editing = !editing;
    }

    pub fn toggle_graph(&mut self) {
        self.view = match self.view {
            View::Note => View::Graph,
            View::Graph => View::Note,
        };
    }

    /// Toggles a tag filter on the file list.
    pub fn toggle_tag(&mut self, tag: &str) {
        self.tag = if self.tag.as_deref() == Some(tag) {
            None
        } else {
            Some(tag.to_owned())
        };
        // On phones the note list lives in a drawer: show the filtered list.
        if self.tag.is_some() && self.mode.drawer() {
            self.drawer = true;
        }
    }

    pub fn open_switcher(&mut self) {
        self.switcher_query.clear();
        self.switcher_index = 0;
        self.switcher = true;
    }
}

fn on_link(
    link: On<MdLinkActivated>,
    mut nav: ResMut<Nav>,
    vault: Res<Vault>,
    touch: Res<bevy_ui_touchscroll::TouchScroll>,
) {
    // The touch scrolled or stopped a fling: not a tap.
    if touch.gesture_scrolled() {
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
                nav.anchor = Some(Anchor::Heading(bevy_ui_commonmark::slugify(heading)));
            } else {
                warn!("no note named {:?}", wiki.target);
            }
        }
        LinkTarget::Tag(tag) => nav.toggle_tag(tag),
        LinkTarget::Footnote(label) => nav.anchor = Some(Anchor::Footnote(label.clone())),
        LinkTarget::Url(url) => open_url(url),
        LinkTarget::Docs(path) => open_url(&crate::docs::url(path)),
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

/// Opening a path's note starts following that path.
fn follow_paths(mut nav: ResMut<Nav>, vault: Res<Vault>, mut last: Local<Option<NoteId>>) {
    let current = nav.current;
    if *last == Some(current) {
        return;
    }
    *last = Some(current);
    if vault.path(current).is_some() && nav.path != Some(current) {
        nav.path = Some(current);
    }
}

/// The note being read counts as visited.
fn mark_visited(mut nav: ResMut<Nav>) {
    let current = nav.current;
    if !nav.visited.contains(&current) {
        nav.visited.push(current);
    }
}

/// Typing in the quick switcher starts again from its first result.
fn reset_switcher_index(mut nav: ResMut<Nav>, mut last: Local<String>) {
    if *last != nav.switcher_query {
        last.clone_from(&nav.switcher_query);
        if nav.switcher_index != 0 {
            nav.switcher_index = 0;
        }
    }
}

fn shortcuts(
    mut keys_in: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut nav: ResMut<Nav>,
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
        let step = if modifiers.shift { -1 } else { 1 };
        match input.key_code {
            KeyCode::KeyO if command => nav.open_switcher(),
            KeyCode::KeyF if command => nav.open_find(),
            KeyCode::F3 if nav.find => nav.step_find(step),
            KeyCode::KeyG if command => nav.toggle_graph(),
            KeyCode::KeyE if command => nav.toggle_editing(),
            KeyCode::Comma if command => nav.settings = !nav.settings,
            KeyCode::Equal | KeyCode::NumpadAdd if command => nav.change_text_size(1),
            KeyCode::Minus | KeyCode::NumpadSubtract if command => nav.change_text_size(-1),
            KeyCode::ArrowLeft if alt => nav.go_back(),
            KeyCode::ArrowRight if alt => nav.go_forward(),
            KeyCode::Escape => {
                if nav.find {
                    nav.find = false;
                    focus.clear();
                } else if nav.links {
                    nav.links = false;
                } else if nav.switcher {
                    nav.switcher = false;
                    focus.clear();
                } else if nav.settings {
                    nav.settings = false;
                } else if nav.drawer {
                    nav.drawer = false;
                } else {
                    selection.clear();
                }
            }
            KeyCode::ArrowDown | KeyCode::ArrowUp if nav.switcher => {
                let count = nav.switcher_results(&vault).0.len().max(1);
                let index = nav.switcher_index;
                nav.switcher_index = if input.key_code == KeyCode::ArrowDown {
                    (index + 1) % count
                } else {
                    (index + count - 1) % count
                };
            }
            KeyCode::Enter | KeyCode::NumpadEnter if nav.find && !nav.switcher => {
                nav.step_find(step);
            }
            KeyCode::Enter | KeyCode::NumpadEnter if nav.switcher => {
                let (results, _) = nav.switcher_results(&vault);
                if let Some(&id) = results.get(nav.switcher_choice(results.len())) {
                    nav.open(id, None);
                }
                nav.switcher = false;
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
    let current = nav.current;
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

/// How far (logical px) the reading pane scrolls before "back to top" shows.
const SCROLLED_PAST: f32 = 600.0;

fn track_scrolled(mut nav: ResMut<Nav>, panes: Query<&ScrollPosition, With<NotePane>>) {
    let scrolled = nav.view == View::Note
        && panes
            .single()
            .is_ok_and(|position| position.y > SCROLLED_PAST);
    if nav.scrolled != scrolled {
        nav.scrolled = scrolled;
    }
}

/// A heading counts as the current section once its top passes this far (logical px) below
/// the top of the reading pane, so the section a link scrolled to is current right away.
const SECTION_LINE: f32 = 96.0;

/// Follows the reading position: the last heading whose top is above the section line (or,
/// scrolled to the end, the last heading on screen).
fn track_section(
    mut nav: ResMut<Nav>,
    panes: Query<(&ComputedNode, &UiGlobalTransform, &ScrollPosition), With<NotePane>>,
    headings: Query<(&MdHeading, &ComputedNode, &UiGlobalTransform)>,
) {
    let section = match panes.single() {
        Ok((pane, pane_at, scroll)) if nav.view == View::Note => {
            let scale = pane.inverse_scale_factor.recip();
            let top = pane_at.translation.y - pane.size.y / 2.0;
            // At the very end the last sections can't reach the line: then any heading on
            // screen counts, so the last one shown is current.
            let end = (pane.content_size().y - pane.size.y) * pane.inverse_scale_factor;
            let line = if end > 0.0 && scroll.y >= end - 2.0 {
                top + pane.size.y
            } else {
                top + SECTION_LINE * scale
            };
            headings
                .iter()
                .map(|(heading, node, at)| (at.translation.y - node.size.y / 2.0, heading))
                .filter(|(top, _)| *top <= line)
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, heading)| heading.slug.clone())
        }
        _ => None,
    };
    if nav.section != section {
        nav.section = section;
    }
}

/// Where a heading or footnote lands when scrolled to: this far (logical px) below the top of
/// the reading pane, above the section line so it becomes the current section.
const ANCHOR_MARGIN: f32 = 24.0;

/// Scrolls the reading pane so the anchor's target sits at its top, even if it was already
/// on screen.
fn scroll_to_anchor(
    mut nav: ResMut<Nav>,
    headings: Query<(Entity, &MdHeading)>,
    footnotes: Query<(Entity, &MdFootnote)>,
    targets: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut panes: Query<(&ComputedNode, &UiGlobalTransform, &mut ScrollPosition), With<NotePane>>,
    mut commands: Commands,
    mut wait: Local<u8>,
) {
    let Some(anchor) = nav.anchor.clone() else {
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
        if let (Ok((pane, pane_at, mut scroll)), Ok((node, at))) =
            (panes.single_mut(), targets.get(entity))
        {
            let scale = pane.inverse_scale_factor;
            let pane_top = pane_at.translation.y - pane.size.y / 2.0;
            let target_top = at.translation.y - node.size.y / 2.0;
            let end = ((pane.content_size().y - pane.size.y) * scale).max(0.0);
            let y = scroll.y + (target_top - pane_top) * scale - ANCHOR_MARGIN;
            scroll.y = y.clamp(0.0, end);
        } else {
            commands.trigger(ScrollIntoView { entity });
        }
        nav.anchor = None;
    } else if *wait > 30 {
        nav.anchor = None;
    }
}

/// Width class of the window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutMode {
    /// Desktop (≥ 1100 px).
    #[default]
    Wide,
    /// Tablet (≥ 760 px): the sidebar is still inline.
    Medium,
    /// Phone: the sidebar is a drawer and controls are touch-sized.
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

    /// Whether the sidebar is a drawer.
    pub fn drawer(self) -> bool {
        self == Self::Compact
    }
}

//! Note editor: the pen button (or `Ctrl+E`) swaps the reading view for the note's markdown.
//!
//! The text is a Bevy [`EditableText`] (caret, selection, keyboard, mouse, clipboard and IME
//! come from `bevy_ui_widgets`). It draws uniform text only, so its glyphs are made
//! transparent and a [`HighlightLayer`] with coloured spans of the same text, font and width
//! is drawn exactly on top ([`bevy_ui_commonmark::highlight_markdown`]). The node grows with
//! its content so the note pane scrolls it, and follows the caret.
//!
//! Undo/redo (`Ctrl+Z`, `Ctrl+Shift+Z`/`Ctrl+Y`) keeps snapshots. Edits are saved into the
//! vault after a pause, on `Ctrl+S` and when the editor closes; on the web they are kept in
//! `localStorage`, natively written to `vault/` when running from the repository.

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    input_focus::{AutoFocus, FocusedInput, tab_navigation::TabIndex},
    picking::events::{PointerDragEnd, PointerDragStart},
    prelude::*,
    text::{
        EditableText, FontCx, LayoutCx, LineBreak, LineHeight, TextCursorStyle, TextEdit,
        TextEditChange, TextLayoutInfo,
    },
    ui::UiGlobalTransform,
    ui_widgets::TextInput,
};
use bevy_ui_commonmark::{MdTheme, MdToken, highlight_markdown};

use bevsidian_core::{Nav, NoteId, Vault, nav::NotePane, theme::palette};

pub fn plugin(app: &mut App) {
    app.init_resource::<EditSession>()
        .add_observer(open_editor)
        .add_systems(
            Update,
            (refresh_highlight, fit_editor, follow_caret, autosave).chain(),
        )
        .add_systems(
            PostUpdate,
            drop_touch_selection.before(bevy::text::EditableTextSystems),
        );
}

/// Where the editor goes: the note pane shows this instead of the rendered note.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EditorSlot(pub NoteId);

/// The editable text of the open note.
#[derive(Component)]
struct NoteEditor;

/// Coloured copy of the editor's text, drawn over it.
#[derive(Component)]
struct HighlightLayer;

/// One undo step: the text and the selection (byte range) to restore.
#[derive(Clone)]
struct Snapshot {
    text: String,
    selection: (usize, usize),
}

/// The note being edited.
#[allow(clippy::struct_excessive_bools)]
#[derive(Resource, Default)]
struct EditSession {
    note: Option<NoteId>,
    editor: Option<Entity>,
    overlay: Option<Entity>,
    /// The editor's text as last seen.
    text: String,
    selection: (usize, usize),
    /// Edited since the last save.
    dirty: bool,
    last_edit: f64,
    /// The overlay needs new spans.
    stale_highlight: bool,
    /// The caret moved or the text changed: keep it in view.
    reveal: bool,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    /// When the newest undo step was taken (typing within a burst shares one step).
    last_snapshot: f64,
    /// A finger is dragging over the editor: it scrolls the page instead of selecting.
    touch_drag: bool,
}

/// Room below the text (logical px), so the editor never needs to scroll inside.
const SPARE_HEIGHT: f32 = 48.0;

/// Idle time before an edit is saved.
const AUTOSAVE_DELAY: f64 = 0.8;
/// Typing within this many seconds is undone as one step.
const UNDO_BURST: f64 = 0.6;
const UNDO_LIMIT: usize = 200;

fn open_editor(
    add: On<Add<EditorSlot>>,
    slots: Query<&EditorSlot>,
    vault: Res<Vault>,
    nav: Res<Nav>,
    theme: Res<MdTheme>,
    mut session: ResMut<EditSession>,
    mut commands: Commands,
) {
    let Ok(&EditorSlot(id)) = slots.get(add.entity) else {
        return;
    };
    // Switching notes while editing: keep the previous note's edits.
    commit(&mut session, &vault);
    let data = vault.get_untracked();
    let Some(note) = data.notes.get(id) else {
        return;
    };
    let font = TextFont {
        font: theme.mono.clone(),
        font_size: (nav.text_size.get_untracked() as f32 * 0.9).into(),
        ..default()
    };
    let line_height = LineHeight::RelativeToFont(1.6);
    let layout = TextLayout {
        linebreak: LineBreak::WordOrCharacter,
        ..default()
    };
    let mut editable = EditableText {
        allow_newlines: true,
        visible_lines: None,
        cursor_width: 0.12,
        ..EditableText::new(&note.source)
    };
    editable.queue_edit(TextEdit::TextStart(false));
    let editor = commands
        .spawn((
            NoteEditor,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(200.0),
                ..default()
            },
            TextInput,
            editable,
            layout,
            font.clone(),
            line_height,
            // Glyphs come from the highlight layer.
            TextColor(Color::NONE),
            TextCursorStyle {
                color: palette::ACCENT,
                selection_color: palette::ACCENT.with_alpha(0.32),
                unfocused_selection_color: palette::ACCENT.with_alpha(0.18),
                selected_text_color: None,
                selection_radius: 2.0,
            },
            TabIndex(0),
            AutoFocus,
            bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Text),
            ChildOf(add.entity),
        ))
        .observe(on_edit)
        .observe(on_key)
        .observe(
            |drag: On<PointerDragStart>, mut session: ResMut<EditSession>| {
                session.touch_drag = drag.pointer.id.is_touch();
            },
        )
        .observe(|_: On<PointerDragEnd>, mut session: ResMut<EditSession>| {
            session.touch_drag = false;
        })
        .id();
    let overlay = commands
        .spawn((
            HighlightLayer,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            Text::default(),
            layout,
            font,
            line_height,
            TextColor(palette::TEXT),
            Pickable::IGNORE,
            ChildOf(add.entity),
        ))
        .id();
    *session = EditSession {
        note: Some(id),
        editor: Some(editor),
        overlay: Some(overlay),
        text: note.source.clone(),
        stale_highlight: true,
        ..default()
    };
}

/// Records each change of the editor's text or caret.
fn on_edit(
    change: On<TextEditChange>,
    editors: Query<&EditableText, With<NoteEditor>>,
    time: Res<Time<Real>>,
    mut session: ResMut<EditSession>,
) {
    let Ok(editable) = editors.get(change.event_target()) else {
        return;
    };
    let range = editable.editor.raw_selection().text_range();
    let selection = (range.start, range.end);
    session.reveal = true;
    let text = editable.editor.raw_text();
    if text == session.text {
        session.selection = selection;
        return;
    }
    let now = time.elapsed_secs_f64();
    if now - session.last_snapshot > UNDO_BURST || session.undo.is_empty() {
        let snapshot = Snapshot {
            text: std::mem::take(&mut session.text),
            selection: session.selection,
        };
        session.undo.push(snapshot);
        if session.undo.len() > UNDO_LIMIT {
            session.undo.remove(0);
        }
    }
    session.last_snapshot = now;
    session.redo.clear();
    text.clone_into(&mut session.text);
    session.selection = selection;
    session.dirty = true;
    session.last_edit = now;
    session.stale_highlight = true;
}

/// Editor shortcuts that `EditableText` does not handle: undo/redo, save, Tab.
fn on_key(
    mut key: On<FocusedInput<KeyboardInput>>,
    keys: Res<ButtonInput<KeyCode>>,
    vault: Res<Vault>,
    mut session: ResMut<EditSession>,
    mut editors: Query<&mut EditableText, With<NoteEditor>>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
) {
    let input = &key.input;
    if !input.state.is_pressed() {
        return;
    }
    let entity = key.focused_entity;
    let Ok(mut editable) = editors.get_mut(entity) else {
        return;
    };
    if editable.editor.is_composing() {
        return;
    }
    let command = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let restore = match (&input.logical_key, input.key_code) {
        (_, KeyCode::KeyZ) if command && !shift => {
            let previous = session.undo.pop();
            previous.map(|snapshot| (snapshot, true))
        }
        (_, KeyCode::KeyZ | KeyCode::KeyY) if command => {
            let next = session.redo.pop();
            next.map(|snapshot| (snapshot, false))
        }
        (_, KeyCode::KeyS) if command => {
            commit(&mut session, &vault);
            key.propagate(false);
            return;
        }
        (Key::Tab, _) if !command => {
            editable.queue_edit(TextEdit::Insert("    ".into()));
            key.propagate(false);
            return;
        }
        _ => return,
    };
    key.propagate(false);
    let Some((snapshot, is_undo)) = restore else {
        return;
    };
    // The current state goes on the opposite stack.
    let current = Snapshot {
        text: session.text.clone(),
        selection: session.selection,
    };
    if is_undo {
        session.redo.push(current);
    } else {
        session.undo.push(current);
    }
    editable.editor.set_text(&snapshot.text);
    let (start, end) = snapshot.selection;
    let len = snapshot.text.len();
    editable
        .editor
        .driver(&mut font_cx, &mut layout_cx.0)
        .select_byte_range(start.min(len), end.min(len));
    session.text = snapshot.text;
    session.selection = snapshot.selection;
    session.dirty = true;
    session.stale_highlight = true;
    session.reveal = true;
    // A new burst starts after an undo.
    session.last_snapshot = 0.0;
}

/// Touch drags scroll the page (see `bevy_ui_touchscroll`); drop the selection they would
/// also make. Taps still place the caret and double-taps select words.
fn drop_touch_selection(
    session: Res<EditSession>,
    mut editors: Query<&mut EditableText, With<NoteEditor>>,
) {
    if !session.touch_drag {
        return;
    }
    for mut editable in &mut editors {
        if editable
            .pending_edits
            .iter()
            .any(|edit| matches!(edit, TextEdit::ExtendSelectionToPoint(_)))
        {
            editable
                .pending_edits
                .retain(|edit| !matches!(edit, TextEdit::ExtendSelectionToPoint(_)));
        }
    }
}

/// Rebuilds the coloured spans after an edit.
fn refresh_highlight(
    mut session: ResMut<EditSession>,
    theme: Res<MdTheme>,
    overlays: Query<(&TextFont, &LineHeight), With<HighlightLayer>>,
    mut commands: Commands,
) {
    if !session.stale_highlight {
        return;
    }
    session.stale_highlight = false;
    let Some(overlay) = session.overlay else {
        return;
    };
    let Ok((font, line_height)) = overlays.get(overlay) else {
        return;
    };
    let mut entity = commands.entity(overlay);
    entity.despawn_children();
    for (range, token) in highlight_markdown(&session.text) {
        entity.with_child((
            TextSpan::new(&session.text[range]),
            font.clone(),
            *line_height,
            TextColor(token_color(token, &theme)),
        ));
    }
}

fn token_color(token: MdToken, theme: &MdTheme) -> Color {
    match token {
        MdToken::Text => palette::TEXT,
        MdToken::Marker => palette::FAINT,
        MdToken::Heading => palette::ACCENT,
        MdToken::Strong => Color::WHITE,
        MdToken::Emphasis => Color::srgb(0.78, 0.86, 0.95),
        MdToken::Strike | MdToken::Url => palette::MUTED,
        MdToken::Code => theme.code,
        MdToken::Link | MdToken::Tag => theme.link,
        MdToken::FrontMatter => Color::srgb(0.55, 0.7, 0.6),
        MdToken::Quote => Color::srgb(0.7, 0.7, 0.75),
    }
}

/// Grows the editor with its text (one spare line, so it never scrolls inside), and keeps
/// the highlight layer on the editor's scroll offset.
fn fit_editor(
    mut editors: Query<
        (&EditableText, &TextLayoutInfo, &ComputedNode, &mut Node),
        With<NoteEditor>,
    >,
    mut overlays: Query<&mut Node, (With<HighlightLayer>, Without<NoteEditor>)>,
) {
    for (editable, info, computed, mut node) in &mut editors {
        let scale = computed.inverse_scale_factor;
        let height = Val::Px(info.size.y * scale + SPARE_HEIGHT);
        if node.height != height {
            node.height = height;
        }
        let offset = editable.viewport.offset * scale;
        for mut overlay in &mut overlays {
            let (left, top) = (Val::Px(-offset.x), Val::Px(-offset.y));
            if overlay.top != top || overlay.left != left {
                overlay.top = top;
                overlay.left = left;
            }
        }
    }
}

/// Scrolls the note pane so the caret stays visible while typing or moving it.
fn follow_caret(
    mut session: ResMut<EditSession>,
    editors: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<NoteEditor>>,
    mut panes: Query<(&ComputedNode, &UiGlobalTransform, &mut ScrollPosition), With<NotePane>>,
) {
    if !session.reveal {
        return;
    }
    session.reveal = false;
    let Some(Ok((info, node, transform))) = session.editor.map(|e| editors.get(e)) else {
        return;
    };
    let Some((_, caret)) = info.cursor else {
        return;
    };
    let Ok((pane, pane_transform, mut scroll)) = panes.single_mut() else {
        return;
    };
    // Physical pixels, in window space.
    let top = transform.translation.y - node.size.y / 2.0;
    let (caret_top, caret_bottom) = (top + caret.min.y, top + caret.max.y);
    let view_top = pane_transform.translation.y - pane.size.y / 2.0;
    let view_bottom = view_top + pane.size.y;
    let margin = caret.height() * 2.0;
    let delta = if caret_top < view_top + margin {
        caret_top - view_top - margin
    } else if caret_bottom > view_bottom - margin {
        caret_bottom - view_bottom + margin
    } else {
        return;
    };
    scroll.0.y = (scroll.0.y + delta * pane.inverse_scale_factor).max(0.0);
}

/// Saves after a pause in typing, or once the editor is gone.
fn autosave(
    time: Res<Time<Real>>,
    vault: Res<Vault>,
    nav: Res<Nav>,
    editors: Query<(), With<NoteEditor>>,
    mut session: ResMut<EditSession>,
) {
    let open = session.editor.is_some_and(|e| editors.contains(e));
    if session.dirty && (!open || time.elapsed_secs_f64() - session.last_edit > AUTOSAVE_DELAY) {
        commit(&mut session, &vault);
    }
    if !open && session.note.is_some() && !nav.editing.get_untracked() {
        *session = EditSession::default();
    }
}

/// Saves the edited text into the vault and storage.
fn commit(session: &mut EditSession, vault: &Vault) {
    let Some(id) = session.note else {
        return;
    };
    if !session.dirty {
        return;
    }
    session.dirty = false;
    if vault.save(id, &session.text) {
        let path = vault.get_untracked().notes[id].path.clone();
        persist(&path, &session.text);
    }
}

/// Key under which an edited note is kept in `localStorage`.
#[cfg(target_arch = "wasm32")]
fn storage_key(path: &str) -> String {
    format!("bevsidian.note:{path}")
}

fn persist(path: &str, source: &str) {
    #[cfg(target_arch = "wasm32")]
    crate::settings::write(&storage_key(path), source);
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Running from the repository: write the note back to its file.
        let file = std::path::Path::new("vault").join(path);
        if file.is_file()
            && let Err(error) = std::fs::write(&file, source)
        {
            warn!("could not save {}: {error}", file.display());
        }
    }
}

/// The embedded notes, with edits saved in this browser applied.
pub fn load_edits(notes: &[(&'static str, &'static str)]) -> Vec<(String, String)> {
    notes
        .iter()
        .map(|&(path, source)| {
            #[cfg(target_arch = "wasm32")]
            if let Some(edited) = crate::settings::read(&storage_key(path)) {
                return (path.to_owned(), edited);
            }
            (path.to_owned(), source.to_owned())
        })
        .collect()
}

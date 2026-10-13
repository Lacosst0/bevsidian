//! The application shell: sidebars, top bar, reading pane and quick switcher.
//!
//! Static structure is plain BSN; everything that depends on navigation state is a
//! `view!` region driven by the [`Nav`] signals.

use bevy::{
    feathers::controls::{FeathersTextInput, FeathersTextInputContainer},
    input_focus::AutoFocus,
    prelude::*,
    text::FontWeight,
    ui_widgets::{Activate, ScrollArea},
};
use bevy_radioactive::view::{memo, view};
use bevy_ui_commonmark::{MdTheme, RenderOptions, markdown_scene};
use bevy_ui_textselect::{ClickableSpan, SelectableText};

use bevsidian_core::{
    nav::{LayoutMode, Nav, NotePane, SearchInput, View},
    vault::{NoteId, Vault, VaultData},
};
use bevsidian_graph::GraphViewport;
use bevsidian_ui::{
    Selected, Size, Variant, badge_button, button, caption, color, dialog, empty, glyph, icon,
    icon_button, icon_font, item, label, layer, radius, scrim, section_label, shortcuts, toggle,
    toggle_group, ui_font,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_shell);
}

fn spawn_shell(mut commands: Commands, nav: Res<Nav>, vault: Res<Vault>, theme: Res<MdTheme>) {
    // No MSAA: UI is anti-aliased per glyph/shape, and 4x MSAA at phone resolutions is costly.
    commands.spawn((Camera2d, IsDefaultUiCamera, Msaa::Off));
    commands.spawn_scene(shell(nav.clone(), vault.clone(), theme.clone()));
}

fn shell(nav: Nav, vault: Vault, theme: MdTheme) -> impl Scene {
    let left = left_sidebar(nav.clone(), vault.clone());
    let main = main_area(nav.clone(), vault.clone(), theme);
    let right = right_sidebar(nav.clone(), vault.clone());
    let backdrop = drawer_backdrop(nav.clone());
    let selection = crate::layout::selection_bar(nav.clone());
    let keyboard = crate::layout::keyboard_panel(nav.clone());
    let settings = crate::settings::settings_panel(nav.clone());
    let switcher = quick_switcher(nav, vault);
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            // UI roots are flex items of the camera viewport: without this the root grows to
            // the min-content width of wide content (e.g. long code lines) on narrow screens.
            min_width: Val::Px(0.0),
            min_height: Val::Px(0.0),
            overflow: Overflow::clip(),
        }
        BackgroundColor(color::BACKGROUND)
        Children [
            @{left}
            --
            @{main}
            --
            @{right}
            --
            @{backdrop}
            --
            @{switcher}
            --
            @{settings}
            --
            @{selection}
            --
            @{keyboard}
        ]
    }
}

/// Dims the page behind an open drawer; tapping it closes the drawer.
fn drawer_backdrop(nav: Nav) -> impl Scene {
    let (mode, left_open, right_open) = (
        nav.mode.clone(),
        nav.left_open.clone(),
        nav.right_open.clone(),
    );
    view! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
        }
        GlobalZIndex(layer::DRAWER_BACKDROP)
        Pickable::IGNORE
        Children [
            if ($left_open && $mode.left_drawer()) || ($right_open && $mode.right_drawer()) {
                let nav = nav.clone();
                @{scrim(move || nav.close_drawers())}
            }
        ]
    }
}

fn left_sidebar(nav: Nav, vault: Vault) -> impl Scene {
    let files_title = section_label("Files");
    let tags_title = section_label("Tags");

    let tags_vault = vault.clone();
    let (query, tag, current) = (nav.query.clone(), nav.tag.clone(), nav.current.clone());
    let files_vault = vault.clone();
    let empty_vault = vault.clone();
    let excerpt_vault = vault.clone();
    let nav_files = nav.clone();
    let nav_tags = nav.clone();
    let nav_clear = nav.clone();
    let (mode, open, touch) = (nav.mode.clone(), nav.left_open.clone(), nav.touch.clone());
    view! {
        Node {
            display: $(if !$mode.left_drawer() || $open { Display::Flex } else { Display::None }),
            position_type: $(if $mode.left_drawer() { PositionType::Absolute } else { PositionType::Relative }),
            width: $(if $mode.left_drawer() { Val::Percent(86.0) } else { Val::Px(260.0) }),
            max_width: Val::Px(340.0),
            height: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            border: UiRect::right(Val::Px(1.0)),
        }
        GlobalZIndex($(if $mode.left_drawer() { layer::DRAWER } else { 0 }))
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        BoxShadow::new(color::SHADOW.with_alpha($(if $mode.left_drawer() { 0.5 } else { 0.0 })), Val::Px(0.0), Val::Px(0.0), Val::Px(0.0), Val::Px(24.0))
        Children [
            Node {
                padding: UiRect::new(Val::Px(14.0), Val::Px(14.0), Val::Px(14.0), Val::Px(8.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
            }
            Children [
                Node { width: Val::Px(18.0), height: Val::Px(18.0), border_radius: BorderRadius::all(Val::Px(radius::MD)) }
                BackgroundColor(color::PRIMARY)
                --
                Text("Bevsidian") @{ui_font(13.0, FontWeight::MEDIUM)} TextColor(color::FOREGROUND)
            ]
            --
            Node { padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)) }
            Children [
                @FeathersTextInputContainer
                Node { flex_grow: 1.0 }
                Children [
                    @FeathersTextInput
                    SearchInput::Sidebar
                ]
            ]
            --
            if $tag.is_some() {
                // The active tag filter; clicking it clears the filter.
                let nav_clear = nav_clear.clone();
                Node { padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(0.0)) }
                Children [
                    @{badge_button()}
                    Selected(true)
                    ClickableSpan
                    on(move |_: On<Activate>| nav_clear.clear_tag())
                    Children [
                        Text($(format!("#{}", $tag.clone().unwrap_or_default()))) @{ui_font(12.0, FontWeight::MEDIUM)} TextColor(color::PRIMARY) Pickable::IGNORE
                        --
                        @{icon("x", 13.0, color::MUTED_FOREGROUND)}
                    ]
                ]
            }
            --
            @{files_title}
            --
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                overflow: Overflow::scroll_y(),
                padding: UiRect::horizontal(Val::Px(6.0)),
            }
            ScrollArea
            Children [
                #[key(entry.clone())]
                for entry in $({
                    // The revision re-renders rows (icons, excerpts) after an edit.
                    let vault = files_vault.get();
                    vault.search(&$query, $tag.as_deref()).into_iter().map(|id| (id, $query.clone(), vault.revision)).collect::<Vec<_>>()
                }) {
                    let (id, search, _) = entry;
                    @{file_row(id, nav_files.clone(), &excerpt_vault.get_untracked(), &search, current.clone())}
                }
                --
                if $(empty_vault.get().search(&$query, $tag.as_deref()).is_empty()) {
                    @{empty("No matching notes")}
                }
            ]
            --
            @{tags_title}
            --
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(0.0), Val::Px(14.0)),
            }
            Children [
                #[key(entry.clone())]
                for entry in $(tags_vault.get().tags.iter().map(|(name, notes)| (name.clone(), notes.len())).collect::<Vec<_>>()) {
                    let (name, count) = entry;
                    @{tag_chip(name, count, nav_tags.clone())}
                }
            ]
            --
            @{shortcuts(&[("Ctrl O", "switch"), ("Ctrl G", "graph"), ("Ctrl ±", "text size")])}
            Node {
                display: $(if $touch { Display::None } else { Display::Flex }),
                margin: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(0.0), Val::Px(12.0)),
            }
        ]
    }
}

fn file_row(
    id: NoteId,
    nav: Nav,
    vault: &VaultData,
    query: &str,
    current: bevy_radioactive::view::Signal<NoteId>,
) -> impl Scene + use<> {
    let note = &vault.notes[id];
    let name = Text::new(note.name.clone());
    let icon = Text::new(note.icon.to_string());
    let folder = (!note.folder.is_empty()).then(|| {
        let text = Text::new(format!("{}/", note.folder));
        bsn! { text @{ui_font(11.0, FontWeight::NORMAL)} TextColor(color::SUBTLE_FOREGROUND) }
    });
    let excerpt = vault.excerpt(id, query).map(|excerpt| {
        let text = Text::new(excerpt);
        bsn! { text @{ui_font(11.5, FontWeight::NORMAL)} TextColor(color::MUTED_FOREGROUND) }
    });
    let open = nav.clone();
    view! {
        @{item()}
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
            row_gap: Val::Px(2.0),
        }
        Selected($($current == id))
        ClickableSpan
        on(move |_: On<Activate>| open.open(id, None))
        Children [
            @{folder}
            --
            Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0) }
            Pickable::IGNORE
            Children [
                icon @{icon_font(15.0)} TextColor($(if $current == id { color::PRIMARY } else { color::SUBTLE_FOREGROUND }))
                Pickable::IGNORE
                --
                name @{ui_font(14.5, FontWeight::NORMAL)} TextColor($(if $current == id { color::FOREGROUND } else { color::MUTED_FOREGROUND }))
                Pickable::IGNORE
            ]
            --
            @{excerpt}
        ]
    }
}

fn tag_chip(tag: String, count: usize, nav: Nav) -> impl Scene {
    let label = Text::new(format!("#{tag}"));
    let count = Text::new(count.to_string());

    let active_tag = nav.tag.clone();
    let chip_tag = tag.clone();
    view! {
        @{badge_button()}
        Selected($($active_tag.as_deref() == Some(tag.as_str())))
        on(move |_: On<Activate>| nav.toggle_tag(&chip_tag))
        Children [
            label @{ui_font(12.0, FontWeight::MEDIUM)} TextColor(color::PRIMARY) Pickable::IGNORE
            --
            count @{ui_font(11.0, FontWeight::NORMAL)} TextColor(color::MUTED_FOREGROUND) Pickable::IGNORE
        ]
    }
}

fn top_bar(nav: Nav, vault: Vault) -> impl Scene {
    let (back, forward, view, current, mode) = (
        nav.back.clone(),
        nav.forward.clone(),
        nav.view.clone(),
        nav.current.clone(),
        nav.mode.clone(),
    );
    let nav_settings = nav.clone();
    let (nav_edit, editing) = (nav.clone(), nav.editing.clone());
    let (pen, done) = (glyph("pencil"), glyph("check"));
    let (back_glyph, forward_glyph) = (glyph("chevron-left"), glyph("chevron-right"));
    let (nav_back, nav_fwd, nav_note, nav_graph, nav_switch, nav_left, nav_right) = (
        nav.clone(),
        nav.clone(),
        nav.clone(),
        nav.clone(),
        nav.clone(),
        nav.clone(),
        nav.clone(),
    );
    view! {
        Node {
            height: $(if $mode == LayoutMode::Compact { Val::Px(50.0) } else { Val::Px(44.0) }),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(6.0)),
            column_gap: Val::Px(2.0),
            border: UiRect::bottom(Val::Px(1.0)),
        }
        BorderColor::all(color::BORDER)
        Children [
            @{icon_button(Variant::Ghost, Size::Icon, "panel-left")}
            Node { display: $(if $mode.left_drawer() { Display::Flex } else { Display::None }) }
            on(move |_: On<Activate>| nav_left.toggle_drawer(true))
            --
            @{button(Variant::Ghost, Size::Icon)}
            on(move |_: On<Activate>| nav_back.go_back())
            Children [
                Text({back_glyph.clone()}) @{icon_font(20.0)} Pickable::IGNORE
                TextColor($(if $back.is_empty() { color::SUBTLE_FOREGROUND } else { color::FOREGROUND }))
            ]
            --
            @{button(Variant::Ghost, Size::Icon)}
            on(move |_: On<Activate>| nav_fwd.go_forward())
            Children [
                Text({forward_glyph.clone()}) @{icon_font(20.0)} Pickable::IGNORE
                TextColor($(if $forward.is_empty() { color::SUBTLE_FOREGROUND } else { color::FOREGROUND }))
            ]
            --
            Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_width: Val::Px(0.0),
                overflow: Overflow::clip(),
                // Centred when it fits; on phones it starts at the left and is cut at the end.
                justify_content: $(if $mode == LayoutMode::Compact { JustifyContent::Start } else { JustifyContent::Center }),
                margin: UiRect::horizontal(Val::Px(6.0)),
            }
            Children [
                Text($(match $view {
                    View::Note if $mode == LayoutMode::Compact => vault.get().notes[$current].name.clone(),
                    View::Note => vault.get().notes[$current].path.trim_end_matches(".md").replace('/', " / "),
                    View::Graph => "Graph view".to_owned(),
                }))
                @{ui_font(13.5, FontWeight::MEDIUM)}
                TextColor(color::MUTED_FOREGROUND)
                TextLayout { linebreak: LineBreak::NoWrap }
            ]
            --
            @{toggle_group()}
            Node { margin: UiRect::horizontal(Val::Px(4.0)) }
            Children [
                @{toggle("Note")}
                Selected($($view == View::Note))
                on(move |_: On<Activate>| nav_note.show(View::Note))
                --
                @{toggle("Graph")}
                Selected($($view == View::Graph))
                on(move |_: On<Activate>| nav_graph.show(View::Graph))
            ]
            --
            @{icon_button(Variant::Ghost, Size::Icon, "search")}
            Node { display: $(if $mode == LayoutMode::Compact { Display::None } else { Display::Flex }) }
            on(move |_: On<Activate>| nav_switch.open_switcher())
            --
            // Edit the note (Ctrl+E); a check mark closes the editor.
            @{button(Variant::Ghost, Size::Icon)}
            Node { display: $(if $view == View::Note { Display::Flex } else { Display::None }) }
            on(move |_: On<Activate>| nav_edit.toggle_editing())
            Children [
                Text($(if $editing { done.clone() } else { pen.clone() }))
                @{icon_font(18.0)}
                TextColor($(if $editing { color::PRIMARY } else { color::FOREGROUND }))
                Pickable::IGNORE
            ]
            --
            @{icon_button(Variant::Ghost, Size::Icon, "sliders-horizontal")}
            on(move |_: On<Activate>| nav_settings.settings.update(|open| *open = !*open))
            --
            @{icon_button(Variant::Ghost, Size::Icon, "panel-right")}
            Node { display: $(if $mode.right_drawer() { Display::Flex } else { Display::None }) }
            on(move |_: On<Activate>| nav_right.toggle_drawer(false))
        ]
    }
}

fn main_area(nav: Nav, vault: Vault, theme: MdTheme) -> impl Scene {
    let top = top_bar(nav.clone(), vault.clone());
    let view = nav.view.clone();
    // The reading pane is rebuilt when the note, text size, layout or vault content changes.
    let note_key = {
        let (current, size, mode) = (nav.current.clone(), nav.text_size.clone(), nav.mode.clone());
        let vault = vault.clone();
        memo(move || (current.get(), size.get(), mode.get(), vault.get().revision))
    };
    let render_note = {
        let nav = nav.clone();
        move |(id, size, mode, _): (NoteId, u32, LayoutMode, u64)| {
            note_scene((id, size, mode), &vault, &nav, &theme)
        }
    };
    let mode = nav.mode.clone();
    let (editing, current) = (nav.editing.clone(), nav.current.clone());
    let graph_nav = nav.clone();
    view! {
        Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: Val::Px(0.0),
            min_width: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip_x(),
        }
        Children [
            @{top}
            --
            match $view {
                View::Note => {
                    let (note_key, render_note) = (note_key.clone(), render_note.clone());
                    Node {
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        overflow: Overflow { x: OverflowAxis::Clip, y: OverflowAxis::Scroll },
                        justify_content: JustifyContent::Center,
                        min_width: Val::Px(0.0),
                    }
                    NotePane
                    Children [
                        Node {
                            width: Val::Percent(100.0),
                            max_width: Val::Px(780.0),
                            min_width: Val::Px(0.0),
                            padding: $(if $mode == LayoutMode::Compact {
                                UiRect::new(Val::Px(18.0), Val::Px(18.0), Val::Px(18.0), Val::Px(140.0))
                            } else {
                                UiRect::new(Val::Px(40.0), Val::Px(40.0), Val::Px(28.0), Val::Px(120.0))
                            }),
                            flex_direction: FlexDirection::Column,
                            align_self: AlignSelf::Start,
                        }
                        Children [
                            if $editing {
                                Node { position_type: PositionType::Relative, width: Val::Percent(100.0) }
                                crate::editor::EditorSlot($current)
                            } else {
                                let (note_key, render_note) = (note_key.clone(), render_note.clone());
                                Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0) }
                                Children [
                                    let render_note = render_note.clone();
                                    match $note_key {
                                        key => { @{render_note(key)} }
                                    }
                                ]
                            }
                        ]
                    ]
                },
                View::Graph => {
                    let graph_nav = graph_nav.clone();
                    Node { flex_grow: 1.0, flex_basis: Val::Px(0.0), overflow: Overflow::clip() }
                    GraphViewport
                    Children [ @{bevsidian_graph::zoom_controls(graph_nav)} ]
                },
            }
        ]
    }
}

/// The note icon shown before its title (`size` is the title's font size): an accent glyph on
/// a tinted rounded tile, centred on the title's first line.
fn title_icon(icon: char, size: f32) -> impl Scene + use<> {
    let tile = (size * 1.5).round();
    let line = size * 1.2;
    let node = Node {
        width: Val::Px(tile),
        height: Val::Px(tile),
        flex_shrink: 0.0,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        // Centred on the first line.
        margin: UiRect::top(Val::Px(((line - tile) / 2.0).round())),
        border_radius: BorderRadius::all(Val::Px((tile * 0.28).round())),
        ..default()
    };
    let tint = BackgroundColor(color::PRIMARY.with_alpha(0.14));
    let text = Text::new(icon.to_string());
    bsn! {
        node
        tint
        Pickable::IGNORE
        Children [ text @{icon_font(tile * 0.6)} TextColor(color::PRIMARY) Pickable::IGNORE ]
    }
}

fn note_scene(
    (id, text_size, mode): (NoteId, u32, LayoutMode),
    vault: &Vault,
    nav: &Nav,
    theme: &MdTheme,
) -> Box<dyn Scene> {
    let vault = vault.get_untracked();
    let nav = nav.clone();
    let mut theme = theme.clone();
    theme.base_size = text_size as f32;
    if mode == LayoutMode::Compact {
        theme = theme.compact();
    }
    let note = &vault.notes[id];
    let resolve = |link: &bevy_ui_commonmark::WikiLink| vault.resolve(link).is_some();
    let icon = note.icon;
    let badge = move |size: f32| -> Box<dyn Scene> { Box::new(title_icon(icon, size)) };
    let options = RenderOptions {
        theme: &theme,
        resolve: &resolve,
        title_badge: Some(&badge),
        code_preview: Some(&bevsidian_preview::code_preview),
    };
    let body = markdown_scene(&note.doc, &options);
    // Obsidian-style inline title when the note has no level-1 heading.
    let title = note.doc.title.is_none().then(|| {
        let text = Text::new(note.name.clone());
        let size = theme.heading_size(1);
        let row = Node {
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(size * 0.45),
            margin: UiRect::bottom(Val::Px(16.0)),
            ..default()
        };
        bsn! {
            row
            Children [
                @{title_icon(icon, size)}
                --
                text @{ui_font(size, FontWeight::BOLD)} TextColor(color::FOREGROUND) SelectableText Node { flex_shrink: 1.0 }
            ]
        }
    });
    // Without an inline sidebar, show backlinks at the end of the note like Obsidian mobile.
    let footer: Vec<Box<dyn Scene>> = (mode.right_drawer() && !note.backlinks.is_empty())
        .then(|| {
            let rows: Vec<Box<dyn Scene>> = note
                .backlinks
                .iter()
                .map(|&source| {
                    let link = note_label(&vault.notes[source], 15.0, color::PRIMARY);
                    let nav = nav.clone();
                    Box::new(bsn! {
                        @{side_row(Val::Px(10.0))}
                        on(move |_: On<Activate>| nav.open(source, None))
                        Children [ @{link} ]
                    }) as Box<dyn Scene>
                })
                .collect();
            let heading = section_label(&format!("Linked mentions ({})", note.backlinks.len()));
            Box::new(bsn! {
                Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(36.0)),
                    padding: UiRect::top(Val::Px(8.0)),
                    border: UiRect::top(Val::Px(1.0)),
                }
                BorderColor::all(color::BORDER)
                Children [ @{heading} -- {rows} ]
            }) as Box<dyn Scene>
        })
        .into_iter()
        .collect();
    Box::new(bsn! {
        Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), min_width: Val::Px(0.0) }
        Children [
            {title}
            --
            @{body}
            --
            {footer}
        ]
    })
}

fn right_sidebar(nav: Nav, vault: Vault) -> impl Scene {
    let side_key = {
        let current = nav.current.clone();
        let vault = vault.clone();
        memo(move || (current.get(), vault.get().revision))
    };
    let render_side = {
        let nav = nav.clone();
        move |(id, _): (NoteId, u64)| side_panel(id, &vault, nav.clone())
    };
    let (mode, open) = (nav.mode.clone(), nav.right_open.clone());
    view! {
        Node {
            display: $(if !$mode.right_drawer() || $open { Display::Flex } else { Display::None }),
            position_type: $(if $mode.right_drawer() { PositionType::Absolute } else { PositionType::Relative }),
            right: Val::Px(0.0),
            width: $(if $mode == LayoutMode::Compact { Val::Percent(86.0) } else { Val::Px(260.0) }),
            max_width: Val::Px(320.0),
            height: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            border: UiRect::left(Val::Px(1.0)),
            overflow: Overflow::scroll_y(),
        }
        GlobalZIndex($(if $mode.right_drawer() { layer::DRAWER } else { 0 }))
        ScrollArea
        BackgroundColor(color::CARD)
        BorderColor::all(color::BORDER)
        Children [
            Node { flex_direction: FlexDirection::Column, padding: UiRect::bottom(Val::Px(16.0)) }
            Children [
                let render_side = render_side.clone();
                match $side_key {
                    key => { @{render_side(key)} }
                }
            ]
        ]
    }
}

fn side_panel(id: NoteId, vault: &Vault, nav: Nav) -> Box<dyn Scene> {
    let vault = vault.get_untracked();
    let note = &vault.notes[id];

    let outline: Vec<Box<dyn Scene>> = note
        .doc
        .headings
        .iter()
        .filter(|h| h.level > 1 || note.doc.headings.iter().filter(|h| h.level == 1).count() > 1)
        .map(|heading| {
            let text = Text::new(heading.text.clone());

            let indent = Val::Px(12.0 + 12.0 * f32::from(heading.level.saturating_sub(2)));
            let slug = heading.text.clone();
            let nav = nav.clone();
            Box::new(bsn! {
                @{side_row(indent)}
                on(move |_: On<Activate>| nav.open(id, Some(slug.clone())))
                Children [ text @{ui_font(13.0, FontWeight::NORMAL)} TextColor(color::MUTED_FOREGROUND) Pickable::IGNORE ]
            }) as Box<dyn Scene>
        })
        .collect();

    let note_rows = |ids: &[NoteId]| -> Vec<Box<dyn Scene>> {
        ids.iter()
            .map(|&target| {
                let link = note_label(&vault.notes[target], 13.0, color::PRIMARY);
                let nav = nav.clone();
                Box::new(bsn! {
                    @{side_row(Val::Px(12.0))}
                    on(move |_: On<Activate>| nav.open(target, None))
                    Children [ @{link} ]
                }) as Box<dyn Scene>
            })
            .collect()
    };
    let backlinks = note_rows(&note.backlinks);
    let outgoing = note_rows(&note.outgoing);
    let tags: Vec<Box<dyn Scene>> = note
        .doc
        .tags
        .iter()
        .map(|tag| {
            let count = vault.tags.get(tag).map_or(0, Vec::len);
            Box::new(tag_chip(tag.clone(), count, nav.clone())) as Box<dyn Scene>
        })
        .collect();

    let empty = |text: &str| -> Option<Box<dyn Scene>> { Some(Box::new(empty(text))) };
    let outline_empty: Vec<_> = outline
        .is_empty()
        .then(|| empty("No headings"))
        .flatten()
        .into_iter()
        .collect();
    let backlinks_empty: Vec<_> = backlinks
        .is_empty()
        .then(|| empty("No backlinks"))
        .flatten()
        .into_iter()
        .collect();
    let outgoing_empty: Vec<_> = outgoing
        .is_empty()
        .then(|| empty("No links"))
        .flatten()
        .into_iter()
        .collect();
    let stats = caption(&format!(
        "{} words · {} backlinks",
        note.doc.plain_text().split_whitespace().count(),
        note.backlinks.len()
    ));

    let titles = (
        section_label("Outline"),
        section_label(&format!("Backlinks ({})", note.backlinks.len())),
        section_label("Outgoing links"),
        section_label("Tags"),
    );
    let (t_outline, t_back, t_out, t_tags) = titles;
    Box::new(bsn! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            @{t_outline}
            --
            Node { flex_direction: FlexDirection::Column } Children [ {outline} -- {outline_empty} ]
            --
            @{t_back}
            --
            Node { flex_direction: FlexDirection::Column } Children [ {backlinks} -- {backlinks_empty} ]
            --
            @{t_out}
            --
            Node { flex_direction: FlexDirection::Column } Children [ {outgoing} -- {outgoing_empty} ]
            --
            @{t_tags}
            --
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
            }
            Children [ {tags} ]
            --
            @{stats}
            Node { margin: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(18.0), Val::Px(0.0)) }
        ]
    })
}

/// A note's icon and name, e.g. in link lists.
fn note_label(
    note: &bevsidian_core::vault::Note,
    size: f32,
    text_color: Color,
) -> impl Scene + use<> {
    let (icon, name) = (
        Text::new(note.icon.to_string()),
        label(&note.name, size, FontWeight::NORMAL, text_color),
    );
    let row = Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(size * 0.5),
        ..default()
    };
    bsn! {
        row
        Pickable::IGNORE
        Children [
            icon @{icon_font(size)} TextColor(color::SUBTLE_FOREGROUND) Pickable::IGNORE
            --
            @{name}
        ]
    }
}

/// A side panel row, indented by `indent`.
fn side_row(indent: Val) -> impl Scene {
    bsn! {
        @{item()}
        Node {
            padding: UiRect::new(indent, Val::Px(10.0), Val::Px(7.0), Val::Px(7.0)),
            margin: UiRect::horizontal(Val::Px(6.0)),
        }
    }
}

fn quick_switcher(nav: Nav, vault: Vault) -> impl Scene {
    let (is_open, query, index) = (
        nav.switcher.clone(),
        nav.switcher_query.clone(),
        nav.switcher_index.clone(),
    );
    let close = nav.switcher.clone();
    let names = vault.clone();

    let nav_rows = nav.clone();
    view! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
        }
        GlobalZIndex(layer::COMMAND)
        Pickable::IGNORE
        Children [
            if $is_open {
                let close = close.clone();
                let nav_rows = nav_rows.clone();
                let names = names.clone();
                let vault = vault.clone();
                @{scrim(move || close.set(false))}
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Start,
                    padding: UiRect::top(Val::Percent(8.0)),
                }
                Children [
                    // A command palette: search, results, key hints.
                    @{dialog(560.0)}
                    Node { padding: UiRect::all(Val::Px(10.0)), row_gap: Val::Px(6.0) }
                    Children [
                        @FeathersTextInputContainer
                        Children [
                            @FeathersTextInput
                            SearchInput::Switcher
                            AutoFocus
                        ]
                        --
                        #[key(*entry)]
                        for entry in $(vault.get().search(&$query, None).into_iter().take(12).enumerate().collect::<Vec<_>>()) {
                            let (i, id) = entry;
                            let nav_rows = nav_rows.clone();
                            let names = names.clone();
                            @{item()}
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Stretch,
                                row_gap: Val::Px(2.0),
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            }
                            Selected($($index == i))
                            on(move |_: On<Activate>| {
                                nav_rows.open(id, None);
                                nav_rows.switcher.set(false);
                            })
                            Children [
                                @{note_label(&names.get_untracked().notes[id], 14.0, color::FOREGROUND)}
                                --
                                @{caption(&names.get_untracked().notes[id].path)}
                            ]
                        }
                        --
                        @{shortcuts(&[("↑ ↓", "navigate"), ("↵", "open"), ("Esc", "close")])}
                        Node { margin: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(4.0), Val::Px(2.0)) }
                    ]
                ]
            }
        ]
    }
}

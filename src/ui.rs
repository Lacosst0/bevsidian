//! The application shell: sidebars, top bar, reading pane and quick switcher.
//!
//! Static structure is plain BSN; everything that depends on navigation state is a
//! `coat!` region or `shard` driven by the [`Nav`] signals.

use bevy::{
    feathers::controls::{FeathersTextInput, FeathersTextInputContainer},
    input_focus::{AutoFocus, tab_navigation::TabIndex},
    picking::events::PointerClick,
    prelude::*,
    text::FontWeight,
    ui_widgets::{Activate, ScrollArea},
};
use bevy_radioactive::coat::{coat, shard};
use bevy_ui_commonmark::{MdTheme, RenderOptions, insert, markdown_scene};
use bevy_ui_textselect::{ClickableSpan, SelectableText};

use crate::{
    graph::GraphViewport,
    nav::{Nav, NotePane, SearchInput, View},
    vault::{NoteId, Vault},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_shell)
        .add_systems(Update, hover_rows);
}

/// Interface colours (Obsidian-like dark palette).
pub mod palette {
    use bevy::prelude::Color;

    pub const BG: Color = Color::srgb(0.118, 0.118, 0.118);
    pub const BG_SIDEBAR: Color = Color::srgb(0.149, 0.149, 0.149);
    pub const BG_HOVER: Color = Color::srgb(0.2, 0.2, 0.21);
    pub const BG_ACTIVE: Color = Color::srgb(0.24, 0.22, 0.32);
    pub const BORDER: Color = Color::srgb(0.21, 0.21, 0.21);
    pub const TEXT: Color = Color::srgb(0.855, 0.855, 0.855);
    pub const MUTED: Color = Color::srgb(0.6, 0.6, 0.6);
    pub const FAINT: Color = Color::srgb(0.42, 0.42, 0.42);
    pub const ACCENT: Color = Color::srgb(0.659, 0.545, 0.98);
    pub const TAG_BG: Color = Color::srgba(0.545, 0.424, 0.937, 0.18);
}

use palette::{ACCENT, BG, BG_ACTIVE, BG_HOVER, BG_SIDEBAR, BORDER, FAINT, MUTED, TAG_BG, TEXT};

fn ui_font(size: f32, weight: FontWeight) -> impl Scene {
    insert(TextFont {
        font: bevy::text::FontSource::families("Inter, sans-serif"),
        font_size: size.into(),
        weight,
        ..default()
    })
}

/// A list row that highlights on hover.
#[derive(Component, Clone, Copy, Debug, Default)]
struct HoverRow;

fn hover_rows(
    mut rows: Query<
        (
            &bevy::picking::hover::Hovered,
            &mut BackgroundColor,
            Option<&ActiveRow>,
        ),
        (With<HoverRow>, Changed<bevy::picking::hover::Hovered>),
    >,
) {
    for (hovered, mut bg, active) in &mut rows {
        if active.is_some() {
            continue;
        }
        bg.0 = if hovered.get() { BG_HOVER } else { Color::NONE };
    }
}

/// Marks the row of the open note.
#[derive(Component, Clone, Copy, Debug, Default)]
struct ActiveRow;

fn spawn_shell(mut commands: Commands, nav: Res<Nav>, vault: Res<Vault>) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.spawn_scene(shell(nav.clone(), vault.clone()));
}

fn shell(nav: Nav, vault: Vault) -> impl Scene {
    let left = left_sidebar(nav.clone(), vault.clone());
    let main = main_area(nav.clone(), vault.clone());
    let right = right_sidebar(nav.clone());
    let switcher = quick_switcher(nav, vault);
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
        }
        BackgroundColor(BG)
        Children [
            @{left}
            --
            @{main}
            --
            @{right}
            --
            @{switcher}
        ]
    }
}

fn section_title(title: &str) -> impl Scene + use<> {
    let text = Text::new(title.to_uppercase());
    bsn! {
        text @{ui_font(11.0, FontWeight::SEMIBOLD)}
        TextColor(FAINT)
        Node { margin: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(14.0), Val::Px(6.0)) }
    }
}

fn left_sidebar(nav: Nav, vault: Vault) -> impl Scene {
    let files_title = section_title("Files");
    let tags_title = section_title("Tags");

    let tags: Vec<(String, usize)> = vault
        .tags
        .iter()
        .map(|(tag, notes)| (tag.clone(), notes.len()))
        .collect();
    let (query, tag, current) = (nav.query.clone(), nav.tag.clone(), nav.current.clone());
    let files_vault = vault.clone();
    let empty_vault = vault.clone();
    let excerpt_vault = vault.clone();
    let nav_files = nav.clone();
    let nav_tags = nav.clone();
    let nav_clear = nav.clone();
    coat! {
        Node {
            width: Val::Px(260.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            border: UiRect::right(Val::Px(1.0)),
        }
        BackgroundColor(BG_SIDEBAR)
        BorderColor::all(BORDER)
        Children [
            Node {
                padding: UiRect::new(Val::Px(14.0), Val::Px(14.0), Val::Px(14.0), Val::Px(8.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
            }
            Children [
                Node { width: Val::Px(18.0), height: Val::Px(18.0), border_radius: BorderRadius::all(Val::Px(5.0)) }
                BackgroundColor(ACCENT)
                --
                Text("Bevsidian") @{ui_font(13.0, FontWeight::MEDIUM)} TextColor(TEXT)
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
                let nav_clear = nav_clear.clone();
                Node {
                    margin: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(0.0)),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    justify_content: JustifyContent::SpaceBetween,
                }
                BackgroundColor(TAG_BG)
                bevy::ui_widgets::Button
                ClickableSpan
                on(move |_: On<PointerClick>| nav_clear.clear_tag())
                Children [
                    Text($(format!("#{}", $tag.clone().unwrap_or_default()))) @{ui_font(12.0, FontWeight::MEDIUM)} TextColor(ACCENT)
                    --
                    Text("clear ×") @{ui_font(12.0, FontWeight::NORMAL)} TextColor(MUTED)
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
                #[key(*id)]
                for id in $(files_vault.search(&$query, $tag.as_deref())) {
                    @{file_row(id, nav_files.clone(), &excerpt_vault, &query.get_untracked(), current.clone())}
                }
                --
                if $(empty_vault.search(&$query, $tag.as_deref()).is_empty()) {
                    Text("No matching notes") @{ui_font(12.0, FontWeight::NORMAL)} TextColor(FAINT)
                    Node { margin: UiRect::all(Val::Px(8.0)) }
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
                for (name, count) in tags.clone() {
                    @{tag_chip(name, count, nav_tags.clone())}
                }
            ]
            --
            Text("Ctrl+O switch  ·  Ctrl+G graph") @{ui_font(12.0, FontWeight::NORMAL)} TextColor(FAINT)
            Node { margin: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(0.0), Val::Px(12.0)) }
        ]
    }
}

fn file_row(
    id: NoteId,
    nav: Nav,
    vault: &Vault,
    query: &str,
    current: bevy_radioactive::coat::Signal<NoteId>,
) -> impl Scene + use<> {
    let note = &vault.notes[id];
    let name = Text::new(note.name.clone());
    let folder = (!note.folder.is_empty()).then(|| {
        let text = Text::new(format!("{}/", note.folder));
        bsn! { text @{ui_font(11.0, FontWeight::NORMAL)} TextColor(FAINT) }
    });
    let excerpt = vault.excerpt(id, query).map(|excerpt| {
        let text = Text::new(excerpt);
        bsn! { text @{ui_font(11.5, FontWeight::NORMAL)} TextColor(MUTED) }
    });
    let open = nav.clone();
    coat! {
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            row_gap: Val::Px(2.0),
        }
        BackgroundColor($(if $current == id { BG_ACTIVE } else { Color::NONE }))
        HoverRow
        bevy::picking::hover::Hovered
        bevy::ui_widgets::Button
        TabIndex(0)
        ClickableSpan
        bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
        on(move |_: On<Activate>| open.open(id, None))
        Children [
            @{folder}
            --
            name @{ui_font(13.5, FontWeight::NORMAL)} TextColor($(if $current == id { TEXT } else { MUTED }))
            Pickable::IGNORE
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
    coat! {
        Node {
            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            column_gap: Val::Px(5.0),
            align_items: AlignItems::Center,
        }
        BackgroundColor($(if $active_tag.as_deref() == Some(tag.as_str()) { ACCENT.with_alpha(0.45) } else { TAG_BG }))
        bevy::ui_widgets::Button
        TabIndex(0)
        bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
        on(move |_: On<Activate>| nav.toggle_tag(&chip_tag))
        Children [
            label @{ui_font(12.0, FontWeight::MEDIUM)} TextColor(ACCENT) Pickable::IGNORE
            --
            count @{ui_font(11.0, FontWeight::NORMAL)} TextColor(MUTED) Pickable::IGNORE
        ]
    }
}

fn top_bar(nav: Nav, vault: Vault) -> impl Scene {
    let (back, forward, view, current) = (
        nav.back.clone(),
        nav.forward.clone(),
        nav.view.clone(),
        nav.current.clone(),
    );
    let (nav_back, nav_fwd, nav_note, nav_graph, nav_switch) = (
        nav.clone(),
        nav.clone(),
        nav.clone(),
        nav.clone(),
        nav.clone(),
    );
    coat! {
        Node {
            height: Val::Px(42.0),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(10.0)),
            column_gap: Val::Px(4.0),
            border: UiRect::bottom(Val::Px(1.0)),
        }
        BorderColor::all(BORDER)
        Children [
            @{icon_button()}
            on(move |_: On<Activate>| nav_back.go_back())
            Children [
                Text("‹") @{ui_font(20.0, FontWeight::NORMAL)}
                TextColor($(if $back.is_empty() { FAINT } else { TEXT }))
                Pickable::IGNORE
            ]
            --
            @{icon_button()}
            on(move |_: On<Activate>| nav_fwd.go_forward())
            Children [
                Text("›") @{ui_font(20.0, FontWeight::NORMAL)}
                TextColor($(if $forward.is_empty() { FAINT } else { TEXT }))
                Pickable::IGNORE
            ]
            --
            Text($(match $view {
                View::Note => vault.notes[$current].path.trim_end_matches(".md").replace('/', " / "),
                View::Graph => "Graph view".to_owned(),
            }))
            @{ui_font(13.0, FontWeight::MEDIUM)}
            TextColor(MUTED)
            Node { flex_grow: 1.0, justify_content: JustifyContent::Center, margin: UiRect::horizontal(Val::Px(8.0)) }
            --
            @{segment("Note")}
            BackgroundColor($(if $view == View::Note { BG_ACTIVE } else { Color::NONE }))
            on(move |_: On<Activate>| nav_note.show(View::Note))
            --
            @{segment("Graph")}
            BackgroundColor($(if $view == View::Graph { BG_ACTIVE } else { Color::NONE }))
            on(move |_: On<Activate>| nav_graph.show(View::Graph))
            --
            @{segment("Search")}
            on(move |_: On<Activate>| nav_switch.open_switcher())
        ]
    }
}

fn icon_button() -> impl Scene {
    bsn! {
        Node {
            width: Val::Px(28.0),
            height: Val::Px(28.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::NONE)
        bevy::ui_widgets::Button
        TabIndex(0)
        HoverRow
        bevy::picking::hover::Hovered
        bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
    }
}

fn segment(label: &str) -> impl Scene + use<> {
    let text = Text::new(label);
    bsn! {
        Node {
            padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
            border_radius: BorderRadius::all(Val::Px(5.0)),
            margin: UiRect::left(Val::Px(4.0)),
        }
        bevy::ui_widgets::Button
        TabIndex(0)
        bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
        Children [ text @{ui_font(12.5, FontWeight::MEDIUM)} TextColor(TEXT) Pickable::IGNORE ]
    }
}

fn main_area(nav: Nav, vault: Vault) -> impl Scene {
    let top = top_bar(nav.clone(), vault);
    let view = nav.view.clone();
    let note_key = {
        let current = nav.current.clone();
        move || current.get()
    };
    coat! {
        Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            min_width: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
        }
        Children [
            @{top}
            --
            match $view {
                View::Note => {
                    let note_key = note_key.clone();
                    Node {
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        overflow: Overflow::scroll_y(),
                        justify_content: JustifyContent::Center,
                    }
                    NotePane
                    Children [
                        Node {
                            width: Val::Percent(100.0),
                            max_width: Val::Px(780.0),
                            padding: UiRect::new(Val::Px(40.0), Val::Px(40.0), Val::Px(28.0), Val::Px(120.0)),
                            flex_direction: FlexDirection::Column,
                            align_self: AlignSelf::Start,
                        }
                        shard(note_key, |id: &NoteId, world: &World| note_scene(*id, world))
                    ]
                },
                View::Graph => {
                    Node { flex_grow: 1.0, flex_basis: Val::Px(0.0), overflow: Overflow::clip() }
                    GraphViewport
                },
            }
        ]
    }
}

fn note_scene(id: NoteId, world: &World) -> Box<dyn Scene> {
    let vault = world.resource::<Vault>().clone();
    let theme = world.resource::<MdTheme>().clone();
    let note = &vault.notes[id];
    let resolve = |link: &bevy_ui_commonmark::WikiLink| vault.resolve(link).is_some();
    let options = RenderOptions {
        theme: &theme,
        resolve: &resolve,
    };
    let body = markdown_scene(&note.doc, &options);
    // Obsidian-style inline title when the note has no level-1 heading.
    let title = note.doc.title.is_none().then(|| {
        let text = Text::new(note.name.clone());
        let size = theme.heading_size(1);
        bsn! { text @{ui_font(size, FontWeight::BOLD)} TextColor(TEXT) SelectableText Node { margin: UiRect::bottom(Val::Px(16.0)) } }
    });
    Box::new(bsn! {
        Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0) }
        Children [
            {title}
            --
            @{body}
        ]
    })
}

fn right_sidebar(nav: Nav) -> impl Scene {
    let side_key = {
        let current = nav.current.clone();
        move || current.get()
    };
    coat! {
        Node {
            width: Val::Px(250.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            border: UiRect::left(Val::Px(1.0)),
            overflow: Overflow::scroll_y(),
        }
        ScrollArea
        BackgroundColor(BG_SIDEBAR)
        BorderColor::all(BORDER)
        Children [
            Node { flex_direction: FlexDirection::Column, padding: UiRect::bottom(Val::Px(16.0)) }
            shard(side_key.clone(), move |id: &NoteId, world: &World| side_panel(*id, world, nav.clone()))
        ]
    }
}

fn side_panel(id: NoteId, world: &World, nav: Nav) -> Box<dyn Scene> {
    let vault = world.resource::<Vault>().clone();
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
                Children [ text @{ui_font(13.0, FontWeight::NORMAL)} TextColor(MUTED) Pickable::IGNORE ]
            }) as Box<dyn Scene>
        })
        .collect();

    let note_rows = |ids: &[NoteId]| -> Vec<Box<dyn Scene>> {
        ids.iter()
            .map(|&target| {
                let text = Text::new(vault.notes[target].name.clone());

                let nav = nav.clone();
                Box::new(bsn! {
                    @{side_row(Val::Px(12.0))}
                    on(move |_: On<Activate>| nav.open(target, None))
                    Children [ text @{ui_font(13.0, FontWeight::NORMAL)} TextColor(ACCENT) Pickable::IGNORE ]
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

    let empty = |label: &str| -> Option<Box<dyn Scene>> {
        let text = Text::new(label);

        Some(Box::new(bsn! {
            text @{ui_font(12.5, FontWeight::NORMAL)} TextColor(FAINT) Node { margin: UiRect::horizontal(Val::Px(12.0)) }
        }))
    };
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
    let stats = Text::new(format!(
        "{} words · {} backlinks",
        note.doc.plain_text().split_whitespace().count(),
        note.backlinks.len()
    ));

    let titles = (
        section_title("Outline"),
        section_title(&format!("Backlinks ({})", note.backlinks.len())),
        section_title("Outgoing links"),
        section_title("Tags"),
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
            stats @{ui_font(11.5, FontWeight::NORMAL)} TextColor(FAINT)
            Node { margin: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(18.0), Val::Px(0.0)) }
        ]
    })
}

fn side_row(indent: Val) -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::new(indent, Val::Px(10.0), Val::Px(4.0), Val::Px(4.0)),
            margin: UiRect::horizontal(Val::Px(6.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::NONE)
        HoverRow
        bevy::picking::hover::Hovered
        bevy::ui_widgets::Button
        TabIndex(0)
        bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
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
    coat! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
        }
        GlobalZIndex(100)
        Pickable::IGNORE
        Children [
            if $is_open {
                let close = close.clone();
                let nav_rows = nav_rows.clone();
                let names = names.clone();
                let vault = vault.clone();
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Start,
                    padding: UiRect::top(Val::Percent(12.0)),
                }
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55))
                on(move |click: On<PointerClick>| {
                    if click.entity == click.original_event_target() {
                        close.set(false);
                    }
                })
                Children [
                    Node {
                        width: Val::Px(560.0),
                        max_width: Val::Percent(92.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(6.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                    }
                    BackgroundColor(BG_SIDEBAR)
                    BorderColor::all(BORDER)
                    BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.5), Val::Px(0.0), Val::Px(8.0), Val::Px(0.0), Val::Px(24.0))
                    Children [
                        @FeathersTextInputContainer
                        Children [
                            @FeathersTextInput
                            SearchInput::Switcher
                            AutoFocus
                        ]
                        --
                        #[key(*entry)]
                        for entry in $(vault.search(&$query, None).into_iter().take(12).enumerate().collect::<Vec<_>>()) {
                            let (i, id) = entry;
                            let nav_rows = nav_rows.clone();
                            let names = names.clone();
                            Node {
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(5.0)),
                            }
                            BackgroundColor($(if $index == i { BG_ACTIVE } else { Color::NONE }))
                            bevy::ui_widgets::Button
                            bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
                            on(move |_: On<Activate>| {
                                nav_rows.open(id, None);
                                nav_rows.switcher.set(false);
                            })
                            Children [
                                Text({names.notes[id].name.clone()}) @{ui_font(14.0, FontWeight::NORMAL)} TextColor(TEXT) Pickable::IGNORE
                                --
                                Text({names.notes[id].path.clone()}) @{ui_font(11.5, FontWeight::NORMAL)} TextColor(FAINT) Pickable::IGNORE
                            ]
                        }
                        --
                        Text("↑↓ navigate   ↵ open   esc close") @{ui_font(11.5, FontWeight::NORMAL)} TextColor(FAINT)
                        Node { margin: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(4.0), Val::Px(0.0)) }
                    ]
                ]
            }
        ]
    }
}

//! Right-click context menu for notes, and the "Copied!" toast.
//!
//! Right-clicking a note opens a small menu at the pointer: Copy (when text is selected),
//! Select all, and Open link (over a link). It closes on any other press, on Escape or
//! after choosing an item.

use bevy::{
    picking::{
        events::{PointerPress, PointerScroll},
        hover::Hovered,
        pointer::PointerButton,
    },
    prelude::*,
    text::FontWeight,
    ui_widgets::Activate,
    window::PrimaryWindow,
};
use bevy_ui_commonmark::{CopiedBubble, LinkTarget, MdLink, MdLinkActivated, MdRoot};
use bevy_ui_textselect::{CopySelection, PreserveSelection, SelectAll, TextSelection};

use crate::ui::{HoverRow, palette};

pub fn plugin(app: &mut App) {
    app.add_observer(open_menu)
        .add_observer(close_on_press)
        .add_observer(close_on_scroll)
        .add_observer(copied_toast)
        .add_systems(Update, close_on_escape);
}

/// Root of the open context menu.
#[derive(Component, Clone, Copy, Debug, Default)]
struct ContextMenu;

const MENU_WIDTH: f32 = 190.0;
const ITEM_HEIGHT: f32 = 34.0;

#[derive(Clone)]
enum Item {
    Copy,
    SelectAll(Option<Entity>),
    OpenLink(LinkTarget, Entity),
}

fn open_menu(
    press: On<PointerPress>,
    parents: Query<&ChildOf>,
    roots: Query<(), With<MdRoot>>,
    links: Query<&MdLink>,
    menus: Query<Entity, With<ContextMenu>>,
    window: Query<&Window, With<PrimaryWindow>>,
    selection: Res<TextSelection>,
    mut commands: Commands,
) {
    if press.button != PointerButton::Secondary || press.entity != press.original_event_target() {
        return;
    }
    let target = press.entity;
    // Only inside a rendered note.
    let Some(root) = std::iter::once(target)
        .chain(parents.iter_ancestors(target))
        .find(|e| roots.contains(*e))
    else {
        return;
    };
    for menu in &menus {
        commands.entity(menu).despawn();
    }

    let mut items = Vec::new();
    if !selection.is_collapsed() {
        items.push(("Copy", Item::Copy));
    }
    items.push(("Select all", Item::SelectAll(Some(root))));
    if let Ok(link) = links.get(target) {
        items.push(("Open link", Item::OpenLink(link.0.clone(), target)));
    }

    // Keep the menu inside the window.
    let size = Vec2::new(MENU_WIDTH, items.len() as f32 * ITEM_HEIGHT + 10.0);
    let bounds = window.single().map_or(Vec2::splat(f32::MAX), Window::size);
    let position = press
        .pointer
        .position
        .min(bounds - size - 4.0)
        .max(Vec2::splat(4.0));

    let menu = commands
        .spawn((
            ContextMenu,
            PreserveSelection,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                width: Val::Px(MENU_WIDTH),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(palette::BG_SIDEBAR),
            BorderColor::all(palette::BORDER),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.5),
                Val::Px(0.0),
                Val::Px(6.0),
                Val::Px(0.0),
                Val::Px(18.0),
            ),
            GlobalZIndex(150),
        ))
        .id();
    for (label, item) in items {
        commands
            .spawn((
                Node {
                    height: Val::Px(ITEM_HEIGHT),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                HoverRow,
                Hovered::default(),
                bevy::ui_widgets::Button,
                bevy::picking::cursor::EntityCursor::System(
                    bevy::window::SystemCursorIcon::Pointer,
                ),
                ChildOf(menu),
                children![(
                    Text::new(label),
                    TextFont {
                        font: bevy::text::FontSource::families("Inter, sans-serif"),
                        font_size: 14.0.into(),
                        weight: FontWeight::NORMAL,
                        ..default()
                    },
                    TextColor(palette::TEXT),
                    Pickable::IGNORE,
                )],
            ))
            .observe(
                move |_: On<Activate>,
                      menus: Query<Entity, With<ContextMenu>>,
                      mut commands: Commands| {
                    match &item {
                        Item::Copy => commands.trigger(CopySelection),
                        Item::SelectAll(group) => commands.trigger(SelectAll { group: *group }),
                        Item::OpenLink(target, span) => commands.trigger(MdLinkActivated {
                            target: target.clone(),
                            span: *span,
                        }),
                    }
                    for menu in &menus {
                        commands.entity(menu).despawn();
                    }
                },
            );
    }
}

/// Any press outside the menu closes it (presses on items activate them first).
fn close_on_press(
    press: On<PointerPress>,
    parents: Query<&ChildOf>,
    menus: Query<Entity, With<ContextMenu>>,
    mut commands: Commands,
) {
    if press.button == PointerButton::Secondary || press.entity != press.original_event_target() {
        return;
    }
    let inside = std::iter::once(press.entity)
        .chain(parents.iter_ancestors(press.entity))
        .any(|e| menus.contains(e));
    if !inside {
        for menu in &menus {
            commands.entity(menu).despawn();
        }
    }
}

fn close_on_scroll(
    _: On<PointerScroll>,
    menus: Query<Entity, With<ContextMenu>>,
    mut commands: Commands,
) {
    for menu in &menus {
        commands.entity(menu).despawn();
    }
}

fn close_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    menus: Query<Entity, With<ContextMenu>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Escape) {
        for menu in &menus {
            commands.entity(menu).despawn();
        }
    }
}

/// Root of the "Copied!" toast.
#[derive(Component, Clone, Copy, Debug, Default)]
struct Toast;

/// Shows "Copied!" briefly at the bottom when a selection is copied from the menu or the
/// touch selection bar.
fn copied_toast(
    _: On<CopySelection>,
    selection: Res<TextSelection>,
    time: Res<Time<Real>>,
    toasts: Query<Entity, With<Toast>>,
    mut commands: Commands,
) {
    if selection.is_collapsed() {
        return;
    }
    let until = time.elapsed_secs_f64() + 1.2;
    if let Some(toast) = toasts.iter().next() {
        commands.entity(toast).insert(CopiedBubble { until });
        return;
    }
    commands.spawn((
        Toast,
        CopiedBubble { until },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            bottom: Val::Px(84.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(160),
        Pickable::IGNORE,
        children![(
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(7.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.26, 0.26, 0.28)),
            Pickable::IGNORE,
            children![(
                Text::new("Copied!"),
                TextFont {
                    font: bevy::text::FontSource::families("Inter, sans-serif"),
                    font_size: 14.0.into(),
                    weight: FontWeight::MEDIUM,
                    ..default()
                },
                TextColor(palette::TEXT),
                Pickable::IGNORE,
            )],
        )],
    ));
}

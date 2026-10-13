//! Right-click context menu for notes, and the "Copied!" toast.
//!
//! Right-clicking a note opens a small menu at the pointer: Copy (when text is selected),
//! Select all, and Open link (over a link). It closes on any other press, on Escape or
//! after choosing an item.

use bevy::{
    picking::{
        events::{PointerPress, PointerScroll},
        pointer::PointerButton,
    },
    prelude::*,
    text::FontWeight,
    ui_widgets::Activate,
    window::PrimaryWindow,
};
use bevy_ui_commonmark::{CopiedBubble, LinkTarget, MdLink, MdLinkActivated, MdRoot};
use bevy_ui_textselect::{CopySelection, PreserveSelection, SelectAll, TextSelection};

use bevsidian_ui::{color, label, layer, popover, radius, size, toast};

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

const MENU_WIDTH: f32 = 196.0;
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

    let (left, top) = (position.x, position.y);
    let menu = commands
        .spawn_scene(bsn! {
            @{popover()}
            ContextMenu
            PreserveSelection
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(MENU_WIDTH),
                flex_direction: FlexDirection::Column,
            }
            GlobalZIndex(layer::MENU)
        })
        .id();
    for (text, item) in items {
        let text = label(text, size::BASE, FontWeight::NORMAL, color::FOREGROUND);
        commands
            .spawn_scene(bsn! {
                @{bevsidian_ui::item()}
                Node { height: Val::Px(ITEM_HEIGHT), border_radius: BorderRadius::all(Val::Px(radius::MD)) }
                Children [ @{text} ]
            })
            .insert(ChildOf(menu))
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
    let toast = toast("Copied!");
    commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                bottom: Val::Px(84.0),
                justify_content: JustifyContent::Center,
            }
            GlobalZIndex(layer::TOAST)
            Pickable::IGNORE
            Children [ @{toast} ]
        })
        .insert((Toast, CopiedBubble { until }));
}

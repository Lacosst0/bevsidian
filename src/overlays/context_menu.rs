//! The right-click menu of a note: Open link (over a link), Copy (with a selection) and Select
//! all. It opens at the pointer, inside the window, and closes on any other press, a scroll,
//! Escape, or after choosing an item.

use bevy::{
    picking::{
        events::{PointerPress, PointerScroll},
        pointer::PointerButton,
    },
    prelude::*,
    ui_widgets::Activate,
    window::PrimaryWindow,
};
use bevy_ui_commonmark::{LinkTarget, MdLink, MdLinkActivated, MdRoot};
use bevy_ui_textselect::{CopySelection, PreserveSelection, SelectAll, TextSelection};

use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(open_menu)
        .add_observer(close_on_press)
        .add_observer(close_on_scroll)
        .add_systems(Update, close_on_escape);
}

const WIDTH: f32 = 224.0;
const ITEM_HEIGHT: f32 = 32.0;

/// What a menu item does.
#[derive(Clone)]
pub enum Action {
    OpenLink(LinkTarget, Entity),
    Copy,
    SelectAll(Entity),
}

/// The open menu: a popover at `at` (window coordinates) with its `actions`, a separator after
/// Open link.
#[derive(SceneComponent, Default, Clone)]
#[scene(ContextMenuProps)]
pub struct ContextMenu;

#[derive(Default)]
pub struct ContextMenuProps {
    pub at: Vec2,
    pub actions: Vec<Action>,
}

impl ContextMenu {
    fn scene(props: ContextMenuProps) -> impl Scene {
        let items: Vec<Box<dyn Scene>> = props
            .actions
            .iter()
            .flat_map(|action| {
                let (icon, text, keys) = match action {
                    Action::OpenLink(..) => ("external-link", "Open link", None),
                    Action::Copy => ("copy", "Copy", Some("Ctrl C")),
                    Action::SelectAll(_) => ("text-select", "Select all", Some("Ctrl A")),
                };
                let run = action.clone();
                let item = Box::new(bsn! {
                    @{menu_item(icon, text, keys)}
                    on(move |_: On<Activate>, menus: Query<Entity, With<ContextMenu>>, mut commands: Commands| {
                        match &run {
                            Action::OpenLink(target, span) => commands.trigger(MdLinkActivated { target: target.clone(), span: *span }),
                            Action::Copy => commands.trigger(CopySelection),
                            Action::SelectAll(root) => commands.trigger(SelectAll { group: Some(*root) }),
                        }
                        for menu in &menus {
                            commands.entity(menu).despawn();
                        }
                    })
                }) as Box<dyn Scene>;
                let separator = matches!(action, Action::OpenLink(..)) && props.actions.len() > 1;
                std::iter::once(item).chain(separator.then(|| Box::new(menu_separator()) as Box<dyn Scene>))
            })
            .collect();
        let (left, top) = (props.at.x, props.at.y);
        bsn! {
            @{popover()}
            PreserveSelection
            Node { position_type: PositionType::Absolute, left: px(left), top: px(top), width: px(WIDTH) }
            GlobalZIndex(layer::MENU)
            Children [ {items} ]
        }
    }
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

    let mut actions = Vec::new();
    if let Ok(link) = links.get(target) {
        actions.push(Action::OpenLink(link.0.clone(), target));
    }
    if !selection.is_collapsed() {
        actions.push(Action::Copy);
    }
    actions.push(Action::SelectAll(root));

    // Keep the menu inside the window.
    let height = actions.len() as f32 * ITEM_HEIGHT + 18.0;
    let bounds = window.single().map_or(Vec2::splat(f32::MAX), Window::size);
    let at = press
        .pointer
        .position
        .min(bounds - Vec2::new(WIDTH, height) - 4.0)
        .max(Vec2::splat(4.0));
    commands.spawn_scene(bsn! { @ContextMenu { @at: at, @actions: actions } });
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

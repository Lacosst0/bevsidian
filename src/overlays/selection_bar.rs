//! The floating toolbar shown while text is selected on a touch screen: Copy, Select all, and
//! a close button that clears the selection.

use bevy::{prelude::*, ui_widgets::Activate};
use bevy_radioactive::view::{Watch, component, view};
use bevy_ui_textselect::{
    ClearSelection, CopySelection, PreserveSelection, SelectAll, TextSelection,
};

use bevsidian_core::nav::Nav;
use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, track_selection);
}

fn track_selection(selection: Res<TextSelection>, mut nav: ResMut<Nav>) {
    if selection.is_changed() && nav.has_selection == selection.is_collapsed() {
        nav.has_selection = !selection.is_collapsed();
    }
}

#[component]
pub fn selection_bar(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), bottom: px(24), justify_content: JustifyContent::Center }
        GlobalZIndex(layer::TOOLBAR)
        Pickable::IGNORE
        PreserveSelection
        Children [
            if $nav.has_selection && $nav.touch {
                @{popover()}
                Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(2) }
                Children [
                    @{icon_text_button(Variant::Ghost, Size::Lg, "copy", "Copy")}
                    on(|_: On<Activate>, mut commands: Commands| {
                        commands.trigger(CopySelection);
                        commands.trigger(ClearSelection);
                    })
                    --
                    @{icon_text_button(Variant::Ghost, Size::Lg, "text-select", "Select all")}
                    on(|_: On<Activate>, mut commands: Commands| commands.trigger(SelectAll::default()))
                    --
                    @{vertical_separator()}
                    --
                    @{icon_button(Variant::Ghost, Size::IconLg, "x")}
                    on(|_: On<Activate>, mut commands: Commands| commands.trigger(ClearSelection))
                ]
            }
        ]
    }
}

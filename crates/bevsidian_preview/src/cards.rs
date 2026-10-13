//! The cards view: a preview world's entities and their component values.

use bevy::prelude::*;

use bevsidian_ui::{ComponentValue, EntityCard, inset, muted};

use crate::{
    WorldPreview,
    runtime::{Card, PreviewRuntime},
};

/// Where the cards go.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct CardsView;

pub(crate) fn cards_frame(preview: WorldPreview) -> impl Scene {
    bsn! {
        @{inset()}
        preview
        CardsView
        Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: px(12), row_gap: px(12) }
    }
}

/// Respawns the cards of the previews that changed.
pub(crate) fn show_cards(
    previews: Query<(Entity, &PreviewRuntime), (Changed<PreviewRuntime>, With<CardsView>)>,
    mut commands: Commands,
) {
    for (host, preview) in &previews {
        commands.entity(host).despawn_children();
        spawn_cards(&mut commands, host, &preview.cards, &preview.placeholder);
    }
}

/// The entity cards, or "No entities" over the invisible `placeholder` cards (keeping the
/// size the preview will have).
fn spawn_cards(commands: &mut Commands, parent: Entity, cards: &[Card], placeholder: &[Card]) {
    let empty = cards.is_empty();
    for card in if empty { placeholder } else { cards } {
        let title = card.title.clone();
        let components: Vec<ComponentValue> = card
            .rows
            .iter()
            .map(|row| ComponentValue::parse(row))
            .collect();
        let mut spawned =
            commands.spawn_scene(bsn! { @EntityCard { @title: title, @components: components } });
        spawned.insert(ChildOf(parent));
        if empty {
            spawned.insert(Visibility::Hidden);
        }
    }
    if empty {
        commands
            .spawn_scene(bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                Children [ @{muted("No entities")} ]
            })
            .insert(ChildOf(parent));
    }
}

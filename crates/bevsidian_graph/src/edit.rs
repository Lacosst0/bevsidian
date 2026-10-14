//! Editing the map in the app (the pencil under the zoom buttons): every note shows, a drag
//! moves a note or a map object, a click chooses one for the edit bar (its shape and size,
//! or deleting an object), and landmarks, labels and nebulae can be added. Every change is
//! written into the map note's ```` ```map ```` block (only the line that changed) and saved
//! like any edit of a note: the app hears [`MapEdited`] and stores the note.

use bevy::prelude::*;

use bevsidian_core::{
    map::{self, MapObject, Place, Scale, Shape},
    nav::Nav,
    vault::{NoteId, Vault},
};

use crate::{GraphCamera, GraphNode, GraphState, world::MapThing};

/// The map note was changed by the editor: the app should store it.
#[derive(Event, Clone, Copy, Debug)]
pub struct MapEdited {
    pub note: NoteId,
}

/// A change asked for in the edit bar.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EditRequest {
    /// The chosen note's shape (`None`: the one its kind gives it).
    Shape(Option<Shape>),
    Scale(Scale),
    /// Adds an object in the middle of the view.
    Add(MapObject),
    /// Deletes the chosen object.
    Delete,
}

/// What to choose again once the map is rebuilt after a save.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pick {
    Note(NoteId),
    Object(usize),
}

/// The map object under `world`, nearest first.
pub(crate) fn thing_at(
    world: Vec2,
    things: &Query<(Entity, &GlobalTransform, &MapThing)>,
) -> Option<Entity> {
    things
        .iter()
        .map(|(e, t, thing)| (e, t.translation().truncate().distance(world), thing))
        .filter(|(_, d, thing)| *d <= thing.reach())
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(e, ..)| e)
}

/// Applies drops and edit-bar requests to the map note, and keeps the choice across the
/// rebuild a save causes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn edit_map(
    mut commands: Commands,
    nav: Res<Nav>,
    vault: Res<Vault>,
    mut state: ResMut<GraphState>,
    nodes: Query<(Entity, &GraphNode, &Transform), Without<MapThing>>,
    things: Query<(Entity, &MapThing, &Transform), Without<GraphNode>>,
    camera: Query<&Transform, (With<GraphCamera>, Without<GraphNode>, Without<MapThing>)>,
    mut pending: Local<Option<Pick>>,
    mut was_editing: Local<bool>,
) {
    let editing = nav.map_edit.get_untracked();
    if editing != *was_editing {
        *was_editing = editing;
        state.editing = editing;
        state.selected = None;
        state.refold = true;
    }
    // Choose again what was chosen before the save rebuilt the map.
    if let Some(pick) = *pending
        && state.selected.is_none()
    {
        state.selected = match pick {
            Pick::Note(id) => nodes.iter().find(|(_, n, _)| n.note == id).map(|(e, ..)| e),
            Pick::Object(index) => things
                .iter()
                .find(|(_, t, _)| t.index == index)
                .map(|(e, ..)| e),
        };
        if state.selected.is_some() {
            *pending = None;
        }
    }
    // A nebula being dragged redraws the sky as it goes.
    if let Some(dragged) = state.dragging
        && things
            .get(dragged)
            .is_ok_and(|(_, t, _)| matches!(t.object, MapObject::Nebula { .. }))
    {
        state.sky_dirty = true;
    }
    let dropped = state.dropped.take();
    let request = state.edit_request.take();
    if !editing {
        return;
    }
    let data = vault.get_untracked();
    let Some(world_map) = &data.map else {
        return;
    };
    let source = &data.notes[world_map.note].source;
    let place_of = |entity: Entity| -> Option<(NoteId, Place)> {
        let (_, node, transform) = nodes.get(entity).ok()?;
        let name = data.notes[node.note].name.clone();
        let mut place = world_map.place(&name).cloned().unwrap_or(Place {
            name,
            at: Vec2::ZERO,
            shape: None,
            scale: Scale::Normal,
        });
        place.at = transform.translation.truncate();
        Some((node.note, place))
    };
    let object_of = |entity: Entity| -> Option<(usize, MapObject)> {
        let (_, thing, transform) = things.get(entity).ok()?;
        let mut object = thing.object.clone();
        object.set_at(transform.translation.truncate());
        Some((thing.index, object))
    };
    let mut edited: Option<(String, Option<Pick>)> = None;
    if let Some(entity) = dropped {
        if let Some((id, place)) = place_of(entity) {
            edited = Some((map::set_place(source, &place), Some(Pick::Note(id))));
        } else if let Some((index, object)) = object_of(entity) {
            edited = Some((
                map::set_object(source, index, &object),
                Some(Pick::Object(index)),
            ));
        }
    }
    let selected = state.selected;
    match request {
        Some(EditRequest::Shape(shape)) => {
            if let Some((id, mut place)) = selected.and_then(place_of) {
                place.shape = shape;
                edited = Some((map::set_place(source, &place), Some(Pick::Note(id))));
            }
        }
        Some(EditRequest::Scale(scale)) => {
            if let Some((id, mut place)) = selected.and_then(place_of) {
                place.scale = scale;
                edited = Some((map::set_place(source, &place), Some(Pick::Note(id))));
            } else if let Some((index, mut object)) = selected.and_then(object_of) {
                *object.size_mut() = (object.default_size() * scale.factor()).round();
                edited = Some((
                    map::set_object(source, index, &object),
                    Some(Pick::Object(index)),
                ));
            }
        }
        Some(EditRequest::Add(mut object)) => {
            let center = camera
                .single()
                .map(|t| t.translation.truncate())
                .unwrap_or_default();
            object.set_at(center);
            edited = Some((
                map::add_object(source, &object),
                Some(Pick::Object(world_map.objects.len())),
            ));
        }
        Some(EditRequest::Delete) => {
            if let Some((index, _)) = selected.and_then(object_of) {
                edited = Some((map::remove_object(source, index), None));
            }
        }
        None => {}
    }
    if let Some((new_source, pick)) = edited
        && vault.save(world_map.note, &new_source)
    {
        *pending = pick;
        state.selected = None;
        commands.trigger(MapEdited {
            note: world_map.note,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevsidian_core::vault::VaultData;

    #[test]
    fn a_drop_while_editing_saves_the_new_place() {
        let vault = Vault::new(VaultData::from_sources([
            ("Hub.md", "---\nhub: true\n---\n# Hub"),
            ("A.md", "# A\n[[Hub]]"),
            (
                "Map.md",
                "---\nmap: true\n---\n# Map\n```map\n# keep me\nA @ 100, 0 diamond\n```\n",
            ),
        ]));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(Nav::new(0))
            .insert_resource(vault.clone())
            .init_resource::<GraphState>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_systems(Update, (crate::spawn_nodes, edit_map).chain());
        app.update();
        let node = {
            let world = app.world_mut();
            let mut q = world.query::<(Entity, &GraphNode)>();
            let data = vault.get_untracked();
            q.iter(world)
                .find(|(_, n)| data.notes[n.note].name == "A")
                .unwrap()
                .0
        };
        app.world().resource::<Nav>().map_edit.set(true);
        app.world_mut()
            .get_mut::<Transform>(node)
            .unwrap()
            .translation = Vec3::new(-40.0, 25.0, 1.0);
        app.world_mut().resource_mut::<GraphState>().dropped = Some(node);
        app.update();
        let data = vault.get_untracked();
        let map = data.map.as_ref().unwrap();
        let place = map.place("A").unwrap();
        assert_eq!(place.at, Vec2::new(-40.0, 25.0));
        assert_eq!(place.shape, Some(Shape::Diamond), "the shape is kept");
        assert!(
            data.notes[map.note].source.contains("# keep me"),
            "and the comment"
        );
    }
}

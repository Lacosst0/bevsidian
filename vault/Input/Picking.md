---
icon: mouse-pointer
tags: [input]
---
# Picking

"What is under the pointer?" Bevy answers it for sprites, meshes and UI, and turns it into **events on those entities**. #input

## Click, over, out, drag

Observe pointer events like any [[Events and observers|event]]. Clicks arrive on the entity that was hit:

| Event | When |
|:--|:--|
| `PointerOver` / `PointerOut` | the pointer enters / leaves |
| `PointerPress` / `PointerRelease` | button down / up |
| `PointerClick` | pressed and released on it |
| `PointerDragStart`, `PointerDrag`, `PointerDragEnd` | dragging |
| `PointerMove`, `PointerScroll` | moving or scrolling over it |

Try it. Each card is a sprite with three observers: pick one up and it rises above the others, drag it, drop it.

```rust example=dragging
use bevy::prelude::*;

/// The highest z handed out: whatever you pick up goes on top.
#[derive(Resource, Default)]
struct Top(f32);

#[derive(Component)]
struct Card;

const COLOURS: [Color; 3] = [
    Color::srgb(0.95, 0.45, 0.4),
    Color::srgb(0.55, 0.85, 0.45),
    Color::srgb(0.6, 0.5, 0.95),
];

fn spawn_cards(mut commands: Commands) {
    for i in 0..3 {
        commands.spawn((
            Card,
            Sprite::from_color(COLOURS[i], Vec2::new(50.0, 70.0)),
            Transform::from_xyz(-70.0 + 70.0 * i as f32, 0.0, i as f32),
        ));
    }
}

fn lift(
    start: On<PointerDragStart>,
    mut top: ResMut<Top>,
    mut cards: Query<&mut Transform, With<Card>>,
) {
    if let Ok(mut transform) = cards.get_mut(start.entity) {
        top.0 += 1.0;
        transform.translation.z = 10.0 + top.0;
        transform.scale = Vec3::splat(1.1);
    }
}

fn drag(drag: On<PointerDrag>, mut cards: Query<&mut Transform, With<Card>>) {
    if let Ok(mut transform) = cards.get_mut(drag.entity) {
        // Screen y points down, world y up.
        transform.translation += Vec3::new(drag.delta.x, -drag.delta.y, 0.0);
    }
}

fn drop(end: On<PointerDragEnd>, mut cards: Query<&mut Transform, With<Card>>) {
    if let Ok(mut transform) = cards.get_mut(end.entity) {
        transform.scale = Vec3::ONE;
    }
}
```

The event carries useful fields: `drag.delta` (movement since last event), `drag.distance` (since the start), `click.hit.position` (world position of the hit), `pointer.position` (screen).

## Attach to one entity or listen to all

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands) {
    commands
        .spawn(Sprite::from_color(Color::WHITE, Vec2::splat(50.0)))
        .observe(|over: On<PointerOver>, mut sprites: Query<&mut Sprite>| {
            sprites.get_mut(over.entity).unwrap().color = Color::srgb(1.0, 0.8, 0.2);
        })
        .observe(|out: On<PointerOut>, mut sprites: Query<&mut Sprite>| {
            sprites.get_mut(out.entity).unwrap().color = Color::WHITE;
        });
}
```

The hover highlight above only touches the entity it is attached to. A *global* `.add_observer(...)` hears every entity's events and checks `event.entity`.

## Bubbling

Pointer events **bubble** up to the parent, like the DOM. A click on a button's label also reaches the button. Call `event.propagate(false)` to stop it.

## Backends: what can be hit

| Target | Needs |
|:--|:--|
| UI nodes | on by default |
| Sprites | on by default with the `picking` features; transparent pixels can be ignored |
| 3D meshes | add `MeshPickingPlugin` |
| 2D meshes | `MeshPickingPlugin` as well |

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))   // 3D / 2D mesh picking
        .run();
}
```

Block clicks with `Pickable::IGNORE` on an entity that should be see-through to the pointer, or `Pickable { should_block_lower: false, is_hoverable: true }`.

## Pick in a 3D game

Click on a unit to select it, then right-click the ground to order it:

```rust
use bevy::prelude::*;

#[derive(Component)] struct Unit;
#[derive(Component)] struct Selected;

fn select(click: On<PointerClick>, units: Query<(), With<Unit>>, mut commands: Commands) {
    if units.contains(click.entity) {
        commands.entity(click.entity).insert(Selected);
    }
}
```

## Next

[[Input mapping]]: put all the input types behind game actions.

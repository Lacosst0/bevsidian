---
icon: backpack
tags: [gameplay, ecs, intermediate]
crates: serde={ version = "1", features = ["derive"] }
---
# Inventory and items

An inventory looks like a UI problem, but it is a **data** problem. Get the data right and the UI, saving and multiplayer follow. #gameplay #ecs

## Two things called "item"

| | What it is | Lives |
|:--|:--|:--|
| **Item definition** | "Health Potion: heals 25, stacks to 10" | once, in data (an enum, a RON file, an asset) |
| **Item instance** | "I am carrying 3 of them" | per owner, in a component |

Mixing them up gives the classic bugs: editing one potion changes every potion. Keep definitions read-only, and put only a *reference* and a *count* in the inventory.

## Definitions as data

Start with an enum; move to files when there are dozens.

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Item {
    Potion,
    Key,
    Sword,
}

struct ItemInfo {
    name: &'static str,
    max_stack: u32,
}

impl Item {
    fn info(self) -> ItemInfo {
        match self {
            Item::Potion => ItemInfo { name: "Health Potion", max_stack: 10 },
            Item::Key => ItemInfo { name: "Key", max_stack: 99 },
            Item::Sword => ItemInfo { name: "Sword", max_stack: 1 },
        }
    }
}
```

When designers should edit them without recompiling, load the same table from a file as an asset ([[Custom assets]], [[bevy_common_assets]]):

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ItemDef {
    id: String,
    name: String,
    max_stack: u32,
    heal: Option<u32>,
}
```

## The simplest inventory: a component

A fixed number of slots, each empty or a stack:

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Item { Potion, Key, Sword }

impl Item {
    fn max_stack(self) -> u32 {
        match self { Item::Potion => 10, Item::Key => 99, Item::Sword => 1 }
    }
}

#[derive(Clone, Copy, Debug)]
struct Stack { item: Item, count: u32 }

#[derive(Component)]
struct Inventory { slots: Vec<Option<Stack>> }

impl Inventory {
    fn new(size: usize) -> Self {
        Self { slots: vec![None; size] }
    }

    /// Adds as many as fit; returns how many did NOT fit.
    fn add(&mut self, item: Item, mut count: u32) -> u32 {
        // 1. top up stacks of the same item
        for stack in self.slots.iter_mut().flatten() {
            if stack.item == item && stack.count < item.max_stack() {
                let room = item.max_stack() - stack.count;
                let moved = room.min(count);
                stack.count += moved;
                count -= moved;
            }
        }
        // 2. start new stacks in empty slots
        for slot in self.slots.iter_mut().filter(|s| s.is_none()) {
            if count == 0 {
                break;
            }
            let moved = item.max_stack().min(count);
            *slot = Some(Stack { item, count: moved });
            count -= moved;
        }
        count
    }

    /// Removes `count` if the inventory has that many; otherwise changes nothing.
    fn remove(&mut self, item: Item, count: u32) -> bool {
        let have: u32 = self.slots.iter().flatten().filter(|s| s.item == item).map(|s| s.count).sum();
        if have < count {
            return false;
        }
        let mut left = count;
        for slot in &mut self.slots {
            if let Some(stack) = slot && stack.item == item {
                let taken = stack.count.min(left);
                stack.count -= taken;
                left -= taken;
                if stack.count == 0 {
                    *slot = None;
                }
            }
        }
        true
    }
}
```

Notice `add` returns the **leftover**. The caller decides what a full bag means: leave the item on the ground, or show "Bag full". The data type does not know about the world.

## Picking things up

Do the world part with a message: a pickup says "this entity touched that one", and one system moves the item.

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Item { Potion }
impl Item { fn max_stack(self) -> u32 { 10 } }

#[derive(Component)]
struct Inventory { slots: Vec<Option<(Item, u32)>> }
impl Inventory {
    fn add(&mut self, item: Item, count: u32) -> u32 { let _ = (item, count); 0 }
}

/// An item lying in the world.
#[derive(Component)]
struct Pickup { item: Item, count: u32 }

#[derive(Message)]
struct Touched { by: Entity, pickup: Entity }

fn collect(
    mut touched: MessageReader<Touched>,
    mut inventories: Query<&mut Inventory>,
    mut pickups: Query<&mut Pickup>,
    mut commands: Commands,
) {
    for touch in touched.read() {
        let (Ok(mut bag), Ok(mut pickup)) = (inventories.get_mut(touch.by), pickups.get_mut(touch.pickup)) else {
            continue;
        };
        pickup.count = bag.add(pickup.item, pickup.count);
        if pickup.count == 0 {
            commands.entity(touch.pickup).despawn();     // all taken
        }
    }
}
```

If the bag is full, `count` stays above zero and the item stays on the ground, with the remainder.

## Equipment and other owners: relationships

When an item must be its **own entity** (it has durability, enchantments, a model in the player's hand), model carrying as a **relationship** ([[Relationships and hierarchy]]). Bevy keeps both sides in sync:

```rust
use bevy::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = Carrying)]
struct CarriedBy(Entity);

#[derive(Component)]
#[relationship_target(relationship = CarriedBy)]
struct Carrying(Vec<Entity>);

#[derive(Component)]
struct Durability(u32);

fn give(mut commands: Commands, player: Single<Entity, With<Name>>) {
    commands.spawn((Name::new("Rusty Sword"), Durability(30), CarriedBy(*player)));
}

fn list(players: Query<(&Name, &Carrying)>, items: Query<&Name>) {
    for (player, carrying) in &players {
        for item in carrying.0.iter().filter_map(|e| items.get(*e).ok()) {
            info!("{player} carries {item}");
        }
    }
}
```

Dropping an item is removing `CarriedBy` and adding a `Transform`. Despawning the player despawns what they carry only if you ask for it (`linked_spawn`); otherwise items are left in the world.

Which to choose?

| | Slots component | Item entities |
|:--|:--|:--|
| items are identical (potions, coins) | **best** | wasteful |
| items are unique (each sword differs) | awkward | **best** |
| saving the game | trivial | needs entity mapping ([[Saving and loading]]) |
| networking | small to send | replicate the entities |

Many games use both: stackable goods in slots, unique gear as entities.

## The UI

Draw from the data, never store state in the UI. A system rebuilds the slots when the inventory changes:

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Inventory { slots: Vec<Option<(String, u32)>> }

#[derive(Component)]
struct InventoryPanel;

fn show_inventory(
    bag: Single<&Inventory, Changed<Inventory>>,
    panel: Single<Entity, With<InventoryPanel>>,
    mut commands: Commands,
) {
    commands.entity(*panel).despawn_children();
    for slot in &bag.slots {
        let label = match slot {
            Some((name, count)) => format!("{name} x{count}"),
            None => String::new(),
        };
        commands.spawn_scene(bsn! {
            Node { width: px(96), height: px(48), border: UiRect::all(px(1)) }
            BorderColor::all(Color::WHITE)
            Children [ Text({label}) ]
        }).insert(ChildOf(*panel));
    }
}
```

`Changed<Inventory>` makes this run only when something moved. For a live-updating UI without systems, see [[bevy_radioactive]]'s `watch`.

## Next

[[Saving and loading]] to keep it, [[Dialogue]] for the shopkeeper, [[Buttons and widgets]] for clicking a slot.

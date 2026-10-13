//! `ECS.md`: a player with health, a system that heals it and an observer that damages it.

use bevy::prelude::*;

use crate::example::{Example, Function, Setup, View, observer, startup, update};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[],
    functions: FUNCTIONS,
    also: &[("attack", "on_damage")],
    view: View::Cards { fps: 4.0 },
};

const FUNCTIONS: &[Function] = &[
    startup!(spawn_player),
    update!(regenerate),
    update!(attack),
    observer!(on_damage),
];

fn setup(_: &World) -> Setup {
    Box::new(|world| {
        let registry = world.resource::<AppTypeRegistry>();
        let mut registry = registry.write();
        registry.register::<Health>();
        registry.register::<Player>();
        registry.register::<Transform>();
    })
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Health(pub u32);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player, Health(50), Transform::default()));
}

fn regenerate(_time: Res<Time>, mut query: Query<&mut Health, With<Player>>) {
    for mut health in &mut query {
        health.0 = (health.0 + 1).min(100);
    }
}

#[derive(EntityEvent)]
struct Damage {
    entity: Entity,
    amount: u32,
}

fn on_damage(damage: On<Damage>, mut health: Query<&mut Health>) {
    if let Ok(mut health) = health.get_mut(damage.entity) {
        health.0 = health.0.saturating_sub(damage.amount);
    }
}

fn attack(players: Query<Entity, With<Player>>, mut commands: Commands) {
    for player in &players {
        commands.trigger(Damage {
            entity: player,
            amount: 10,
        });
    }
}

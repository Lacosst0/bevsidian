//! `Audio.md`: four pads that play the reader's sounds ("Make it yours") when clicked.

use bevy::{audio::AudioSource, prelude::*};

use super::stage::{self, spawn_sky};
use crate::{
    example::{Example, Setup, observer, startup, update},
    sound::Slot,
    yours::Yours,
};

pub static EXAMPLE: Example = Example {
    setup,
    base: &[startup!(spawn_sky)],
    functions: &[startup!(spawn_pads), observer!(play), update!(spring_back)],
    tick: None,
    actions: &[],
    also: &[],
    view: stage::view(),
};

/// The reader's sounds, from the app.
fn setup(app: &World) -> Setup {
    let sound = |slot| {
        app.get_resource::<Yours>()
            .map_or_else(Handle::default, |yours| yours.sound(slot))
    };
    let sounds = [
        sound(Slot::Hit),
        sound(Slot::Bounce),
        sound(Slot::Pop),
        sound(Slot::Coin),
    ];
    Box::new(move |world| {
        let [hit, bounce, pop, coin] = sounds.clone();
        world.insert_resource(Sounds {
            hit,
            bounce,
            pop,
            coin,
        });
    })
}

#[derive(Resource)]
struct Sounds {
    hit: Handle<AudioSource>,
    bounce: Handle<AudioSource>,
    pop: Handle<AudioSource>,
    coin: Handle<AudioSource>,
}

/// A pad to click, and the sound it plays.
#[derive(Component)]
struct Pad(Handle<AudioSource>);

fn spawn_pads(mut commands: Commands, sounds: Res<Sounds>) {
    let pads = [
        (&sounds.hit, Color::srgb(0.95, 0.45, 0.4)),
        (&sounds.bounce, Color::srgb(0.35, 0.75, 0.9)),
        (&sounds.pop, Color::srgb(0.55, 0.85, 0.45)),
        (&sounds.coin, Color::srgb(0.98, 0.75, 0.3)),
    ];
    for (i, (sound, color)) in pads.into_iter().enumerate() {
        commands.spawn((
            Pad(sound.clone()),
            Sprite::from_color(color, Vec2::splat(56.0)),
            Transform::from_xyz(-105.0 + i as f32 * 70.0, 0.0, 1.0),
        ));
    }
}

fn play(click: On<PointerClick>, mut pads: Query<(&Pad, &mut Transform)>, mut commands: Commands) {
    let Ok((pad, mut transform)) = pads.get_mut(click.entity) else {
        return;
    };
    // A new player each time: the same sound can overlap itself.
    commands.spawn((AudioPlayer::new(pad.0.clone()), PlaybackSettings::DESPAWN));
    transform.scale = Vec3::splat(0.8);
}

/// Pressed pads spring back to their size.
fn spring_back(time: Res<Time>, mut pads: Query<&mut Transform, With<Pad>>) {
    let t = (time.delta_secs() * 12.0).min(1.0);
    for mut transform in &mut pads {
        transform.scale = transform.scale.lerp(Vec3::ONE, t);
    }
}

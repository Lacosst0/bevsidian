---
icon: volume-2
tags: [audio]
---
# Audio

Music and sound effects. In Bevy a sound is an entity: spawn it to play, despawn it to stop. #audio

## Play a sound

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Plays once, then the entity is despawned.
    commands.spawn(AudioPlayer::new(asset_server.load("sounds/jump.ogg")));
}
```

Put `.ogg` files in `assets/sounds/`. (Vorbis is on with the `audio` feature; other formats are separate features: `mp3`, `wav`, `flac`, see [[Project setup]].)

## Looping music

```rust
use bevy::{audio::Volume, prelude::*};

#[derive(Component)] struct Music;

fn music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Music,
        AudioPlayer::new(asset_server.load("sounds/theme.ogg")),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.5)),
    ));
}
```

`PlaybackSettings` has presets: `ONCE` (default), `LOOP`, `DESPAWN` (remove when done), `REMOVE`. Chain tweaks: `.with_speed(1.2)`, `.paused()`, `.with_spatial(true)`.

## Sound effects on events

Load the handle once ([[Assets]]), spawn a new player per play:

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct Sounds { hit: Handle<AudioSource> }

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Sounds { hit: asset_server.load("sounds/hit.ogg") });
}

#[derive(EntityEvent)]
struct Hit { entity: Entity }

fn play_hit(_: On<Hit>, sounds: Res<Sounds>, mut commands: Commands) {
    commands.spawn((
        AudioPlayer::new(sounds.hit.clone()),
        PlaybackSettings::DESPAWN,     // clean up the entity when finished
    ));
}
```

An [[Events and observers|observer]] on a game event keeps sound out of your gameplay code.

## Make your own sounds

Game sounds do not have to be recordings. The classic "blips" of old games are made from a few numbers: a **wave** (square, saw, sine or noise), a **pitch** that can **slide** up or down, a **length** and a **volume**. Press the **speaker** button above this preview to make your own, from ready-made ones (coin, jump, hit, laser…) or by surprise. They are saved, and the other examples play them: the hit in Dodge ([[Make a game]]), the bouncing balls in [[Physics]], the pop of a new square in [[Sprites]].

Click a pad to play its sound:

```rust example=sounds
use bevy::prelude::*;

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
```

Each click spawns a new player, so a sound can overlap itself; `PlaybackSettings::DESPAWN` cleans each one up when it ends.

## Control a playing sound

While it plays, the entity has an [`AudioSink`]:

```rust
use bevy::{audio::Volume, prelude::*};

#[derive(Component)] struct Music;

fn controls(keys: Res<ButtonInput<KeyCode>>, mut sink: Single<&mut AudioSink, With<Music>>) {
    if keys.just_pressed(KeyCode::Space) {
        sink.toggle_playback();                 // pause / resume
    }
    if keys.just_pressed(KeyCode::KeyM) {
        sink.toggle_mute();
    }
    if keys.just_pressed(KeyCode::Equal) {
        let louder = sink.volume().to_linear() + 0.1;
        sink.set_volume(Volume::Linear(louder));
    }
    sink.set_speed(1.0);                        // pitch and speed together
}
```

## Spatial audio

Sounds can come from a position: louder when close, panned left and right, using the distance and direction to a *listener* (usually on the camera):

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // The source.
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/engine.ogg")),
        PlaybackSettings::LOOP.with_spatial(true),
        Transform::from_xyz(5.0, 0.0, 0.0),
    ));
    // The ears: the gap is the distance between them.
    commands.spawn((Camera3d::default(), SpatialListener::new(0.2)));
}
```

Works in 2D too: place sources and the listener on the same plane (`spatial_audio_2d` example).

## Volume channels: music, effects, master

Players expect separate sliders. Mark each sound with its kind, keep the volumes in a resource, and write them to the sinks whenever the resource changes:

```rust
use bevy::{audio::Volume, prelude::*};

#[derive(Resource)]
struct Volumes { master: f32, music: f32, effects: f32 }

#[derive(Component)]
struct Music;

#[derive(Component)]
struct Effect;

fn apply_volumes(
    volumes: Res<Volumes>,
    mut music: Query<&mut AudioSink, (With<Music>, Without<Effect>)>,
    mut effects: Query<&mut AudioSink, (With<Effect>, Without<Music>)>,
) {
    if !volumes.is_changed() {
        return;
    }
    for mut sink in &mut music {
        sink.set_volume(Volume::Linear(volumes.master * volumes.music));
    }
    for mut sink in &mut effects {
        sink.set_volume(Volume::Linear(volumes.master * volumes.effects));
    }
}
```

Spawn effects with `Effect` and the starting volume from the same resource, so a new sound is not louder than the slider says. Volume sliders feel right on a curve: use the slider value squared, since loudness is not linear.

## Fade in and out

Abrupt starts and stops click. Fade the sink's volume over time. This one fades out and then despawns (an easing curve would be gentler, see [[Game feel]]):

```rust
use bevy::{audio::Volume, prelude::*};

#[derive(Component)]
struct FadeOut { seconds: f32, left: f32 }

fn fade_out(
    time: Res<Time>,
    mut commands: Commands,
    mut sounds: Query<(Entity, &mut FadeOut, &mut AudioSink)>,
) {
    for (entity, mut fade, mut sink) in &mut sounds {
        fade.left -= time.delta_secs();
        sink.set_volume(Volume::Linear((fade.left / fade.seconds).max(0.0)));
        if fade.left <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

To change the music, fade the old track out and spawn the new one at once: that is a crossfade.

## Variety and limits

The same footstep a hundred times is fatiguing. Two cheap fixes:

```rust
use bevy::{audio::Volume, prelude::*};

#[derive(Resource)]
struct Footsteps(Vec<Handle<AudioSource>>);

fn footstep(mut commands: Commands, steps: Res<Footsteps>, time: Res<Time>) {
    // Pick one of several recordings, and shift the pitch a little.
    let t = time.elapsed_secs();
    let which = (t * 7.0) as usize % steps.0.len();     // use real randomness in a real game
    let pitch = 0.95 + (t * 13.0).sin().abs() * 0.1;
    commands.spawn((
        AudioPlayer::new(steps.0[which].clone()),
        PlaybackSettings::DESPAWN.with_speed(pitch).with_volume(Volume::Linear(0.6)),
    ));
}
```

And **limit** how many play at once: a swarm of enemies dying on one frame can spawn hundreds of players. Count `AudioPlayer`s of one kind and skip new ones past a cap (8 is plenty).

## The browser needs a click first

Web browsers refuse to start audio until the player has interacted with the page. If your music is silent on the web ([[Web build]]), that is why: show a "Click to start" screen, and begin the music after it.

## Settings players expect

Store music volume, effects volume and mute in a [[Resources|resource]] and apply on change. Separate music from effects so players can turn down one. See [[Saving and loading]].

## Libraries

Comparisons of crates that do similar jobs: [[Audio crates]].

| Crate | For | Bevy |
|:--|:--|:--|
| [[bevy_kira_audio]] | channels, tweens, effects | #0.20 |
| [[bevy_seedling]] | audio graph, effects | #0.19 |
| [[bevy_fmod]] | FMOD middleware | #0.19 |

## Next

Bevy's audio is basic by design. For buses, effects (reverb, EQ), crossfades and dynamic music use a library: [[bevy_kira_audio]] (#0.20), [[bevy_seedling]] (Firewheel based), or [[bevy_fmod]] (professional FMOD middleware).

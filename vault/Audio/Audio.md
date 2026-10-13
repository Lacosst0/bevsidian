---
icon: volume-2
tags: [audio, beginner]
---
# Audio

Music and sound effects. In Bevy a sound is an entity: spawn it to play, despawn it to stop. #audio #beginner

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

## Settings players expect

Store music volume, effects volume and mute in a [[Resources|resource]] and apply on change. Separate music from effects so players can turn down one. See [[Saving and loading]].

## Next

Bevy's audio is basic by design. For buses, effects (reverb, EQ), crossfades and dynamic music use a library: [[bevy_kira_audio]] (works with Bevy 0.20 now), [[bevy_seedling]] (Firewheel based), or [[bevy_fmod]] (professional FMOD middleware).

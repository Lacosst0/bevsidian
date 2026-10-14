---
lib: true
icon: package
tags: [audio]
crates: bevy_kira_audio=0.27
---
# Audio crates

When Bevy's [[Audio]] is not enough: Kira for channels, tweens and effects, FMOD for professional sound design.

## bevy_kira_audio

Audio with **channels**, tweens and effects, powered by the Kira engine. Use it when Bevy's built-in audio ([[Audio]]) is not enough: separate music/effects buses, fades, loops with intros, reverb, spatial sound, and finer control of playing sounds. #audio

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_kira_audio 0.27` supports **Bevy 0.20**.

```toml
bevy_kira_audio = "0.27"
```

It replaces Bevy's own audio: add `AudioPlugin` from the crate **instead of** the built-in one (turn off `bevy_audio` in `default-features`).

### Play a sound

```rust
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_systems(Startup, music)
        .run();
}

fn music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("sounds/theme.ogg"))
        .looped()
        .fade_in(AudioTween::linear(std::time::Duration::from_secs(2)))
        .with_volume(0.6);
}
```

`Audio` is a global channel: call `.play(handle)` and chain settings. The returned handle controls that one sound (pause, stop, seek).

### Channels: buses for music and effects

A channel is a typed bus with its own volume, pause and effects:

```rust
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

#[derive(Resource)]
struct Music;

#[derive(Resource)]
struct Effects;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AudioPlugin))
        .add_audio_channel::<Music>()
        .add_audio_channel::<Effects>()
        .add_systems(Startup, start)
        .run();
}

fn start(asset_server: Res<AssetServer>, music: Res<AudioChannel<Music>>, effects: Res<AudioChannel<Effects>>) {
    music.set_volume(0.4);                                   // the settings screen controls these
    effects.set_volume(0.9);
    music.play(asset_server.load("sounds/theme.ogg")).looped();
    effects.play(asset_server.load("sounds/jump.ogg"));
}
```

Your options menu becomes two sliders that call `set_volume` on the channels.

### More

- **Tweens**: smooth changes to volume, pitch and panning.
- **Spatial audio** (`spatial` example): position emitters in the world.
- **Effects and reverb** channels (`reverb_channel`, `effects`).
- **Dynamic channels** (`dynamic_channels`): create channels by name at runtime.
- **Peak meter** for visualisers.

### Compared to alternatives

| | Bevy audio | bevy_kira_audio | [[bevy_seedling]] | [[Audio crates#bevy_fmod\|bevy_fmod]] |
|:--|:--|:--|:--|:--|
| Setup | built in | one plugin | one plugin | FMOD install + licence |
| Buses / mixing | no | channels | a full audio graph | professional mixer |
| Bevy 0.20 | yes | **yes** | release pending | release pending |

### Links

[crates.io](https://crates.io/crates/bevy_kira_audio) · [docs.rs](https://docs.rs/bevy_kira_audio) · [repo](https://github.com/NiklasEi/bevy_kira_audio)

## bevy_fmod

**FMOD** in Bevy: the middleware used by many shipped games. Sound designers build events, mixing and adaptive music in the FMOD Studio app; your game triggers events and sets parameters. #audio

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): the README states `0.11` works with **Bevy 0.19** and FMOD 2.02.22; a 0.20 release is pending.

### What you need

- A free FMOD account, the **FMOD Engine** download, and linking it by hand (licence terms prevent bundling).
- Attribution as FMOD's licence requires.
- Sound designers who use FMOD Studio.

### When it is worth it

Large audio-driven games: adaptive music, complex mixing, a dedicated audio person. Otherwise [[Audio crates#bevy_kira_audio|bevy_kira_audio]] (released for 0.20), [[bevy_seedling]] or Bevy's own audio ([[Audio]]) are far simpler.

### Links

[crates.io](https://crates.io/crates/bevy_fmod) · [repo](https://github.com/Salzian/bevy_fmod) · [FMOD](https://www.fmod.com)

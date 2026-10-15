---
parent: Audio
icon: sliders-horizontal
tags: [audio]
crates: bevy_kira_audio=0.27
---
# Audio crates

For when Bevy's built-in [[Audio]] isn't enough. From friendly to professional:

| | [[bevy_kira_audio]] | [[bevy_seedling]] | [[bevy_fmod]] |
|:--|:--|:--|:--|
| Bevy | #0.20 | #0.19 | #0.19 |
| Model | channels with tweens | an audio graph of nodes | FMOD Studio events |
| Good at | fades, loops with intros, buses | effects wired like a mixer | adaptive music, a sound designer's workflow |
| Cost | free | free | FMOD licence for commercial games |

> [!TIP]
> - Music plus effects with volume sliders: built-in audio is plenty.
> - Fades, channels and reverb: [[bevy_kira_audio]].
> - A node graph of effects: [[bevy_seedling]].
> - A sound designer owns the audio: [[bevy_fmod]].

## Looping music, both ways

```rust variant="Built in" fresh
use bevy::{audio::Volume, prelude::*};

fn music(mut commands: Commands, asset_server: Res<AssetServer>) {
    // A sound is an entity: despawn it to stop.
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/theme.ogg")),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.6)),
    ));
}
```

```rust variant=bevy_kira_audio fresh
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

fn music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // A sound is a call on a channel, with a fade built in.
    audio
        .play(asset_server.load("sounds/theme.ogg"))
        .looped()
        .fade_in(AudioTween::linear(std::time::Duration::from_secs(2)))
        .with_volume(0.6);
}
```

## Next

[[Audio]].

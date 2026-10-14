---
parent: Audio
icon: sliders-horizontal
tags: [audio]
---
# Audio crates

When Bevy's built-in [[Audio]] is not enough. Three crates, from friendly to professional.

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy_kira_audio]] | channels, tweens and effects on the Kira engine | released `0.27` | you want buses, fades, loops with intros |
| [[bevy_seedling]] | an audio graph with effects | pending `0.8` | you want to wire effects like a mixer |
| [[bevy_fmod]] | FMOD: events and adaptive music designed in FMOD Studio | pending `0.11` (Bevy 0.19) | a sound designer owns the audio |

## Which one?

- Music and effect volumes only: the built-in audio is enough.
- Fades, channels, reverb on Bevy 0.20 now: [[bevy_kira_audio]].
- A node graph of effects: [[bevy_seedling]].
- A professional pipeline with a sound designer: [[bevy_fmod]] (a commercial licence may apply).

## Next

[[Audio]].

---
parent: Audio
icon: sliders-horizontal
tags: [audio]
---
# Audio crates

When Bevy's built-in [[Audio]] is not enough. Three crates, from friendly to professional.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_kira_audio]] | channels, tweens and effects on the Kira engine | #0.20 | you want buses, fades, loops with intros |
| [[bevy_seedling]] | an audio graph with effects | #0.19 | you want to wire effects like a mixer |
| [[bevy_fmod]] | FMOD: events and adaptive music designed in FMOD Studio | #0.19 | a sound designer owns the audio |

## Which one?

- Music and effect volumes only: the built-in audio is enough.
- Fades, channels, reverb on #0.20: [[bevy_kira_audio]].
- A node graph of effects: [[bevy_seedling]].
- A professional pipeline with a sound designer: [[bevy_fmod]] (a commercial licence may apply).

## Next

[[Audio]].

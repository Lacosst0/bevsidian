---
icon: package
tags: [audio]
check: skip
---
# bevy_fmod

**FMOD** in Bevy: the middleware used by many shipped games. Sound designers build events, mixing and adaptive music in the FMOD Studio app; your game triggers events and sets parameters. #audio

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): the README states `0.11` works with **Bevy 0.19** and FMOD 2.02.22; a 0.20 release is pending.

## What you need

- A free FMOD account, the **FMOD Engine** download, and linking it by hand (licence terms prevent bundling).
- Attribution as FMOD's licence requires.
- Sound designers who use FMOD Studio.

## When it is worth it

Large audio-driven games: adaptive music, complex mixing, a dedicated audio person. Otherwise [[bevy_kira_audio]] (released for 0.20), [[bevy_seedling]] or Bevy's own audio ([[Audio]]) are far simpler.

## Links

[crates.io](https://crates.io/crates/bevy_fmod) · [repo](https://github.com/Salzian/bevy_fmod) · [FMOD](https://www.fmod.com)

## Next

[[Audio]].

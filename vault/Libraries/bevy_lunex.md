---
icon: package
tags: [lib, ui]
check: skip
---
# bevy_lunex

A **retained layout engine** for UI built from regular entities: any aspect ratio without distortion, a flexbox-like `flow` layout, **world-space UI** in 2D and 3D, interactivity through normal systems and observers. #lib #ui

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): the `main` branch targets **Bevy 0.20**; the last crates.io release `0.7` targets 0.19. Depend on the git repository for 0.20.

```toml
bevy_lunex = { git = "https://github.com/bytestring-net/bevy_lunex" }
```

## When to use it

- Menus that must look right on any window shape (ultrawide, phone, 4:3).
- UI placed **in the 3D world** (a screen on a wall, a health bar over an enemy).
- You prefer a layout engine to CSS-like flex.

Bevy's own UI ([[UI layout]]) already does flex and grid and is the default; Lunex is the choice when its specific strengths matter. The README's `Bevypunk` demo shows a full game menu.

## Links

[crates.io](https://crates.io/crates/bevy_lunex) · [docs.rs](https://docs.rs/bevy_lunex) · [repo](https://github.com/bytestring-net/bevy_lunex)

## Next

[[UI]], [[bevy_hui]], [[Libraries]].

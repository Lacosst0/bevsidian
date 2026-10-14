---
icon: wrench
tags: [tooling, beginner]
---
# Tooling

Everything around the game: seeing what it does, finding bugs, shipping it. #tooling #beginner

## While you build

| Note | Use it to |
|:--|:--|
| [[Gizmos]] | draw lines, circles, grids on top of the game, for debugging |
| [[Diagnostics and logging]] | read FPS, entity counts, write log messages |
| [[Dev tools]] | FPS overlay, state logging, remote inspection |
| [[Debugging]] | inspectors, the schedule graph, common bugs |
| [[Hot reloading]] | change assets and code without restarting |

## When you ship

| Note | Use it to |
|:--|:--|
| [[Release builds]] | make the game small and fast |
| [[Profiling]] | find out what is slow |
| [[Web build]] | run in a browser, on a phone |
| [[Mobile builds]] | Android and iOS apps |

The path [[Ship it]] puts these in order.

## Libraries

Status for **Bevy 0.20**, checked 2026-10-13: **released** (a crates.io release uses 0.20) · **git** (the `main` branch is on 0.20; use a git dependency) · **pending** (the latest release targets an older Bevy) · **built in** (Bevy has it now). Check crates.io before you start.

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy-inspector-egui]] | live world inspector | pending `0.37` |
| [[bevy_mod_debugdump]] | draw the schedule graph | pending `0.16` |
| `bevy_mod_scripting` | Lua, Rhai and other scripting | released `0.22` |

## Choosing libraries

Bevy is small on purpose; the ecosystem fills the gaps. Every area of this vault ends with a **Libraries** table: what each crate is for and whether it works with Bevy 0.20 today.

1. **Check the compatibility table** in the crate's README before adding it.
2. **Prefer built-in** when it is enough ([[Picking]], camera controllers, text input and atmosphere all moved into Bevy).
3. **Prefer a released crate** to a git branch; use git only for a crate you need now.
4. If two crates do the same job, read both examples and choose the one whose *style* you like: that is what you will live with.
5. Do not add libraries speculatively. Add one when you feel the pain it removes.

A good Bevy library is **one plugin**, configured through resources, not a framework. The Bevy Book's chapter "Creating libraries for Bevy" has the checklist for writing your own.

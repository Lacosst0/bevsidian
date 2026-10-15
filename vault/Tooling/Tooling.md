---
icon: wrench
tags: [tooling]
---
# Tooling

Everything around the game: seeing what it does, finding bugs, shipping it. #tooling

## While you build

| Note | Use it to |
|:--|:--|
| [[Gizmos]] | draw lines, circles, grids on top of the game, for debugging |
| [[Diagnostics and logging]] | read FPS, entity counts, write log messages |
| [[Diagnostics and logging#Dev tools\|Dev tools]] | FPS overlay, state logging, remote inspection |
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

Comparisons of crates that do similar jobs: [[Debug crates]].

| Crate | For | Bevy |
|:--|:--|:--|
| [[bevy-inspector-egui]] | live world inspector | #0.19 |
| [[bevy_mod_debugdump]] | draw the schedule graph | #0.19 |
| `bevy_mod_scripting` | Lua, Rhai and other scripting | #0.20 |

## Choosing libraries

Bevy is small on purpose; the ecosystem fills the gaps. Every area of this vault ends with a **Libraries** table: what each crate is for, tagged #0.20 when it works with Bevy 0.20 and #0.19 while it is still on an older Bevy.

1. **Check the compatibility table** in the crate's README before adding it.
2. **Prefer built-in** when it is enough ([[Picking]], camera controllers, text input and atmosphere all moved into Bevy).
3. **Prefer a released crate** to a git branch; use git only for a crate you need now.
4. If two crates do the same job, read both examples and choose the one whose *style* you like: that is what you will live with.
5. Do not add libraries speculatively. Add one when you feel the pain it removes.

A good Bevy library is **one plugin**, configured through resources, not a framework. The Bevy Book's chapter "Creating libraries for Bevy" has the checklist for writing your own.

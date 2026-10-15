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

Crates compared side by side: [[Debug crates]].

| Crate | For | Bevy |
|:--|:--|:--|
| [[bevy-inspector-egui]] | live world inspector | #0.19 |
| [[bevy_mod_debugdump]] | draw the schedule graph | #0.19 |
| `bevy_mod_scripting` | Lua, Rhai and other scripting | #0.20 |

## Choosing libraries

Bevy is small on purpose; crates fill the gaps. Every area here ends with a **Libraries** table, each crate tagged with the Bevy its latest release supports (#0.20 or #0.19).

- **Check the crate's compatibility table** before adding it.
- **Prefer what's built in** when it's enough: picking, camera controllers, text input and atmosphere all moved into Bevy.
- **Prefer a release** to a git branch.
- **Two crates, one job?** Read both examples and pick the style you'd rather live with.
- **Add a crate when you feel the pain it removes**, not before.

A good Bevy library is **one plugin**, configured through resources. The Bevy Book's "Creating libraries for Bevy" has the checklist.

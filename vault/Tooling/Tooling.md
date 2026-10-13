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

[[bevy-inspector-egui]] (see and edit the world), [[bevy_egui]] (debug windows), [[bevy_mod_debugdump]] (draw your schedule), [[bevy_pkv]] (save settings).

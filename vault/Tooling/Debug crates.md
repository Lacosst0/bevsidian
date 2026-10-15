---
parent: Tooling
icon: sliders-horizontal
tags: [tooling]
---
# Debug crates

Two crates that show what your app is doing. They answer different questions, so most projects use both.

| | [[bevy-inspector-egui]] | [[bevy_mod_debugdump]] |
|:--|:--|:--|
| Bevy | #0.19 | #0.19 |
| Answers | "what's in my world right now?" | "in what order do my systems run?" |
| Shows | every entity, component and resource, editable live | the schedule as a graph |
| Output | a window inside the game | a `.dot` file to render |

> [!TIP]
> - Poke at values while the game runs: [[bevy-inspector-egui]], or Bevy's own remote tools ([[Diagnostics and logging#Dev tools|Dev tools]]).
> - A system ran too early: [[bevy_mod_debugdump]].

## Next

[[Debugging]], [[Tooling]].

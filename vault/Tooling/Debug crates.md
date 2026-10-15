---
parent: Tooling
icon: sliders-horizontal
tags: [tooling]
---
# Debug crates

Two crates that show you what the app is doing. They answer different questions and are used together.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy-inspector-egui]] | a live inspector: every entity and resource, editable | #0.19 | you want to see and change the world while it runs |
| [[bevy_mod_debugdump]] | draws the schedule as a graph | #0.19 | you wonder why one system ran before another |

## Which one?

- "What is in my world right now?" [[bevy-inspector-egui]]. Bevy also has its own remote inspection ([[Diagnostics and logging#Dev tools|Dev tools]]).
- "In what order do my systems run?" [[bevy_mod_debugdump]].

## Next

[[Debugging]], [[Tooling]].

---
parent: Tooling
icon: sliders-horizontal
tags: [tooling]
---
# Debug crates

Two crates that show you what the app is doing. They answer different questions and are used together.

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy-inspector-egui]] | a live inspector: every entity and resource, editable | pending `0.37` | you want to see and change the world while it runs |
| [[bevy_mod_debugdump]] | draws the schedule as a graph | pending `0.16` | you wonder why one system ran before another |

## Which one?

- "What is in my world right now?" [[bevy-inspector-egui]]. Bevy 0.20 also has its own remote inspection ([[Diagnostics and logging#Dev tools|Dev tools]]).
- "In what order do my systems run?" [[bevy_mod_debugdump]].

## Next

[[Debugging]], [[Tooling]].

---
icon: component
tags: [bevsidian]
---
# Bevsidian

The app you are reading this in. It is **not** part of Bevy: it is a program written *with* Bevy 0.20, so its own source is a long worked example. This section is for readers who want to see how it is built. #bevsidian

| Note | What it covers |
|:--|:--|
| [[UI kit]] | the app's own component library (`bevsidian_ui`), modelled on shadcn/ui |

## How it is put together

- **Bevy UI and BSN** draw everything: the sidebar, the graph, the dialogs, and the notes themselves ([[Scenes and BSN]]).
- **Reactivity** comes from [[bevy_radioactive]], the `view!` macro.
- **Live code previews** run small Bevy worlds inside a note: the `example=NAME` blocks you see throughout the course.
- **The vault** is plain Markdown in the repository's `vault/` folder, embedded at build time.

The README in the repository has the details; this section only exists so the course notes stay about Bevy itself.

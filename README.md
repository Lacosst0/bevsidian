# Bevsidian

Bevy's documentation lives in many places: the book, examples, API docs, release notes and community guides. **Bevsidian** brings them together as one Obsidian-style vault: markdown notes connected by `[[wikilinks]]` and `#tags`, with backlinks, an outline and a graph view. It is itself a Bevy 0.20 app, built with BSN scenes and [`bevy_radioactive`](https://github.com/Lacosst0/bevy_radioactive)'s `view!` templates.

## Features

- **Markdown reading view** rendered natively with Bevy UI: headings, emphasis, inline code, syntax-highlighted code blocks with a copy button, tables, task lists, nested lists, callouts (`> [!TIP]`), footnotes and horizontal rules.
- **Selection like a web page**: drag across paragraphs, double-click a word, triple-click a block, Shift+click to extend, Shift+arrows, `Ctrl/Cmd+A`, `Ctrl/Cmd+C`.
- **Obsidian linking**: `[[Note]]`, `[[Note#Heading]]`, `[[Note|alias]]`, `#tags` and front-matter `tags:`.
- **Navigation**: one sidebar with a note tree (folders, notes, and the open note's headings), full-text search with tag suggestions, and a links popover by the title (backlinks and outgoing links); back/forward history (`Alt+←/→`, mouse buttons) and a quick switcher (`Ctrl+O`).
- **Graph view** (`Ctrl+G`): a force-directed graph of notes, drawn as their icons, rendered by a 2D camera into a texture shown in the UI. Hover highlights neighbours, click opens a note, drag moves nodes or pans, and the wheel zooms.
- **Single-file web build**: one `index.html` with CSS, JS and the gzip-compressed WASM inlined. Host it on any static server or open it from disk.

## Workspace

| Crate | Purpose |
| --- | --- |
| [`bevy_ui_textselect`](crates/bevy_ui_textselect) | Mouse and keyboard selection, clipboard copy, link-click events and accessibility labels for rich `Text`/`TextSpan` UI text. Hit-testing and highlight geometry come straight from the parley layout Bevy already computes. |
| [`bevy_ui_commonmark`](crates/bevy_ui_commonmark) | CommonMark + GFM + Obsidian wikilinks/tags parser, and a builder that turns documents into BSN scenes of selectable text. |
| [`bevy_ui_touchscroll`](crates/bevy_ui_touchscroll) | Native-feeling touch scrolling for `ScrollArea`s: 1:1 drags, Android/Flutter fling physics, tap-to-stop, and eased wheel scrolling. |
| [`bevsidian_core`](crates/bevsidian_core) | The note vault (links, backlinks, tags, search), navigation state (`Nav` signals, shortcuts, links) and the note icons. |
| [`bevsidian_ui`](crates/bevsidian_ui) | The UI kit, modelled on [shadcn/ui](https://ui.shadcn.com): design tokens (colours, radii, spacing, shadows, layers, Geist type) and one file per component (buttons, tabs, segmented controls, badges, keys, rows and trees, inputs, switches, steppers, dialogs, menus, toasts, alerts, entity cards) as `bsn!`/`view!` scenes. Its [README](crates/bevsidian_ui/README.md) explains the conventions; the [`UI kit`](vault/UI%20kit.md) note previews the components live, and [`design/`](design) holds the mockups. |
| [`bevsidian_graph`](crates/bevsidian_graph) | The graph view: force-directed notes with pan, pinch/wheel zoom and click-to-open. |
| [`bevsidian_preview`](crates/bevsidian_preview) | Bevy-in-Bevy: live previews above code blocks. A preview runs an example program with the block's systems in a world of its own: as entity cards, one system at a time with Bevy's stepping (back, play/pause, forward, highlighting the lines that just ran), or as a playable 2D game. |
| `bevsidian` (root) | The app: the shell (`src/shell`: sidebar, top bar), pages (`src/pages`: note, graph), overlays (`src/overlays`: settings, quick switcher, context menu, toast, selection toolbar, on-screen keyboard), responsive layout, the note editor, the FPS readout and the embedded vault. |

## Adding notes

Put markdown files in [`vault/`](vault) (sub-folders are fine). They are embedded at compile time by `build.rs`. A note's name is its file name, so `vault/guides/Assets.md` is linked as `[[Assets]]`.

Give a note an icon with an `icon:` property naming a [Lucide](https://lucide.dev/icons) icon:

```markdown
---
icon: rocket
---
```

The icon shows next to the title, in the sidebar and in the graph. Only the icons listed in [`tools/icons.py`](tools/icons.py) are bundled (to keep the download small); add a name there and run `python3 tools/icons.py` to bundle another.

## Live code previews

Code blocks can run **examples**: small Bevy programs in [`crates/bevsidian_preview/src/examples/`](crates/bevsidian_preview/src/examples), each set up once and shared by any number of blocks. Give a block `example=NAME` in its info string (` ```rust example=mario `): the preview above it runs the example in a Bevy world of its own, with the example's base systems plus the functions the block's code mentions. A note can show a game piece by piece, each block running the part it shows. `fps=N` sets how fast a stepped preview plays.

An example shows its world either as **cards** of its entities and component values (stepped one system at a time with Bevy's stepping, highlighting the lines that ran), or as a **game**: its sprites, in real time, with pointer input turned into Bevy picking events (`PointerClick`, `PointerDrag`, …) in its world. See [`ECS`](vault/ECS.md) and [`Mario`](vault/Mario.md).

To add one, write the program in a file there, list its functions with `startup!`, `update!` and `observer!` in an `Example`, and name it in `examples/mod.rs`. Keep the notes' code in sync with it.

`preview=NAME` shows a UI scene instead (see [`UI kit`](vault/UI%20kit.md)); those live in `examples/scenes.rs`.

## Running

```sh
# native
cargo run

# web (single file)
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <version in Cargo.lock>
tools/build-web.sh          # writes dist/index.html and dist/bevsidian-web.zip
```

For quick test builds of the web version (seconds after the first build; no LTO or wasm-opt,
so it is bigger and a little slower):

```sh
PROFILE=wasm-dev tools/build-web.sh
```

On Linux, native builds need `libwayland-dev libxkbcommon-dev`.

## Releases

[`.github/workflows/release.yml`](.github/workflows/release.yml) builds the single-file web bundle and attaches `bevsidian-web.zip` to a GitHub release whenever a `v*` tag is pushed. It can also be run manually, in which case the zip is uploaded as a workflow artifact.

## Hosting on GitHub Pages

[`.github/workflows/pages.yml`](.github/workflows/pages.yml) publishes one page per branch to the `gh-pages` branch on every push: `main` at the site root, every other branch at `/<branch>/` (`/` in the name becomes `-`). [`pages-cleanup.yml`](.github/workflows/pages-cleanup.yml) removes a branch's page when the branch is deleted. To enable it, open **Settings → Pages** and set **Source** to **Deploy from a branch**, Branch `gh-pages` / `(root)` (the first workflow run creates the branch). For access-controlled ("private") Pages, the repository must belong to an organization on GitHub Enterprise Cloud; then set **Visibility** to **Private** on the same page.

## Fonts

Geist and Geist Mono are bundled under the SIL Open Font License (see `assets/fonts/`).

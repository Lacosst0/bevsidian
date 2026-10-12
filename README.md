# Bevsidian

Bevy's documentation lives in many places: the book, examples, API docs, release notes and community guides. **Bevsidian** brings them together as one Obsidian-style vault: markdown notes connected by `[[wikilinks]]` and `#tags`, with backlinks, an outline and a graph view. It is itself a Bevy 0.20 app, built with BSN scenes and [`bevy_radioactive`](https://github.com/Lacosst0/bevy_radioactive)'s `coat!` templates.

## Features

- **Markdown reading view** rendered natively with Bevy UI: headings, emphasis, inline code, syntax-highlighted code blocks with a copy button, tables, task lists, nested lists, callouts (`> [!TIP]`), footnotes and horizontal rules.
- **Selection like a web page**: drag across paragraphs, double-click a word, triple-click a block, Shift+click to extend, Shift+arrows, `Ctrl/Cmd+A`, `Ctrl/Cmd+C`.
- **Obsidian linking**: `[[Note]]`, `[[Note#Heading]]`, `[[Note|alias]]`, `#tags` and front-matter `tags:`.
- **Navigation**: file list with full-text search, tag filters, back/forward history (`Alt+←/→`, mouse buttons), quick switcher (`Ctrl+O`), outline, backlinks and outgoing links.
- **Graph view** (`Ctrl+G`): a force-directed graph of notes and tags, rendered by a 2D camera into a texture shown in the UI. Hover highlights neighbours, click opens a note, drag moves nodes or pans, and the wheel zooms.
- **Single-file web build**: one `index.html` with CSS, JS and the gzip-compressed WASM inlined. Host it on any static server or open it from disk.

## Workspace

| Crate | Purpose |
| --- | --- |
| [`bevy_ui_textselect`](crates/bevy_ui_textselect) | Mouse and keyboard selection, clipboard copy, link-click events and accessibility labels for rich `Text`/`TextSpan` UI text. Hit-testing and highlight geometry come straight from the parley layout Bevy already computes. |
| [`bevy_ui_commonmark`](crates/bevy_ui_commonmark) | CommonMark + GFM + Obsidian wikilinks/tags parser, and a builder that turns documents into BSN scenes of selectable text. |
| [`bevy_ui_touchscroll`](crates/bevy_ui_touchscroll) | Native-feeling touch scrolling for `ScrollArea`s: 1:1 drags, Android/Flutter fling physics, tap-to-stop, and eased wheel scrolling. |
| [`bevsidian_core`](crates/bevsidian_core) | The note vault (links, backlinks, tags, search), navigation state (`Nav` signals, shortcuts, links) and the theme. |
| [`bevsidian_graph`](crates/bevsidian_graph) | The graph view: force-directed notes and tags with pan, pinch/wheel zoom and click-to-open. |
| `bevsidian` (root) | The app: UI shell, responsive layout, context menu, settings, FPS readout, and the embedded vault. |

## Adding notes

Put markdown files in [`vault/`](vault) (sub-folders are fine). They are embedded at compile time by `build.rs`. A note's name is its file name, so `vault/guides/Assets.md` is linked as `[[Assets]]`.

Give a note an icon with an `icon:` property naming a [Lucide](https://lucide.dev/icons) icon:

```markdown
---
icon: rocket
---
```

The icon shows next to the title, in the file list and in the graph. Only the icons listed in [`tools/icons.py`](tools/icons.py) are bundled (to keep the download small); add a name there and run `python3 tools/icons.py` to bundle another.

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

[`.github/workflows/pages.yml`](.github/workflows/pages.yml) deploys `dist/index.html` to GitHub Pages on every push to `main`. To enable it, open **Settings → Pages** and set **Source** to **GitHub Actions**. For access-controlled ("private") Pages, the repository must belong to an organization on GitHub Enterprise Cloud; then set **Visibility** to **Private** on the same page.

## Fonts

Inter and JetBrains Mono are bundled under the SIL Open Font License (see `assets/fonts/`).

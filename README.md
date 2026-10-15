# Bevsidian

Bevy's documentation lives in many places: the book, examples, API docs, release notes and community guides. **Bevsidian** brings them together as one Obsidian-style vault: markdown notes connected by `[[wikilinks]]` and `#tags`, with backlinks, an outline and a graph view. It is itself a Bevy 0.20 app, built with BSN scenes and [`bevy_radioactive`](https://github.com/Lacosst0/bevy_radioactive)'s `view!` templates.

## Features

- **Markdown reading view** rendered natively with Bevy UI: headings, emphasis, inline code, syntax-highlighted code blocks with a copy button, code variants (consecutive blocks marked `variant=NAME`, or `variant="Two words"`, become one block with tabs; each variant shows the lines it adds, removes and changes against the first, and can be written as full code or as a `diff` block of the first with ` `/`-`/`+` line markers), tables (row hover; wide ones scroll sideways), task lists, nested lists, callouts (`> [!TIP]`), footnotes and horizontal rules.
- **Find in the note** (`Ctrl+F`): every match is marked and the current one stands out; Enter or F3 steps through them (with Shift, backwards) and scrolls each into view; Escape closes.
- **Autoscroll**: press the mouse wheel and move away from that point to scroll, faster the further you go; hold it and let go to stop, or click it and click again (or press `Escape`) to stop.
- **Selection like a web page**: drag across paragraphs (starting on text or beside it, e.g. in the margin), double-click a word, triple-click a block, Shift+click to extend, Shift+arrows, `Ctrl/Cmd+A`, `Ctrl/Cmd+C`.
- **Obsidian linking**: `[[Note]]`, `[[Note#Heading]]`, `[[Note|alias]]`, `#tags` and front-matter `tags:`.
- **Docs references**, like rustdoc's intra-doc links: `` [`Commands`] ``, `` [`Commands::spawn`] ``, `` [text][`bsn!`] ``, `[text](ecs::system::Query)` or `[text](docs:Time)` open Bevy's API docs on docs.rs. Names resolve through an index of Bevy's items ([`tools/docs_index.py`](tools/docs_index.py)); unknown ones open a docs.rs search.
- **Navigation**: one sidebar with a note tree (folders, notes, and the open note's headings), full-text search with tag suggestions, and a links popover by the title (backlinks and outgoing links); back/forward history (`Alt+←/→`, mouse buttons) and a quick switcher (`Ctrl+O`).
- **Graph view as a star map** (`Ctrl+G`): the vault drawn as a hand-made map of constellations. Every note sits where [`Map`](vault/Map.md) puts it: Bevy, the largest, in the middle; the main quest as a spiral winding out of it; each area's notes together, over a parallax field of background stars and a faint celestial grid (labels and nebulae can be added in `Map`). A note is a filled shape with its icon inside and its name above, large enough to read; the shape says what it is (an octagon for Bevy, a hexagon for an area's index, a diamond for a milestone, a star for a route's goal, a pentagon for a comparison of crates, a square for a crate, a circle for a lesson), and it is dark inside until you have read the note, filled grey after (orange while you read it); its icon on the note's page wears its route's colour. Routes are lines like a metro map's: **each route has its colour**, on its line, around its notes and filling its first page, while the main quest is one solid white line whose notes take no other colour. The colours go round the colour wheel in the order the routes sit round the map, and related areas sit side by side (UI beside BSN; 2D, 3D and rendering together; gameplay beside ECS, input and physics), so neighbours have neighbouring colours that still tell apart. Solid for the route itself, **dashed where you choose your path** (the line a route leaves the main quest by), **dotted for optional steps**; every route branches off the spiral at the note nearest its first step, and each side step hangs from the nearest core step of its route. A note no route takes (mostly crates) is tied by a thin line to a note near it that it links with, such as the comparison of crates it is in, and joins that note's route: it wears its colour and shows with it. The first time the map opens, a checklist asks **what's in your game** (2D, 3D, menus, controls, physics, sound, enemies, procedural levels, items and saves, multiplayer, polish; and ECS, BSN or shipping to get better at): the map then shows the main quest and those routes (the checklist's icons are in the routes' colours), and **Whole map** shows every note, with every route in its colour. The quest log in the corner lists the routes shown, in their colours, with how many of their steps you have read; choosing one follows it: its line brightens, its notes are numbered, and each of its notes ends with links to the previous and next step. Zoomed out, notes keep a size on screen, names are drawn larger to stay readable and areas keep theirs furthest out; names sit on the sky so lines pass under them, and a name that would run into another, or over a note, waits until you zoom in (the route you follow keeps all its names). The pencil under the zoom buttons **edits the map**: drag notes, landmarks, labels and nebulae; give a note another shape or size; add sparkles, moons, suns, orbits, telescopes, satellites, labels and nebulae. Edits are written into `Map`'s block line by line and saved like any note edit. The map is rendered by a 2D camera into a texture shown in the UI; the wheel or a pinch zooms (in to twice the size, out to a little past the notes shown), a drag pans.
- **Single-file web build**: one `index.html` with CSS, JS and the gzip-compressed WASM inlined. Host it on any static server or open it from disk.

## Workspace

| Crate | Purpose |
| --- | --- |
| [`bevy_ui_textselect`](crates/bevy_ui_textselect) | Mouse and keyboard selection, clipboard copy, link-click events and accessibility labels for rich `Text`/`TextSpan` UI text. Hit-testing and highlight geometry come straight from the parley layout Bevy already computes. |
| [`bevy_ui_commonmark`](crates/bevy_ui_commonmark) | CommonMark + GFM + Obsidian wikilinks/tags parser, and a builder that turns documents into BSN scenes of selectable text. |
| [`bevy_ui_touchscroll`](crates/bevy_ui_touchscroll) | Native-feeling touch scrolling for `ScrollArea`s: 1:1 drags, Android/Flutter fling physics, tap-to-stop, eased wheel scrolling, and browser-style autoscroll on a mouse-wheel press (hold and pull, or click to keep scrolling). |
| [`bevsidian_core`](crates/bevsidian_core) | The note vault (links, backlinks, tags, search, reading paths), the map note's format (`map.rs`), navigation state (`Nav` signals, shortcuts, links) and the note icons. |
| [`bevsidian_ui`](crates/bevsidian_ui) | The UI kit, modelled on [shadcn/ui](https://ui.shadcn.com): design tokens (colours, radii, spacing, shadows, layers, Geist type) and one file per component (buttons, tabs, segmented controls, badges, keys, rows and trees, inputs, switches, steppers, dialogs, menus, toasts, alerts, entity cards) as `bsn!`/`view!` scenes. Its [README](crates/bevsidian_ui/README.md) explains the conventions; the [`UI kit`](vault/Bevsidian/UI%20kit.md) note previews the components live, and [`design/`](design) holds the mockups. |
| [`bevsidian_graph`](crates/bevsidian_graph) | The graph view as a star map: notes as stars placed by the vault's map note, routes as coloured lines, the sky (background stars, nebulae, celestial grid), labels and landmarks, the map editor, pan and pinch/wheel zoom. |
| [`bevsidian_preview`](crates/bevsidian_preview) | Bevy-in-Bevy: live previews above code blocks. A preview runs an example program with the block's systems in a world of its own: as entity cards, one system at a time with Bevy's stepping (back, play/pause, forward, highlighting the lines that just ran), or as a playable 2D game. |
| `bevsidian` (root) | The app: the shell (`src/shell`: sidebar, top bar), pages (`src/pages`: note, graph), overlays (`src/overlays`: settings, quick switcher, context menu, toast, selection toolbar, on-screen keyboard), responsive layout, the note editor, the FPS readout and the embedded vault. |

## The Bevy 0.20 course

The vault is a hands-on course. [`Bevy`](vault/Bevy.md) is the hub (the capital in the middle of the map); each area has an index note and short notes with code you can run on the page: [`ECS`](vault/ECS), [`Assets`](vault/Assets), [`Rendering`](vault/Rendering), [`2D`](vault/2D), [`3D`](vault/3D), [`UI`](vault/UI), [`Input`](vault/Input), [`Audio`](vault/Audio), [`Physics`](vault/Physics), [`Networking`](vault/Networking), [`Tooling`](vault/Tooling) and a note per ecosystem crate inside the area it belongs to (each area index ends with a Libraries table).

**Routes** (`vault/Paths/`) are notes with `path: true` whose links are the steps (put no other links in them). A step line ending in `(optional)` or `(extra)` hangs off the nearest core step as an optional step (dotted line); plain steps form the solid line (on the main quest, every step is on its solid line). `quest: main` marks the main quest (always on the map), `quest: feature` a feature a game may have and `quest: mastery` something to get better at; `feature:` is its line in the map's checklist.

**The map** ([`vault/Map.md`](vault/Map.md), `map: true`) holds where everything sits, in a ```` ```map ```` block: one line per note (`Name @ x, y`, optionally a shape and `small`/`large`) and per object (`label:`, `landmark:`, `nebula:`); the note explains the format. Every note of the course must be on it (the workspace test says which is missing); the welcome note and the `Bevsidian` folder stay off it.

**Every Rust block is compiled** against the Bevy version it is written for:

```sh
python3 tools/check_snippets.py            # all notes
python3 tools/check_snippets.py Queries    # one note
```

It writes a scratch crate in `target/snippets` (one module per block; see the script's header for the `cont`, `ignore` and `fresh` flags and the `crates:`, `bevy:` and `check: skip` front-matter properties). Notes about crates that have no release for the current Bevy yet say `bevy: 0.19` and are checked against that version.

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

An example shows its world either as **cards** of its entities and component values (stepped one system at a time with Bevy's stepping, highlighting the lines that ran), or as a **game**: its sprites, in real time, with pointer input turned into Bevy picking events (`PointerClick`, `PointerDrag`, …) in its world. See the [`2D`](vault/2D) and [`ECS`](vault/ECS) notes.

To add one, write the program in a file there, list its functions with `startup!`, `update!` and `observer!` in an `Example`, and name it in `examples/mod.rs`. Keep the notes' code in sync with it. An `Example` can also have **actions** (buttons above the preview that do something to its world, shown when the block mentions the action's `name`), a **`Log`** resource (lines the systems "say", shown under the cards) and a **`tick`** that runs at the start of every frame (e.g. to swap message buffers). 2D games draw sprites (including texture atlases and rotation); `examples/stage.rs` has the shared stage.

Examples list their functions with `startup!`, `update!`, `update_if!(f, condition)` (a system with a run condition), `observer!` and `scheduled!(f, OnEnter(State))` (a system in another schedule). `examples/states.rs` shows how to run Bevy's real state machinery in a bare world.

`preview=NAME` shows a UI scene instead (see [`UI kit`](vault/Bevsidian/UI%20kit.md)); those live in `examples/scenes.rs`.

### Pick blocks

A ```` ```rust pick ```` block is a program whose parts the reader switches on and off with chips (see [`bevy_radioactive`](vault/UI/bevy_radioactive.md)). Everything before the first `//@ option` is the base. `//@ anchor NAME` marks a line where options insert code (the marker is not shown); `//@ option ID Label` starts a chip, and each `//@ at NAME` below it starts the lines inserted at that anchor, indented like the anchor. Added lines are highlighted. `tools/check_snippets.py` compiles the base, every option alone and all of them together. The code is in `crates/bevy_ui_commonmark/src/pick.rs`.

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

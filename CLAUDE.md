# Bevsidian

An Obsidian-like documentation vault for Bevy, written in Bevy 0.20 (BSN scenes,
`bevy_radioactive` `view!` for reactivity). Native and web (`tools/build-web.sh`).

## UI

- All interface pieces come from the kit in `crates/bevsidian_ui` (tokens + components).
  **Before designing a new component, look at the matching shadcn/ui component**
  (<https://ui.shadcn.com/docs/components>, sources in `apps/v4/registry/new-york-v4/ui/` of
  `shadcn-ui/ui`) and follow its anatomy, sizes and states. The kit's README maps Tailwind
  classes to tokens.
- One component per file in `crates/bevsidian_ui/src/components/`, one function returning
  `bsn!` (or `view!` when it reads signals). Callers compose with `@{...}` and patch fields
  rather than adding options. Use a `SceneComponent` with props when a marker is queried.
- App screens: `src/shell` (sidebar, top bar), `src/pages` (note, graph), `src/overlays`
  (dialogs, menus, toasts, keyboard). One screen or overlay per file.
- Colours only from `bevsidian_ui::color`. Translucent tokens are calibrated for Bevy's
  linear blending; never copy CSS alphas.
- Mockups and the design canvas: `design/`.

## The map

The graph is a hand-drawn star map: `vault/Map.md` places every note (and the map's nebulae,
labels and landmarks); routes are the notes in `vault/Paths/` (`quest: main|feature|mastery`).
Add a new note to the map too, beside its region (the workspace test lists missing ones and
notes placed too close together).

## Code previews

Notes run code with `example=NAME` (README, "Live code previews"). Examples live in
`crates/bevsidian_preview/src/examples/`, one file each; a block runs the example's base
systems plus the functions its code mentions. Keep a note's code in sync with its example. A ```` ```rust pick ```` block lets the reader switch parts of a program on and off (README, "Pick blocks").

## Checks

`cargo fmt`, `cargo clippy --all-targets` (pedantic, keep it warning-free) and
`cargo test --workspace`. After editing notes also run `python3 tools/check_snippets.py`: every Rust block
in `vault/` must compile against Bevy (see README). The workspace test checks links, icons and docs references.

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

## Checks

`cargo fmt`, `cargo clippy --all-targets` (pedantic, keep it warning-free) and
`cargo test --workspace`.

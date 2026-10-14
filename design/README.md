# Design

The UI kit and screen mockups Bevsidian's interface is built from: shadcn/ui's dark *neutral*
theme (Geist, neutral greys, white primary buttons) with one orange accent for links (the mockups predate the switch from violet).

- **Canvas**: <https://claude.ai/artifact/5XrYzBsKFf7zg3LgpzwUxJ> (pan and zoom, interactive
  previews, comments). It is private: ask the owner for access.
- **Sources**: [`mockups/`](mockups) holds the same screens as files: one `.dc.html` per
  artboard (HTML with inline styles, written for the canvas runtime) and `canvas.json`, the
  board layout. Read them for exact sizes, colours and spacing.
- **Code**: the kit is [`crates/bevsidian_ui`](../crates/bevsidian_ui); its README maps
  shadcn/ui's Tailwind classes to the tokens and lists every component.

| Mockup | What it shows | In the app |
| --- | --- | --- |
| `Main.dc.html` | Desktop: sidebar tree, top bar, note with live preview | `src/shell`, `src/pages/note.rs` |
| `Graph.dc.html` | Graph view | `src/pages/graph.rs`, `crates/bevsidian_graph` |
| `Settings.dc.html` | Settings dialog | `src/overlays/settings.rs` |
| `Command.dc.html` | Quick switcher | `src/overlays/command.rs` |
| `Overlays.dc.html` | Context menu, selection toolbar, toast | `src/overlays/` |
| `Mobile.dc.html`, `MobileSheet.dc.html` | Phone note and sidebar drawer | `src/shell` (compact layout) |
| `Kit.dc.html` | Tokens, components, ECS identity | `crates/bevsidian_ui` |

Where the app and a mockup differ, the app follows real data and behaviour: the graph is a
hand-made star map (see `vault/Map.md`: notes as stars with an outline per kind, constellation
lines per route, background stars, a celestial grid, landmarks and optional labels and nebulae,
with the map tokens `MAP_*` in `color.rs`) with a quest log, a checklist and an edit bar instead
of the mockup's filter card and tag nodes; the switcher lists notes only; and settings keep
steppers for text size and scroll speed (any value in range) with a segmented frame-rate limit.

## Colour in Bevy vs. CSS

Bevy blends translucent colours in linear light, CSS in sRGB, so `rgba(255,255,255,0.1)` from
a mockup is far brighter in the app. The tokens in `crates/bevsidian_ui/src/tokens/color.rs`
use alphas that land on the mockup's colours; reuse them rather than copying CSS alphas.

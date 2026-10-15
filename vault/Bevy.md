---
icon: orbit
hub: true
---
# Bevy

Everything you need to make a game with **Bevy 0.20**, in small steps. Notes are short, most code runs right on the page, and each one points to the next. #bevy

> [!TIP]
> Open the **map** (`Ctrl+G`). The white line is the main quest, drawn as Bevy's bird with this note as its eye. Tick what your game needs and the map shows the routes that get you there.

## Areas

| Area | What's inside |
|:--|:--|
| [[Start]] | install, first window, first game |
| [[ECS]] | entities, components, systems: how Bevy thinks |
| [[BSN]] | scenes written as code, new in 0.20 |
| [[Assets]] | images, sounds, models, saving |
| [[Rendering]] | cameras, transforms, colours, shaders |
| [[2D]] | sprites, animation, tilemaps |
| [[3D]] | meshes, lights, models |
| [[Physics]] | movement, collisions, engines |
| [[Audio]] | music and sound effects |
| [[Input]] | keyboard, mouse, gamepad, touch |
| [[UI]] | menus, HUDs, text, buttons |
| [[Gameplay]] | AI, pathfinding, generation, inventory |
| [[Networking]] | multiplayer |
| [[Tooling]] | debugging, profiling, shipping |

Each area ends with a **Libraries** table. Crates are tagged with the Bevy version their latest release supports: #0.20 or #0.19.

## Routes

A route is a reading order, drawn on the map as a coloured line. Pick one in the map's quest log, then use **Previous / Next** at the bottom of each note.

- **Main quest**: [[Your first game]].
- **What's in your game?** [[A 2D game]] · [[A 3D game]] · [[Building UI]] · [[Controls]] · [[Collisions]] · [[Sound and music]] · [[Enemies and AI]] · [[Procedural worlds]] · [[Items and stories]] · [[Multiplayer]] · [[Make it look good]]
- **Get better at**: [[Understanding ECS]] · [[Learn BSN]] · [[Ship it]]

## Reading the map

| You see | It means |
|:--|:--|
| solid line | the route: read in order |
| dashed line | where a route leaves the main quest: choose your path |
| dotted line | optional: skip it if you're in a hurry |
| hexagon · circle · diamond | an area · a lesson · a milestone where you build something |
| pentagon · square | crates compared · one crate |
| dark · grey · orange | unread · read · the note you're on |

[[Map]] stores where everything sits; the map's pencil button edits it.

## Sources

The official [Bevy Book](https://bevy.org/learn/book/intro/) and examples, the API docs (write [`Commands`] in backticks and brackets to link docs.rs), [Tainted Coders](https://taintedcoders.com/), the [Bevy Cheat Book](https://bevy-cheatbook.github.io/) and the [Bevy Quickstart Book](https://thebevyflock.github.io/bevy-quickstart-book/). Every code block is compiled against Bevy by `tools/check_snippets.py`.

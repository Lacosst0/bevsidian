---
icon: orbit
hub: true
---
# Bevy

Everything you need to make a game with **Bevy 0.20**, one small step at a time. Each note is short, has code you can run right on the page, and links to the next thing to learn. #bevy

> [!TIP]
> Open the **Map** (`Ctrl+G`). Bevy is the capital in the middle; the main quest rings it, and a road leaves from it for every kind of game. Tick what your game has, and the map shows the roads to build it.

## The map

| Area | You learn | Start with |
|:--|:--|:--|
| **Start** | install, first window, first game | [[Start]] |
| **ECS** | the heart of Bevy: data and logic | [[ECS]] |
| **BSN** | scenes in code: the new way to spawn | [[BSN]] |
| **Assets** | images, sounds, models, scenes | [[Assets]] |
| **Rendering** | cameras, transforms, colours | [[Rendering]] |
| **2D** | sprites, animation, tilemaps | [[2D]] |
| **3D** | meshes, lights, models | [[3D]] |
| **UI** | menus, text, buttons | [[UI]] |
| **Input** | keyboard, mouse, gamepad, touch | [[Input]] |
| **Audio** | music and sound effects | [[Audio]] |
| **Physics** | movement, collisions, engines | [[Physics]] |
| **Gameplay** | AI, pathfinding, generation, inventory | [[Gameplay]] |
| **Networking** | multiplayer | [[Networking]] |
| **Tooling** | debugging, web, shipping | [[Tooling]] |

Every area ends with a **Libraries** table: the best crates for that topic, and which work with 0.20 today.

## Routes

A route is a list of notes to read in order, drawn on the map as a road. Click one in the map's quest log, then use the **Previous / Next** bar at the bottom of each note.

**Main quest**, always on the map:

- [[Your first game]]: the ring road around Bevy, to a game of your own.

**What's in your game?** Each feature is a road out of the capital:

- [[A 2D game]]: sprites, animation, tilemaps.
- [[A 3D game]]: meshes, light, models.
- [[Building UI]]: menus and HUDs.
- [[Controls]]: gamepads, touch and remapping.
- [[Collisions]]: physics by hand, then an engine.
- [[Sound and music]]: effects and music.
- [[Enemies and AI]]: guards that patrol, chase and find their way.
- [[Procedural worlds]]: levels from rules and seeds.
- [[Items and stories]]: inventory, dialogue and saves.
- [[Multiplayer]]: networking from the first idea.
- [[Make it look good]]: colour, materials and post-processing.

**Get better at**:

- [[Understanding ECS]]: every ECS idea, with the finer points on the side.
- [[Learn BSN]]: Bevy 0.20's scene notation.
- [[Ship it]]: debugging, profiling, release, web and mobile builds.

## How the roads work

| Road | Meaning |
|:--|:--|
| solid | the main road; do these in order |
| dashed | a *side quest*: useful, skip if you are in a hurry |
| dotted | a *secret*: extras for the curious |

Shapes say what a stop is: a hexagon is a region's town (an area's index), a circle a lesson, a diamond a milestone where you have built something, a star the goal at the end of a road and a square a crate. [[Map]] holds where everything sits; edit it in the app with the map's pencil button.

## Where this comes from

- the official [Bevy Book](https://bevy.org/learn/book/intro/) and the examples shipped with the `bevy` crate;
- the API docs: write a name in backticks inside brackets, like [`Commands`], and it links to docs.rs;
- the community guides [Tainted Coders](https://taintedcoders.com/), the [Bevy Cheat Book](https://bevy-cheatbook.github.io/) and the [Bevy Quickstart Book](https://thebevyflock.github.io/bevy-quickstart-book/).

Every code block was compiled against Bevy 0.20 (`tools/check_snippets.py`). Blocks with a play button run for real, inside this page.

---
icon: orbit
hub: true
---
# Bevy

Everything you need to make a game with **Bevy 0.20**, one small step at a time. Each note is short, has code you can run right on the page, and links to the next thing to learn. #bevy

> [!TIP]
> Open the **Map** (`Ctrl+G`). Bevy is the brightest star, in the middle; the main quest spirals out of it, and a route branches off the spiral for every kind of game. Tick what your game has, and the map shows the routes to build it.

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
| **Physics** | movement, collisions, engines | [[Physics]] |
| **Audio** | music and sound effects | [[Audio]] |
| **Input** | keyboard, mouse, gamepad, touch | [[Input]] |
| **UI** | menus, text, buttons | [[UI]] |
| **Gameplay** | AI, pathfinding, generation, inventory | [[Gameplay]] |
| **Networking** | multiplayer | [[Networking]] |
| **Tooling** | debugging, web, shipping | [[Tooling]] |

Every area ends with a **Libraries** table: the best crates for that topic, and which work with 0.20 today.

## Routes

A route is a list of notes to read in order, drawn on the map as a line of stars. Click one in the map's quest log, then use the **Previous / Next** bar at the bottom of each note.

**Main quest**, always on the map:

- [[Your first game]]: the spiral out of Bevy, to a game of your own.

**What's in your game?** Each feature is a route that branches off the main quest:

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

## How the lines work

| Line | Meaning |
|:--|:--|
| solid | the route itself; do these in order (the arrows point the way) |
| dashed | a *side quest*: useful, skip if you are in a hurry |
| dotted | a *secret*: extras for the curious |

Each route has its own colour, on its line and around its notes, like the lines of a metro map; its first page is filled with it. The main quest, and the lines leaving it, are grey. Related areas sit side by side in neighbouring colours.

Every note is a shape with its icon inside, dim until you have read it. The shape says what it is: a hexagon is an area's index, a diamond a milestone where you have built something, a star the goal at the end of a route, a pentagon a comparison of crates, a square a crate and a circle a lesson. [[Map]] holds where everything sits; edit it in the app with the map's pencil button.

## Where this comes from

- the official [Bevy Book](https://bevy.org/learn/book/intro/) and the examples shipped with the `bevy` crate;
- the API docs: write a name in backticks inside brackets, like [`Commands`], and it links to docs.rs;
- the community guides [Tainted Coders](https://taintedcoders.com/), the [Bevy Cheat Book](https://bevy-cheatbook.github.io/) and the [Bevy Quickstart Book](https://thebevyflock.github.io/bevy-quickstart-book/).

Every code block was compiled against Bevy 0.20 (`tools/check_snippets.py`). Blocks with a play button run for real, inside this page.

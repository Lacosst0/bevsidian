---
icon: orbit
hub: true
tags: [start-here]
---
# Bevy

Everything you need to make a game with **Bevy 0.20**, one small step at a time. Each note is short, has code you can run right on the page, and links to the next thing to learn. #bevy #beginner

> [!TIP]
> Open the **Graph** (`Ctrl+G`). Bevy sits in the middle; every area is a branch. Pick a path at the top of the graph to see its steps in order.

## The map

| Area | You learn | Start with |
|:--|:--|:--|
| **Start** | install, first window, plugins | [[Hello Bevy]] |
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
| **Networking** | multiplayer | [[Networking]] |
| **Tooling** | debugging, web, shipping | [[Tooling]] |
| **Libraries** | the best crates, one note each | [[Libraries]] |

## Paths

A path is a list of notes to read in order. Click one, then use the **Previous / Next** bar at the bottom of each note.

- [[Your first game]]: the fastest way to something playable.
- [[Getting started]]: the core of Bevy, from data to pixels.
- [[Understanding ECS]]: every ECS idea, with the advanced parts on the side.
- [[Learn BSN]]: Bevy 0.20's scene notation.
- [[A 2D game]]: sprites, animation, input, sound.
- [[A 3D game]]: meshes, lights, cameras.
- [[Multiplayer]]: networking from the first idea.
- [[Building UI]]: menus and HUDs.
- [[Ship it]]: web builds, release builds, profiling.
- [[Make it look good]]: colour, light and post-processing.

## How the lines work

In a path, the lines mean:

| Line | Meaning |
|:--|:--|
| solid | the main route; do these in order |
| dashed | *optional*: useful, skip if you are in a hurry |
| dotted | *extra*: for the curious |

In the normal graph, solid is "links both ways", dashed "links one way" and dotted "share a tag".

## Where this comes from

- the official [Bevy Book](https://bevy.org/learn/book/intro/) and the examples shipped with the `bevy` crate;
- the API docs: write a name in backticks inside brackets, like [`Commands`], and it links to docs.rs;
- the community guides [Tainted Coders](https://taintedcoders.com/), the [Bevy Cheat Book](https://bevy-cheatbook.github.io/) and the [Bevy Quickstart Book](https://thebevyflock.github.io/bevy-quickstart-book/).

Every code block was compiled against Bevy 0.20 (`tools/check_snippets.py`). Blocks with a play button run for real, inside this page.

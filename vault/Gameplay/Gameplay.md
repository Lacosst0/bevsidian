---
icon: swords
tags: [gameplay]
---
# Gameplay

Bevy gives you the engine. These notes are about what you build **on** it: the systems that most games need and that no engine ships for you. Every one is a small amount of plain Bevy code you can read and change. #gameplay

| System | You learn | Preview |
|:--|:--|:--|
| [[Game AI]] | state machines, senses, steering, utility AI | a guard that patrols and chases |
| [[Pathfinding]] | A* on a grid, following a path | an agent finding its way round walls |
| [[Randomness]] | seeds, weighted picks, fair luck | |
| [[Procedural generation]] | caves, tunnels, noise, terrain | a cave growing before your eyes |
| [[Inventory and items]] | stacks, pickups, equipment | |
| [[Dialogue]] | a script graph, a typewriter, choices | |
| [[Game feel]] | shake, hit stop, easing, squash | |
| [[Accessibility and localization]] | settings, screen readers, translations | |

## How they fit together

A typical action game loop touches most of them:

1. **Procedural generation** makes a level from a seed.
2. **Pathfinding** and **Game AI** make the enemies move with a purpose.
3. **Randomness** decides what they drop; **Inventory** stores it.
4. **Dialogue** gives the world a voice.
5. **Game feel** makes every hit land.
6. **Accessibility and localization** lets more people play it.

## The pattern behind all of them

Each is the same shape:

- **Data** in a component or resource (a state, a grid, a bag of items).
- **A system** that reads the data and changes it.
- **A message or event** when something happens that others care about.

If you are stuck on how to build a feature, ask what the data is. See [[ECS]].

## Where to start

Follow the path [[Gameplay systems]].

## Libraries

Status for **Bevy 0.20**, checked 2026-10-13: **released** (a crates.io release uses 0.20) · **git** (the `main` branch is on 0.20; use a git dependency) · **pending** (the latest release targets an older Bevy) · **built in** (Bevy has it now). Check crates.io before you start.

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[Randomness#bevy_rand|bevy_rand]] | seeded random numbers | released `0.16` |
| `bevy_northstar`, `vleue_navigator` | pathfinding, navmeshes | pending `0.7`, `0.16` |
| `bevy_yarnspinner` | Yarn dialogue | pending `0.9` |
| `bevy_behave` | behaviour trees | pending `0.5` (Bevy 0.18) |

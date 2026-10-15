---
icon: swords
tags: [gameplay]
---
# Gameplay

The engine is Bevy's job; the game is yours. These notes cover the systems most games need and no engine ships: each is a little plain Bevy code you can read and bend. #gameplay

| System | You learn | Preview |
|:--|:--|:--|
| [[Game AI]] | state machines, senses, steering, utility AI | a guard that patrols and chases |
| [[Pathfinding]] | A* on a grid, following a path | an agent finding its way round walls |
| [[Randomness]] | seeds, weighted picks, fair luck | rocks scattered by a seed |
| [[Procedural generation]] | caves, tunnels, noise, terrain | a cave growing before your eyes |
| [[Inventory and items]] | stacks, pickups, equipment | |
| [[Dialogue]] | a script graph, a typewriter, choices | |
| [[Game feel]] | shake, hit stop, easing, squash | a hit you can feel |
| [[Accessibility and localization]] | settings, screen readers, translations | |

## How they fit together

In a typical action game, **procedural generation** builds the level from a seed, **pathfinding** and **AI** give enemies purpose, **randomness** picks their drops and the **inventory** keeps them, **dialogue** gives the world a voice, and **game feel** makes every hit land.

## The pattern behind all of them

Every one is **data** (a state, a grid, a bag of items), **a system** that changes it, and **an event** when others should know. Stuck on a feature? Ask what the data is first ([[ECS]]).

## Where to start

Three routes cross this area: [[Enemies and AI]], [[Procedural worlds]] and [[Items and stories]].

## Libraries

| Crate | For | Bevy |
|:--|:--|:--|
| [[bevy_rand]] | seeded random numbers | #0.20 |
| `bevy_northstar`, `vleue_navigator` | pathfinding, navmeshes | #0.19 |
| `bevy_yarnspinner` | Yarn dialogue | #0.19 |
| `bevy_behave` | behaviour trees | #0.19 |

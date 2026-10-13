---
icon: braces
tags: [bsn, start-here]
---
# BSN

**BSN** (*Bevy Scene Notation*, said "bison") is new in Bevy 0.20 and is the biggest change to how you build things. Instead of spawning one entity at a time and wiring them by hand, you **describe** a tree of entities, with components, children, observers and assets, and spawn it in one go. #bsn

Everything on the screen of this app is BSN.

## What you get

| You want | BSN gives you |
|:--|:--|
| a tree in one place | `Children [ A -- B ]` |
| to reuse a piece | a plain function that returns `impl Scene` |
| a variant of that piece | **patching**: change one field, keep the rest |
| assets without `AssetServer` plumbing | `Sprite { image: "player.png" }` |
| click handlers next to the thing clicked | `on(\|e: On<..>\| ..)` |
| a widget with a body and settings | a **scene component** with props |

## The notes, in order

1. [[Scenes and BSN]]: your first scene, children, patching.
2. [[BSN syntax]]: every piece of notation, one table.
3. [[Composing scenes]]: reuse, parameters, lists, conditions.
4. [[Templates and assets]]: why some types need `FromTemplate`, and how paths become handles.
5. [[Scene components]]: scenes with props, and when to prefer required components.
6. [[Spawning scenes]]: every way to put a scene in the world.

Then see it used for real in [[UI layout]], [[UI kit]] and [[bevy_radioactive]] (reactive scenes).

## Plain `spawn` or BSN?

Both work together. A quick rule:

- One entity, a few components: `commands.spawn((A, B))`.
- A **tree**, anything **reused**, anything with **assets or observers**: BSN.

> [!IMPORTANT]
> The `.bsn` *file* format is not in Bevy 0.20 yet. The `bsn!` macro is the supported way, and what these notes teach.

---
icon: library
tags: [lib, start-here]
---
# Libraries

Bevy is small on purpose; the ecosystem fills the gaps. This is the map: what each crate is for, whether it works with **Bevy 0.20 today**, and which to pick when two compete. #lib

> [!IMPORTANT]
> **Status checked on 2026-10-13**, five days after Bevy 0.20 shipped, from crates.io and each project's repository. Most crates need a few weeks to catch up. The **Bevy 0.20** column says:
> - ✅ **released**: a crates.io release depends on Bevy 0.20;
> - 🌿 **git**: the `main` branch is on 0.20, the release is pending (use a `git` dependency);
> - ⏳ **pending**: the latest release is for 0.19 (or older);
> - 🚫 **retired**: unmaintained or replaced by something built into Bevy.
>
> Check crates.io for the newest number before you start. Notes marked *0.19* were compiled against Bevy 0.19.

## How to read a note

Each library note has: what it does, a status box, a **short example that was compiled**, when to choose it and the alternatives.

## Input

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_enhanced_input]] | actions, contexts, modifiers; **observer based** | 🌿 git |
| [[leafwing-input-manager]] | action enum + `InputMap`, remapping | 🌿 git |

## Physics

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_rapier]] | the Rapier engine, 2D and 3D | ✅ `0.37` |
| [[Avian]] | ECS-native engine, 2D and 3D | ⏳ `0.7` |
| [[bevy_transform_interpolation]] | smooth fixed-timestep motion | ⏳ `0.5` |

## Networking

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_replicon]] | server-authoritative replication | ✅ `0.45` |
| [[lightyear]] | replication + prediction + interpolation | ⏳ `0.30` |
| [[bevy_renet]] | UDP transport with channels | ⏳ `5.0` |
| [[bevy_ggrs]] | rollback, peer to peer | ⏳ `0.22` |
| [[bevy_matchbox]] | WebRTC peers + signalling | ⏳ `0.14` (Bevy 0.18) |
| [[bevy_steamworks]] | Steam features | ⏳ `0.17` |
| [[bevy_http_client]] | HTTP requests | ✅ `0.12` |

## UI and tools

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_egui]] | immediate-mode UI for tools | ✅ `0.43` |
| [[bevy-inspector-egui]] | live world inspector | ⏳ `0.37` |
| [[bevy_mod_debugdump]] | draw the schedule graph | ⏳ `0.16` |
| [[bevy_hui]] | UI in HTML-like files | ✅ `0.8` |
| [[bevy_lunex]] | retained layout engine, world-space UI | 🌿 git |
| [[bevy_simple_text_input]] | text field | 🚫 use built-in `EditableText` |
| [[bevy_cobweb_ui]] | scripted UI framework | 🚫 unmaintained |

## Assets and saving

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_asset_loader]] | declare assets in a struct, loading states | ✅ `0.28` |
| [[bevy_common_assets]] | RON / JSON / TOML... as assets | ✅ `0.18` |
| [[bevy_pkv]] | key-value saves, desktop and web | ⏳ `0.16` |

## Audio

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_kira_audio]] | channels, tweens, effects | ✅ `0.27` |
| [[bevy_seedling]] | audio graph, effects | ⏳ `0.8` |
| [[bevy_fmod]] | FMOD middleware | ⏳ `0.11` (Bevy 0.19) |

## 2D

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_ecs_tilemap]] | tilemap engine | ⏳ `0.19` |
| [[bevy_ecs_ldtk]] | LDtk levels | ⏳ `0.15` |
| [[bevy_tweening]] | tweens and easing | ⏳ `0.16` |
| [[bevy_spritesheet_animation]] | sprite-sheet clips | ⏳ `7.0` |
| [[bevy_prototype_lyon]] | vector paths and strokes | ⏳ `0.17` |
| [[bevy_vector_shapes]] | painter-style shapes | ✅ `0.14` |
| [[bevy_hanabi]] | GPU particles | ⏳ `0.19` |
| [[bevy_light_2d]] | 2D lights and shadows | ✅ `0.11` |
| [[bevy_firefly]] | richer 2D lighting | ⏳ `0.19` |
| [[bevy_pancam]] | pan and zoom camera | ⏳ `0.21` |

## 3D

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_skein]] | Blender components via glTF | ✅ `0.7.0-rc.1` |
| [[bevy_trenchbroom]] | Quake-style level editor | ⏳ `0.14` |
| [[bevy_mod_outline]] | mesh outlines | ⏳ `0.13` |
| [[bevy_water]] | animated water | ⏳ `0.18` (Bevy 0.18) |
| [[bevy_rich_text3d]] | rich text as meshes | ✅ `0.8` |
| [[bevy_panorbit_camera]] | orbit camera | 🚫 built into Bevy 0.20 |
| [[bevy_atmosphere]] | procedural sky | 🚫 built into Bevy 0.20 |

## Gameplay

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| [[bevy_rand]] | seeded random numbers | ✅ `0.16` |

## More worth knowing (no note yet)

| Crate | For | Bevy 0.20 |
|:--|:--|:--|
| `bevy_mod_scripting` | Lua, Rhai and other scripting | ✅ `0.22` |
| `big_space` | huge worlds without float jitter | ✅ `0.14.0-rc.0` |
| `bevy_northstar`, `vleue_navigator` | pathfinding, navmeshes | ⏳ `0.7`, `0.16` |
| `bevy_yarnspinner` | Yarn dialogue | ⏳ `0.9` |
| `bevy_behave` | behaviour trees | ⏳ `0.5` (Bevy 0.18) |
| `bevy_voxel_world` | voxel terrain | ⏳ `0.17` |
| `bevy_gaussian_splatting` | 3D gaussian splats | ⏳ `8.0` |
| `bevy_sprite3d` | sprites inside 3D | ⏳ `9.0` |
| `bevy_mod_picking` | old picking crate | 🚫 built into Bevy ([[Picking]]) |

## Rules of thumb

1. **Check the compatibility table** in the crate's README before adding it.
2. **Prefer built-in** when it is enough ([[Picking]], camera controllers, text input, atmosphere all moved into Bevy).
3. **Prefer a released crate** to a git branch; use git only for a crate you need now.
4. If two crates do the same job, read both examples and choose the one whose *style* you like: that is what you will live with.
5. Do not add libraries speculatively. Add one when you feel the pain it removes.

## Writing your own

A good Bevy library is **one plugin**, configured through resources, not a framework. The Bevy Book's chapter "Creating libraries for Bevy" has the checklist.

## Next

Pick an area from [[Bevy]], or follow a path.

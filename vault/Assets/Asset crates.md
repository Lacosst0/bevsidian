---
parent: Assets
icon: sliders-horizontal
tags: [assets]
---
# Asset crates

Two crates that make assets less manual. They solve different halves of the problem and are often used together.

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_asset_loader]] | declare assets in a struct and load them during a loading state | #0.20 | you have many assets and a loading screen |
| [[bevy_common_assets]] | load RON, JSON, TOML, YAML and more as assets | #0.20 | game data lives in files designers edit |

## Which one?

- A loading screen with lots of images and sounds: [[bevy_asset_loader]].
- Items, levels or settings in data files: [[bevy_common_assets]].
- Both: `bevy_asset_loader` can load the data files that `bevy_common_assets` understands.

## Next

[[Assets]], [[Custom assets]].

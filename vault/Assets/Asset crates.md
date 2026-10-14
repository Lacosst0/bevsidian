---
parent: Assets
icon: sliders-horizontal
tags: [assets]
---
# Asset crates

Two crates that make assets less manual. They solve different halves of the problem and are often used together.

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy_asset_loader]] | declare assets in a struct and load them during a loading state | released `0.28` | you have many assets and a loading screen |
| [[bevy_common_assets]] | load RON, JSON, TOML, YAML and more as assets | released `0.18` | game data lives in files designers edit |

## Which one?

- A loading screen with lots of images and sounds: [[bevy_asset_loader]].
- Items, levels or settings in data files: [[bevy_common_assets]].
- Both: `bevy_asset_loader` can load the data files that `bevy_common_assets` understands.

## Next

[[Assets]], [[Custom assets]].

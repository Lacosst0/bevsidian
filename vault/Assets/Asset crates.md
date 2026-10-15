---
parent: Assets
icon: sliders-horizontal
tags: [assets]
---
# Asset crates

Two crates that take the busywork out of assets. They handle different halves and are often used together.

| | [[bevy_asset_loader]] | [[bevy_common_assets]] |
|:--|:--|:--|
| Bevy | #0.20 | #0.20 |
| Does | loads a struct of assets during a loading state | reads RON, JSON, TOML, YAML… as assets |
| Saves you | tracking handles and "is it loaded yet?" | writing an asset loader per format |
| Shines with | many images and sounds, a loading screen | items, levels, settings in data files |

> [!TIP]
> - A loading screen: [[bevy_asset_loader]].
> - Game data designers can edit: [[bevy_common_assets]].
> - Both: `bevy_asset_loader` happily loads the files `bevy_common_assets` understands.

## Next

[[Assets]], [[Custom assets]].

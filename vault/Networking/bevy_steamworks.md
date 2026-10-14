---
icon: package
tags: [networking]
check: skip
---
# bevy_steamworks

The **Steamworks SDK** for Bevy: achievements, leaderboards, cloud saves, lobbies, matchmaking, Steam networking and the overlay. For games released on Steam. #networking

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.17` targets **Bevy 0.19**.

```toml
bevy_steamworks = "0.17"
```

## Setup in one paragraph

Add `SteamworksPlugin::init_app(YOUR_APP_ID)` before `DefaultPlugins`, put `steam_appid.txt` with your id next to the executable while developing, and read the `Client` resource to call Steam. Run the Steam client first. The crate bundles the Rust `steamworks` bindings, so you need the redistributable SDK files at build time (see the README).

## What to do with it

| Feature | Use |
|:--|:--|
| Achievements & stats | `client.user_stats()` |
| Cloud saves | `client.remote_storage()` |
| Lobbies & matchmaking | `client.matchmaking()` |
| Networking | pair with [[bevy_replicon]] or [[lightyear]], which have Steam transports |

## Links

[crates.io](https://crates.io/crates/bevy_steamworks) · [docs.rs](https://docs.rs/bevy_steamworks) · [Steamworks docs](https://partner.steamgames.com/doc/sdk)

## Next

[[Networking]], [[Release builds]].

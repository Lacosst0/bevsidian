---
parent: Networking
icon: sliders-horizontal
tags: [networking]
---
# Netcode crates

Four crates for different kinds of multiplayer. The right one depends on the game, so this is a guide by game type. (For the basics see [[bevy_replicon]] and [[bevy_renet]].)

> [!NOTE]
> Status checked 2026-10-13. *released* means a crates.io release uses Bevy 0.20; *pending* means the latest release targets an older Bevy; *git* means the `main` branch is on 0.20.

| Crate | For | Bevy 0.20 | Pick it when |
|:--|:--|:--|:--|
| [[bevy_ggrs]] | rollback networking for peer-to-peer games | pending `0.22` | fighting games and other tightly synchronised duels |
| [[bevy_matchbox]] | WebRTC peer connections with a signalling server | pending `0.14` (Bevy 0.18) | peers must connect from the browser; the usual partner of GGRS |
| [[bevy_steamworks]] | achievements, lobbies, matchmaking, cloud saves | pending `0.17` | the game ships on Steam |
| [[lightyear]] | server-authoritative framework with prediction and interpolation | pending `0.30` | a fast action game with a dedicated server |

## Which one?

- Two to four players, deterministic, lowest latency: [[bevy_ggrs]] with [[bevy_matchbox]] for the connections.
- Many players with a server in charge and prediction: [[lightyear]]. If you cannot wait, [[bevy_replicon]] is released for 0.20.
- Steam features around any of these: [[bevy_steamworks]].

## Next

[[Networking]], [[Multiplayer]].

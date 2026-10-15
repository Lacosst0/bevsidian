---
parent: Networking
icon: sliders-horizontal
tags: [networking]
---
# Netcode crates

Four crates for different kinds of multiplayer. The right one depends on the game, so this is a guide by game type. (For the basics see [[bevy_replicon]] and [[bevy_renet]].)

> [!NOTE]
| Crate | For | Bevy | Pick it when |
|:--|:--|:--|:--|
| [[bevy_ggrs]] | rollback networking for peer-to-peer games | #0.19 | fighting games and other tightly synchronised duels |
| [[bevy_matchbox]] | WebRTC peer connections with a signalling server | #0.19 | peers must connect from the browser; the usual partner of GGRS |
| [[bevy_steamworks]] | achievements, lobbies, matchmaking, cloud saves | #0.19 | the game ships on Steam |
| [[lightyear]] | server-authoritative framework with prediction and interpolation | #0.19 | a fast action game with a dedicated server |

## Which one?

- Two to four players, deterministic, lowest latency: [[bevy_ggrs]] with [[bevy_matchbox]] for the connections.
- Many players with a server in charge and prediction: [[lightyear]]. If you cannot wait, [[bevy_replicon]] works with #0.20.
- Steam features around any of these: [[bevy_steamworks]].

## Next

[[Networking]], [[Multiplayer]].

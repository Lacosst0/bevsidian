---
parent: Networking
icon: sliders-horizontal
tags: [networking]
---
# Netcode crates

The right crate depends on the kind of game more than anything, so here they are by game type. The basics are in [[bevy_replicon]] and [[bevy_renet]].

| | [[bevy_ggrs]] | [[bevy_matchbox]] | [[lightyear]] | [[bevy_steamworks]] |
|:--|:--|:--|:--|:--|
| Bevy | #0.19 | #0.19 | #0.19 | #0.19 |
| Does | rollback for peer-to-peer | WebRTC connections | server, prediction, interpolation | Steam lobbies, achievements, saves |
| Players | 2–4 | any | many | any |
| Best for | fighting games, tight duels | browser games; GGRS's transport | fast action with a server | games on Steam |

> [!TIP]
> - A fighting game or a duel: [[bevy_ggrs]] over [[bevy_matchbox]].
> - Many players and a server in charge: [[lightyear]], or [[bevy_replicon]] if you need #0.20 now.
> - On Steam: add [[bevy_steamworks]] to any of these.

## Next

[[Networking]], [[Multiplayer]].

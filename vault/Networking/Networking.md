---
icon: globe
tags: [networking]
crates: serde={ version = "1", features = ["derive"] }
---
# Networking

Multiplayer is a *design* problem first and a library problem second. This note covers the ideas; the crates do the plumbing. #networking

## Two ways to agree on a world

| Model | How | Good for |
|:--|:--|:--|
| **Client / server** | one program owns the truth; clients send *inputs*, receive *state* | most games, and cheating is hard |
| **Peer to peer** (lockstep, rollback) | everyone runs the same simulation from the same inputs | fighting games, RTS; the simulation must be deterministic |

## Words you'll meet

- **Transport**: how bytes travel. UDP (fast, lossy), TCP (reliable, slower), WebSocket / WebTransport (browsers), Steam relays.
- **Replication**: the server tells clients which entities exist and what changed.
- **Authority**: who may change a thing. Usually the server.
- **Prediction**: apply your own input at once, without waiting for the server.
- **Reconciliation**: fix the prediction when the server's answer differs.
- **Interpolation**: draw *other* players slightly in the past, blended between snapshots, so they move smoothly.
- **Tick rate**: server simulations per second; run them in `FixedUpdate` ([[Time and timers]]).

## What goes over the wire

Inputs one way, facts the other. Never the whole world:

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Client → server: what the player wants.
#[derive(Serialize, Deserialize, Message, Clone)]
struct PlayerInput {
    movement: Vec2,
    jump: bool,
}

/// Server → clients: the facts. Replication crates send these for you.
#[derive(Component, Serialize, Deserialize, Clone)]
struct Health(u32);
```

Replicated components are copied to clients automatically, and [[Messages]] and [[Events and observers|events]] become network messages. Your ECS world is the shared database.

## Building it, in order

- [ ] Single-player first, with **input → simulation → rendering** kept apart ([[Project setup#Project structure|Project structure]]).
- [ ] Simulation on a fixed tick, taking inputs as data.
- [ ] Add replication; mark what the server owns.
- [ ] Prediction only where lag hurts: your own player.
- [ ] Test with fake lag (`tc`, `clumsy`) from day one.

> [!WARNING]
> Browsers can't open raw UDP sockets. For a [[Web build]], pick a transport that speaks WebSocket, WebTransport or WebRTC.

## Libraries

Side by side, by kind of game: [[Netcode crates]].

| Crate | For | Bevy |
|:--|:--|:--|
| [[bevy_replicon]] | server-authoritative replication, any transport | #0.20 |
| [[lightyear]] | replication + prediction + interpolation | #0.19 |
| [[bevy_renet]] | UDP transport with channels | #0.19 |
| [[bevy_ggrs]] | rollback, peer to peer | #0.19 |
| [[bevy_matchbox]] | WebRTC peers and signalling | #0.19 |
| [[bevy_steamworks]] | Steam lobbies, achievements | #0.19 |
| [[bevy_http_client]] | HTTP: leaderboards, accounts | #0.20 |

## Next

[[bevy_replicon]] to try it for real, or follow [[Multiplayer]].

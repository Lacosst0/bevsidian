---
icon: globe
tags: [networking, advanced]
crates: serde={ version = "1", features = ["derive"] }
---
# Networking

Making a game multiplayer is mostly a *design* problem, and only then a library problem. This note gives you the concepts; the libraries do the plumbing. #networking #advanced

## The big idea

Several computers must agree on one game world. Two ways:

| Model | How | Good for |
|:--|:--|:--|
| **Client / server** (authoritative) | one program owns the truth; clients send *inputs* and receive *state* | most games; cheating is hard |
| **Peer to peer, lockstep / rollback** | everyone simulates the same game from the same inputs | fighting games, RTS; needs a deterministic simulation |

## Words you will meet

- **Transport**: how bytes move. UDP (fast, may lose packets), TCP (reliable, slow), WebTransport/WebSocket (browsers), Steam/EOS relays.
- **Replication**: the server tells clients which entities and components exist and what changed.
- **Authority**: who is allowed to change a thing. Usually the server.
- **Prediction**: the client applies its own input at once, without waiting, so it feels instant.
- **Reconciliation**: when the server's answer arrives, correct the prediction if it differed.
- **Interpolation**: show *other* players slightly in the past, blended between two snapshots, so they move smoothly.
- **Tick rate**: how many times per second the server simulates (use [[Time and timers|FixedUpdate]]).

## What goes over the wire

Not the whole world. Small **messages**:

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Client → server: what the player wants.
#[derive(Serialize, Deserialize, Message, Clone)]
struct PlayerInput {
    movement: Vec2,
    jump: bool,
}

/// Server → clients: the facts. Replication libraries do this part for you.
#[derive(Component, Serialize, Deserialize, Clone)]
struct Health(u32);
```

Components marked as *replicated* are copied to clients automatically; [[Events and observers|events]] and [[Messages]] become network messages. Think of your ECS world as the shared database.

## Pick a library

Write the game so the **same systems** run on server and client wherever you can.

| Library | What it gives | Pick it when | Bevy 0.20 |
|:--|:--|:--|:--|
| [[bevy_replicon]] | replication + events, transport-agnostic | you want a clean, simple, modern base | **released** |
| [[lightyear]] | replication, prediction, interpolation, lag compensation | action games that need prediction | in progress |
| [[bevy_renet]] | reliable channels over UDP | you want a transport and make the rest yourself | in progress |
| [[bevy_ggrs]] + [[bevy_matchbox]] | rollback + WebRTC matchmaking | fighters, small peer-to-peer games | in progress |
| [[bevy_http_client]] | HTTP requests | leaderboards, accounts, REST | **released** |
| [[bevy_steamworks]] | Steam lobbies, achievements | shipping on Steam | in progress |

See [[Libraries]] for the live version table.

## A sane way to build it

1. Make the game single-player, with clean separation of **input → simulation → rendering** ([[Project structure]]).
2. Make the simulation run on a fixed tick and take inputs as data.
3. Add a replication library; mark what the server owns.
4. Add prediction only where it hurts (your own player).
5. Test with fake lag (`tc`, `clumsy`) from the first day.

Follow the path [[Multiplayer]] for the order.

## Browsers

WebSockets and WebTransport work in wasm; raw UDP does not. Pick a library and transport that support the web if you ship there ([[Web build]]).

## Next

[[bevy_replicon]] to try it for real.

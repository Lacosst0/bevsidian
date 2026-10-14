---
icon: package
tags: [networking]
check: skip
---
# lightyear

A **full server-authoritative multiplayer framework**: replication, **client-side prediction with rollback**, **snapshot interpolation**, input buffering, lag compensation, bandwidth management. Runs over UDP, WebSocket, WebTransport (browsers) or Steam. #networking

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.30` (and `main`) target **Bevy 0.19**. Use [[bevy_replicon]] (released for 0.20) if you cannot wait; the concepts below carry over.

```toml
lightyear = "0.30"      # for Bevy 0.19
```

## What you get

| Feature | In words |
|:--|:--|
| Replication | the server's entities and components appear on clients (it builds on `bevy_replicon`) |
| **Prediction** | your own player moves instantly from your input; the server's answer corrects it (rollback) |
| **Interpolation** | other players are shown smoothly between snapshots |
| Inputs | buffered per tick and sent redundantly so a lost packet doesn't drop a key press; integrates with [[bevy_enhanced_input]] and [[leafwing-input-manager]] |
| Lag compensation | hit detection that accounts for what the shooter saw |
| Topologies | client-server, host-client (listen server), peer-to-peer |
| Web | compatible with WASM via WebTransport |

## Shape of a game

1. A shared **protocol** crate/module: registers components, messages and inputs that cross the network.
2. A **server** app and a **client** app (or one binary with a mode).
3. Shared **simulation systems** run in `FixedUpdate` on both sides, so prediction matches the server.
4. Mark your player as `Predicted` on its owner and `Interpolated` on the others.

The best way in is the official [book](https://cbournhonesque.github.io/lightyear/book/) and the `simple_box` example, which the tutorial rebuilds step by step.

## When to choose it

Fast action games (shooters, platformers, racing) where **latency hiding** matters. For turn-based or slow games, plain [[bevy_replicon]] is simpler.

## Links

[crates.io](https://crates.io/crates/lightyear) · [docs.rs](https://docs.rs/lightyear) · [book](https://cbournhonesque.github.io/lightyear/book/) · [repo](https://github.com/cBournhonesque/lightyear)

## Next

[[Networking]], [[Multiplayer]].

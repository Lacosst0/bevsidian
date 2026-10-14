---
lib: true
icon: package
tags: [networking]
---
# Netcode crates

Four crates beyond the basics: rollback for fighting games, WebRTC peers, Steam, and a full prediction framework. Start with [[Networking]] to pick.

## bevy_ggrs

**Rollback networking** (GGPO-style) for peer-to-peer games: everyone runs the same deterministic simulation and exchanges only *inputs*. When a late input arrives, the game rolls back, replays, and catches up. Standard for fighting games and tight 1v1/2v2. #networking

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.22` targets **Bevy 0.19**. Pair it with [[Netcode crates#bevy_matchbox|bevy_matchbox]] for browser-friendly connections.

```toml
bevy_ggrs = "0.22"
```

### The contract

Rollback only works if your game is **deterministic**: same state + same inputs = same result on every machine. That means:

- game logic in the `GgrsSchedule`, driven by the crate at a fixed rate;
- no unseeded randomness (see [[Randomness#bevy_rand|bevy_rand]] with a shared seed);
- no dependence on frame time or hash-map iteration order;
- every component and resource that matters is registered for **snapshots** so it can be saved and restored.

```rust ignore
// From the README: the shape of a rollback app.
App::new()
    .add_plugins(GgrsPlugin::<GgrsConfig>::default())
    .insert_resource(RollbackFrameRate(60))
    .rollback_component_with_clone::<Transform>()      // saved/restored on rollback
    .add_systems(ReadInputs, read_local_inputs)         // collect this player's input
    .add_systems(GgrsSchedule, move_players)            // deterministic game logic
    .insert_resource(Session::SyncTest(session))        // or a P2P / spectator session
    .run();

#[derive(Component)]
#[require(Rollback)]                                    // marks entities to snapshot
struct Player;
```

Develop with a **SyncTest** session first: it rolls back every frame and reports any desync.

### Compared

| | Rollback (this) | Server-authoritative ([[bevy_replicon]], [[Netcode crates#lightyear\|lightyear]]) |
|:--|:--|:--|
| Sends | inputs only | state |
| Server | none needed (peer-to-peer) | required |
| Cheat resistance | low | high |
| Good for | fighting, small player counts | most other games |

### Links

[crates.io](https://crates.io/crates/bevy_ggrs) · [docs.rs](https://docs.rs/bevy_ggrs) · [repo](https://github.com/gschup/bevy_ggrs)

## bevy_matchbox

**Peer-to-peer connections over WebRTC**, in the browser and natively. A small *signalling server* introduces the peers, then they talk directly. The usual partner of [[Netcode crates#bevy_ggrs|bevy_ggrs]]. #networking

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.14` targets **Bevy 0.18**; updates for newer Bevy are pending.

```toml
bevy_matchbox = { version = "0.14", features = ["ggrs"] }
```

### How it works

1. Run (or host) a signalling server (`matchbox_server`).
2. Each client opens a socket to a *room* URL: `ws://server:3536/my_room?next=2` matches two players.
3. The socket reports peers and gives you channels to send and receive.

```rust ignore
// From the README (Bevy 0.18): open a socket in a system.
fn start_socket(mut commands: Commands) {
    let socket = MatchboxSocket::new_unreliable("ws://localhost:3536/extreme_bevy?next=2");
    commands.insert_resource(socket);
}
```

### Why WebRTC

Browsers cannot send raw UDP. WebRTC data channels give nearly-UDP behaviour in the browser, with NAT traversal, so P2P works on the web ([[Web build]]).

### Links

[crates.io](https://crates.io/crates/bevy_matchbox) · [docs.rs](https://docs.rs/bevy_matchbox) · [repo](https://github.com/johanhelsing/matchbox)

## bevy_steamworks

The **Steamworks SDK** for Bevy: achievements, leaderboards, cloud saves, lobbies, matchmaking, Steam networking and the overlay. For games released on Steam. #networking

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.17` targets **Bevy 0.19**.

```toml
bevy_steamworks = "0.17"
```

### Setup in one paragraph

Add `SteamworksPlugin::init_app(YOUR_APP_ID)` before `DefaultPlugins`, put `steam_appid.txt` with your id next to the executable while developing, and read the `Client` resource to call Steam. Run the Steam client first. The crate bundles the Rust `steamworks` bindings, so you need the redistributable SDK files at build time (see the README).

### What to do with it

| Feature | Use |
|:--|:--|
| Achievements & stats | `client.user_stats()` |
| Cloud saves | `client.remote_storage()` |
| Lobbies & matchmaking | `client.matchmaking()` |
| Networking | pair with [[bevy_replicon]] or [[Netcode crates#lightyear\|lightyear]], which have Steam transports |

### Links

[crates.io](https://crates.io/crates/bevy_steamworks) · [docs.rs](https://docs.rs/bevy_steamworks) · [Steamworks docs](https://partner.steamgames.com/doc/sdk)

## lightyear

A **full server-authoritative multiplayer framework**: replication, **client-side prediction with rollback**, **snapshot interpolation**, input buffering, lag compensation, bandwidth management. Runs over UDP, WebSocket, WebTransport (browsers) or Steam. #networking

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.30` (and `main`) target **Bevy 0.19**. Use [[bevy_replicon]] (released for 0.20) if you cannot wait; the concepts below carry over.

```toml
lightyear = "0.30"      # for Bevy 0.19
```

### What you get

| Feature | In words |
|:--|:--|
| Replication | the server's entities and components appear on clients (it builds on `bevy_replicon`) |
| **Prediction** | your own player moves instantly from your input; the server's answer corrects it (rollback) |
| **Interpolation** | other players are shown smoothly between snapshots |
| Inputs | buffered per tick and sent redundantly so a lost packet doesn't drop a key press; integrates with [[Input crates#bevy_enhanced_input\|bevy_enhanced_input]] and [[Input crates#leafwing-input-manager\|leafwing-input-manager]] |
| Lag compensation | hit detection that accounts for what the shooter saw |
| Topologies | client-server, host-client (listen server), peer-to-peer |
| Web | compatible with WASM via WebTransport |

### Shape of a game

1. A shared **protocol** crate/module: registers components, messages and inputs that cross the network.
2. A **server** app and a **client** app (or one binary with a mode).
3. Shared **simulation systems** run in `FixedUpdate` on both sides, so prediction matches the server.
4. Mark your player as `Predicted` on its owner and `Interpolated` on the others.

The best way in is the official [book](https://cbournhonesque.github.io/lightyear/book/) and the `simple_box` example, which the tutorial rebuilds step by step.

### When to choose it

Fast action games (shooters, platformers, racing) where **latency hiding** matters. For turn-based or slow games, plain [[bevy_replicon]] is simpler.

### Links

[crates.io](https://crates.io/crates/lightyear) · [docs.rs](https://docs.rs/lightyear) · [book](https://cbournhonesque.github.io/lightyear/book/) · [repo](https://github.com/cBournhonesque/lightyear)

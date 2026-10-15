---
lib: true
parent: Netcode crates
icon: package
tags: [networking]
check: skip
---
# bevy_ggrs

**Rollback networking** (GGPO-style) for peer-to-peer games: everyone runs the same deterministic simulation and exchanges only *inputs*. When a late input arrives, the game rolls back, replays, and catches up. Standard for fighting games and tight 1v1/2v2. #networking

#0.19 · pair it with [[bevy_matchbox]] for browser-friendly connections

```toml
bevy_ggrs = "0.22"
```

## The contract

Rollback only works if your game is **deterministic**: same state + same inputs = same result on every machine. That means:

- game logic in the `GgrsSchedule`, driven by the crate at a fixed rate;
- no unseeded randomness (see [[bevy_rand]] with a shared seed);
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

## Compared

| | Rollback (this) | Server-authoritative ([[bevy_replicon]], [[lightyear]]) |
|:--|:--|:--|
| Sends | inputs only | state |
| Server | none needed (peer-to-peer) | required |
| Cheat resistance | low | high |
| Good for | fighting, small player counts | most other games |

## Links

[crates.io](https://crates.io/crates/bevy_ggrs) · [docs.rs](https://docs.rs/bevy_ggrs) · [repo](https://github.com/gschup/bevy_ggrs)

## Next

[[bevy_matchbox]], [[Networking]].

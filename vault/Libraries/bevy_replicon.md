---
icon: package
tags: [lib, networking]
crates: bevy_replicon=0.45; serde={ version = "1", features = ["derive"] }
---
# bevy_replicon

**Server-authoritative replication**: mark entities `Replicated`, say which components to sync, and clients get a copy of the world. Plus typed messages and events in both directions. It does no I/O itself: you add a *messaging backend*. #lib #networking

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_replicon 0.45` supports **Bevy 0.20**.

```toml
bevy_replicon = "0.45"
serde = { version = "1", features = ["derive"] }
# plus one backend, e.g. bevy_replicon_renet, bevy_replicon_quinnet, aeronet_replicon, bevy_replicon_matchbox
```

## Why choose it

- Small, clear model: *the server owns the world, clients receive it*.
- **The same code runs** in single player, client, dedicated server and listen server.
- Works with any transport (UDP, QUIC, WebRTC, Steam): pick a backend.
- Fine control: per-client visibility, replication rules, serialization of any type.
- Prediction and rollback are available as add-ons (`bevy_rewind` and others, see the README); for built-in prediction look at [[lightyear]].

## Set it up

```rust
use bevy::{prelude::*, state::app::StatesPlugin};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RepliconPlugins /*, YourBackendPlugins */))
        // Which components to copy to clients:
        .replicate::<Health>()
        .replicate::<Player>()
        .replicate::<Transform>()
        .run();
}

#[derive(Component, Serialize, Deserialize)]
struct Health(u32);

#[derive(Component, Serialize, Deserialize)]
struct Player;
```

Components need `Serialize`/`Deserialize`. To replicate Bevy's own (`Transform`), enable Bevy's `serialize` feature.

## Replicate an entity

On the **server** add the `Replicated` marker. Clients get the entity automatically, with the listed components, and a `Remote` marker:

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;

#[derive(Component, serde::Serialize, serde::Deserialize)]
struct Health(u32);

fn spawn_player(mut commands: Commands) {
    commands.spawn((Replicated, Health(100), Transform::default()));
}
```

Despawn it on the server and it vanishes on clients. Rules are per component; add `#[require(...)]` on replicated components to rebuild non-networked ones (meshes, sprites) locally.

## Messages: client to server

Replication is one way (server → clients). Send what players *want* as messages:

```rust
use bevy::{prelude::*, state::app::StatesPlugin};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Message, Serialize, Deserialize)]
struct MoveInput(Vec2);

fn main() {
    App::new()
        .add_plugins((StatesPlugin, RepliconPlugins))
        .add_client_message::<MoveInput>(Channel::Ordered)
        .add_systems(Update, send.run_if(in_state(ClientState::Connected)))
        .add_systems(
            PreUpdate,
            receive.after(ServerSystems::Receive).run_if(in_state(ServerState::Running)),
        );
}

fn send(mut inputs: MessageWriter<MoveInput>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.pressed(KeyCode::KeyD) {
        inputs.write(MoveInput(Vec2::X));
    }
}

fn receive(mut inputs: MessageReader<FromClient<MoveInput>>) {
    for input in inputs.read() {
        info!("client {} wants to move {}", input.client_id, input.message.0);
    }
}
```

`add_server_message`, `add_client_event`, `add_server_event` and `add_shared_event` follow the same pattern with [[Events and observers|observers]] instead of readers.

## States and tick

`ClientState` and `ServerState` are normal [[States]]. The server sends updates on a **tick** (64 Hz by default, change `Time<Fixed>`; see [[Time and timers]]).

## Links

[crates.io](https://crates.io/crates/bevy_replicon) · [docs.rs](https://docs.rs/bevy_replicon) · [repo](https://github.com/simgine/bevy_replicon)

## Next

[[Networking]], [[Multiplayer]].

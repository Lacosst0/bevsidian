---
icon: package
tags: [lib, networking]
bevy: 0.19
crates: bevy_renet=5.0
---
# bevy_renet

A thin Bevy layer over **renet**: a reliable, channel-based **UDP transport** with encrypted, authenticated connections (netcode). It moves bytes between a server and clients; replication is up to you, or use [[bevy_replicon]] with its renet backend. #lib #networking

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `5.0.0` targets **Bevy 0.19**. Compiled against 0.19.

```toml
bevy_renet = "5.0"
```

## Channels

Messages travel on **channels** with delivery guarantees: `ReliableOrdered` (chat, events), `ReliableUnordered`, `Unreliable` (positions: the next one replaces a lost one).

## A server

```rust
use std::{net::UdpSocket, time::SystemTime};

use bevy::prelude::*;
use bevy_renet::{
    netcode::{NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::{ConnectionConfig, DefaultChannel, ServerEvent},
    RenetServer, RenetServerEvent, RenetServerPlugin,
};

fn main() {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let transport = NetcodeServerTransport::new(
        ServerConfig {
            current_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(),
            max_clients: 16,
            protocol_id: 7,                       // clients and server must agree
            public_addresses: vec![server_addr],
            authentication: ServerAuthentication::Unsecure,   // use Secure with keys in production
        },
        socket,
    )
    .unwrap();

    App::new()
        .add_plugins((MinimalPlugins, RenetServerPlugin, NetcodeServerPlugin))
        .insert_resource(RenetServer::new(ConnectionConfig::default()))
        .insert_resource(transport)
        .add_systems(Update, announce)
        .add_observer(handle_events)
        .run();
}

fn announce(mut server: ResMut<RenetServer>) {
    server.broadcast_message(DefaultChannel::ReliableOrdered, "hello, everyone");
}

fn handle_events(event: On<RenetServerEvent>) {
    match &event.0 {
        ServerEvent::ClientConnected { client_id } => info!("client {client_id} connected"),
        ServerEvent::ClientDisconnected { client_id, reason } => {
            info!("client {client_id} left: {reason}")
        }
    }
}
```

A client is the mirror image: `RenetClient`, `NetcodeClientTransport`, `ClientAuthentication`, and `client.receive_message(channel)`. Note that `bevy_renet::RenetServer`/`RenetClient` are thin wrappers (they dereference to the `renet` types), and server events (`ClientConnected`, `ClientDisconnected`) arrive as `RenetServerEvent` [[Events and observers|events]] for an observer.

## Choosing

- You want **control** and a minimal base: renet.
- You want replication on top: [[bevy_replicon]] + `bevy_replicon_renet`.
- You need prediction: [[lightyear]].
- Browsers cannot use UDP; use WebSocket/WebTransport backends ([[Web build]]).

## Links

[crates.io](https://crates.io/crates/bevy_renet) · [docs.rs](https://docs.rs/bevy_renet) · [repo](https://github.com/lucaspoffo/renet)

## Next

[[Networking]], [[Libraries]].

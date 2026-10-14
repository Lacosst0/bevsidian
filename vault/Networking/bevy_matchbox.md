---
icon: package
tags: [networking]
check: skip
---
# bevy_matchbox

**Peer-to-peer connections over WebRTC**, in the browser and natively. A small *signalling server* introduces the peers, then they talk directly. The usual partner of [[bevy_ggrs]]. #networking

> [!WARNING]
> **Bevy 0.20 status** (checked 2026-10-13): latest release `0.14` targets **Bevy 0.18**; updates for newer Bevy are pending.

```toml
bevy_matchbox = { version = "0.14", features = ["ggrs"] }
```

## How it works

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

## Why WebRTC

Browsers cannot send raw UDP. WebRTC data channels give nearly-UDP behaviour in the browser, with NAT traversal, so P2P works on the web ([[Web build]]).

## Links

[crates.io](https://crates.io/crates/bevy_matchbox) · [docs.rs](https://docs.rs/bevy_matchbox) · [repo](https://github.com/johanhelsing/matchbox)

## Next

[[bevy_ggrs]], [[Networking]].

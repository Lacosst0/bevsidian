---
lib: true
icon: package
tags: [networking]
crates: bevy_http_client=0.12; serde={ version = "1", features = ["derive"] }
---
# bevy_http_client

Make **HTTP requests** from systems: leaderboards, accounts, update checks, downloading configuration. Requests and responses are [[Messages|messages]]; no async code in your game. #networking

#0.20

```toml
bevy_http_client = "0.12"
serde = { version = "1", features = ["derive"] }
```

## A typed request

Ask for JSON and get a Rust struct back:

```rust
use bevy::prelude::*;
use bevy_http_client::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
struct IpInfo {
    ip: String,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, HttpClientPlugin))
        .add_systems(Startup, send_request)
        .add_systems(Update, (handle_response, handle_error));
    app.register_request_type::<IpInfo>();   // the response type
    app.run();
}

fn send_request(mut requests: MessageWriter<TypedRequest<IpInfo>>) {
    match HttpClient::new()
        .get("https://api.ipify.org?format=json")
        .try_with_type::<IpInfo>()
    {
        Ok(request) => {
            requests.write(request);
        }
        Err(error) => error!("could not build the request: {error}"),
    }
}

fn handle_response(mut responses: ResMut<Messages<TypedResponse<IpInfo>>>) {
    for response in responses.drain() {
        let info: IpInfo = response.into_inner();
        info!("my address is {}", info.ip);
    }
}

fn handle_error(mut errors: MessageReader<TypedResponseError<IpInfo>>) {
    for error in errors.read() {
        warn!("request failed: {}", error.err);
    }
}
```

## Good to know

- `HttpClient::new().get(url)`, `.post(url)`, `.json(&body)`, `.headers(..)` build requests; `try_with_type::<T>()` expects JSON for `T`.
- Without the type: write an `HttpRequest` message, read an `HttpResponse`.
- Works on the web too ([[Web build]]).
- Never put secrets in the client. For anything with accounts or scores, put a server in between.

## For real-time multiplayer

HTTP is request/response. For live games use [[bevy_replicon]], [[lightyear]] or [[bevy_renet]] ([[Networking]]).

## Links

[crates.io](https://crates.io/crates/bevy_http_client) · [docs.rs](https://docs.rs/bevy_http_client) · [repo](https://github.com/foxzool/bevy_http_client)

## Next

[[Networking]].

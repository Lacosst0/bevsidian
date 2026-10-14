---
icon: globe
tags: [tooling]
---
# Web build

Bevy compiles to WebAssembly and runs in the browser, on WebGL2 or WebGPU. This very site is a Bevy app built that way. #tooling

## The quick way

The Bevy CLI builds and serves for you:

```sh
cargo install --git https://github.com/TheBevyFlock/bevy_cli --locked bevy_cli
bevy run web
```

## The manual way

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli    # the version must match the one in Cargo.lock
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir out target/wasm32-unknown-unknown/release/my_game.wasm
```

Then serve `out/` with a page that loads the module:

```html
<canvas id="bevy"></canvas>
<script type="module">
  import init from "./out/my_game.js";
  init();
</script>
```

## Settings that matter

`.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
# Needed by crates that use random numbers (getrandom).
rustflags = ['--cfg', 'getrandom_backend="wasm_js"']
```

In your window:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#bevy".into()),     // the <canvas id="bevy">
                fit_canvas_to_parent: true,       // fill its container
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .run();
}
```

## Make it small

Download size is load time. Use a size-optimised profile and `wasm-opt`:

```toml
[profile.wasm-release]
inherits = "release"
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "debuginfo"
```

```sh
wasm-opt -Oz --strip-debug -o out/my_game_bg.wasm out/my_game_bg.wasm
```

Compress with gzip or brotli on the server; this repository's `tools/build-web.sh` even inlines a gzip-compressed wasm into one `index.html`.

## WebGL2 or WebGPU

| | WebGL2 (default) | WebGPU |
|:--|:--|:--|
| Browsers | all | recent Chrome, Edge, Safari, Firefox |
| Features | limited (no compute, fewer buffers) | complete |
| Enable | default | `features = ["webgpu"]` |

## Things that differ in a browser

- **No threads** by default: heavy `Update` work blocks the page. Keep frames short.
- **No raw files or UDP.** Load assets over HTTP ([[Assets]]), use WebSocket/WebTransport for networking ([[Networking]]), and `bevy_pkv` for saves ([[Saving and loading]]).
- **Audio** only starts after the first click or key press (browser rule).
- **Autoplay, pointer lock and fullscreen** need a user gesture.
- **Mobile browsers**: use touch ([[Touch]]) and watch memory.

## Publish

Any static host works: GitHub Pages, itch.io (zip the folder), Netlify, Cloudflare Pages. See this repo's `pages.yml` workflow for a branch-per-page setup.

## Next

[[Release builds]] for the desktop and [[Ship it]] for the whole checklist.

---
lib: true
parent: Debug crates
icon: package
tags: [tooling]
bevy: 0.19
crates: bevy_mod_debugdump=0.16
---
# bevy_mod_debugdump

**Draws your app as a graph**: which systems are in which schedule and how they are ordered ([[Schedules and sets]]). When "why did this run before that?" has no obvious answer, look at the picture. #tooling

#0.19

```toml
bevy_mod_debugdump = "0.16"
```

## Print a schedule as a graph

It prints Graphviz `dot` text; pipe it into `dot` to get an image:

```rust
use bevy::{log::LogPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    // No logging, so stdout holds only the graph.
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    // ... add your own plugins and systems ...
    bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
}
```

```sh
cargo run | dot -Tsvg > update.svg
```

Install Graphviz first (`apt install graphviz`, `brew install graphviz`).

## Other graphs

- The **render graph** (what the renderer does each frame).
- The **main schedule order** (First, PreUpdate, Update, ...) and plugin structure.

## Related

- Bevy's own `Stepping` (what this site's previews use) and `schedule_data` dev tool ([[Diagnostics and logging#Dev tools|Dev tools]]).
- [[Debugging]] for the rest of the debugging toolbox.

## Links

[crates.io](https://crates.io/crates/bevy_mod_debugdump) · [docs.rs](https://docs.rs/bevy_mod_debugdump) · [repo](https://github.com/jakobhellermann/bevy_mod_debugdump)

## Next

[[Tooling]].

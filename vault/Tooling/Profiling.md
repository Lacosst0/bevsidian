---
icon: chart-bar
tags: [tooling]
---
# Profiling

You can't make a game faster until you know **what** is slow. Measure first. #tooling

## First look: is it CPU or GPU?

- Shrink the window or lower the resolution. Faster → GPU (fill-rate) bound.
- Disable systems or reduce entities. Faster → CPU bound.
- Show frame time in [[Dev tools|the FPS overlay]]. A steady 16.6 ms is 60 fps.

## Bevy's built-in spans

Every system, render pass and engine function has a *tracing span*. Turn them on with the `trace` feature and pick a viewer:

```sh
# Timeline in the browser (ui.perfetto.dev)
cargo run --release --features bevy/trace_chrome

# Live, nanosecond-level: Tracy
cargo run --release --features bevy/trace_tracy
```

Add your own spans around a block:

```rust
use bevy::prelude::*;

fn pathfinding() {
    let _span = info_span!("pathfinding").entered();
    // ... expensive work ...
}
```

## Flame graphs

`cargo flamegraph` samples where the CPU spends time, per function, without code changes:

```sh
cargo install flamegraph
RUSTFLAGS='-C force-frame-pointers=y' cargo flamegraph
```

Enable debug symbols in your release profile (`debug = "line-tables-only"`) for readable names.

## GPU

Use your vendor's tool (Nsight Graphics, Radeon GPU Profiler, Xcode). RenderDoc is a debugger, not a profiler.

## Usual suspects in Bevy games

| Symptom | Likely fix |
|:--|:--|
| thousands of unique materials/meshes | share handles ([[Meshes and materials]]) |
| a system that loops over everything every frame | `Changed<T>` filters ([[Change detection]]), run conditions ([[Run conditions]]) |
| shadows from many lights | fewer shadow-casting lights ([[Lighting]]) |
| huge images | smaller textures; compress to KTX2 |
| UI rebuilt each frame | update only what changed |
| debug build | use `--release`, or `opt-level = 3` for dependencies ([[Project setup]]) |

## Rules

1. Profile a **release** build.
2. Change one thing at a time.
3. Keep a baseline number.

## Next

[[Web build]].

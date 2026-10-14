---
lib: true
icon: package
tags: [ui, tooling]
crates: bevy_egui=0.43
---
# bevy_egui

[egui](https://www.egui.rs) is an **immediate-mode** UI: you describe the interface in code *every frame* (`if ui.button("Save").clicked() { ... }`), with no entities to manage. `bevy_egui` plugs it into Bevy. The standard choice for **tools, debug panels and editors**. #ui #tooling

> [!NOTE]
> **Bevy 0.20 status** (checked 2026-10-13): `bevy_egui 0.43` supports **Bevy 0.20**.

```toml
bevy_egui = "0.43"
```

## Hello

```rust
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin::default()))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        // egui systems run in their own schedule, once per frame:
        .add_systems(EguiPrimaryContextPass, debug_panel)
        .run();
}

#[derive(Resource)]
struct Settings {
    volume: f32,
    god_mode: bool,
    name: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self { volume: 0.5, god_mode: false, name: "Player".into() }
    }
}

fn debug_panel(mut contexts: EguiContexts, mut settings: Local<Settings>) -> Result {
    egui::Window::new("Debug").show(contexts.ctx_mut()?, |ui| {
        ui.add(egui::Slider::new(&mut settings.volume, 0.0..=1.0).text("volume"));
        ui.checkbox(&mut settings.god_mode, "god mode");
        ui.text_edit_singleline(&mut settings.name);
        if ui.button("Reset").clicked() {
            *settings = Settings::default();
        }
    });
    Ok(())
}
```

The widgets edit the values you hand them (`&mut`), so state lives in your own data. No click handlers, no layout entities.

## When to use it

| Use egui for | Use Bevy UI ([[UI]]) for |
|:--|:--|
| tools, editors, inspectors, cheats, level editors | the game's own menus, HUD, dialogs |
| quick debug numbers | anything players should see styled to the game |
| complicated forms and tables | UI that must scale, animate and use game assets |

Lots of people ship both: egui for developer tools, Bevy UI for the player.

## Things to know

- Run egui UI systems in `EguiPrimaryContextPass`, not `Update`.
- `contexts.ctx_mut()?` gives the context for the primary window; there is one per window or camera.
- egui can capture the mouse and keyboard. Don't move the player when the pointer is over egui: check `ctx.wants_pointer_input()` / `wants_keyboard_input()`, or see the `absorb_input` example.
- Render an egui panel into a texture (`render_egui_to_image` example) to put it on a 3D screen.

## Related

[[bevy-inspector-egui]] shows and edits every component in the world, built on this crate.

## Links

[crates.io](https://crates.io/crates/bevy_egui) · [docs.rs](https://docs.rs/bevy_egui) · [repo](https://github.com/vladbat00/bevy_egui) · [egui demo](https://www.egui.rs/#demo)

## Next

[[Debugging]].

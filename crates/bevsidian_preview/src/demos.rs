//! The code behind each preview: the functions of the note's code block, as written there.
//!
//! A code block opts in with `preview=NAME` in its info string (` ```rust preview=regenerate `).
//! Keep these functions in sync with the notes; components also derive `Reflect` so the
//! preview can show their values.

use bevy::prelude::*;

/// What a preview shows.
pub enum Demo {
    /// Systems run in a world of their own; its entities are shown as cards.
    World(WorldDemo),
    /// A UI scene, spawned inside the preview.
    Scene(fn() -> Box<dyn Scene>),
}

/// Systems run in their own [`World`], optionally one at a time with Bevy's stepping.
pub struct WorldDemo {
    /// Registers the demo's types and adds its systems, in run order.
    pub build: fn(&mut World, &mut Schedule),
    /// The systems' function names, in run order (they are highlighted in the code).
    pub systems: &'static [&'static str],
    /// Systems that run once (`Startup`); later frames skip them.
    pub startup: &'static [&'static str],
    /// Whether the preview has step and play controls. Without them, one frame runs at once.
    pub stepping: bool,
    /// Frames per second while playing.
    pub fps: f32,
    /// Extra functions highlighted when a system runs (e.g. the observers it triggers).
    pub also: &'static [(&'static str, &'static str)],
}

/// The demo named in a code block's info string.
pub fn find(name: &str) -> Option<Demo> {
    Some(match name {
        "spawn-player" => Demo::World(WorldDemo {
            build: |world, schedule| {
                ecs::register(world);
                schedule.add_systems(ecs::spawn_player.run_if(run_once));
            },
            systems: &["spawn_player"],
            startup: &["spawn_player"],
            stepping: false,
            fps: 1.0,
            also: &[],
        }),
        "regenerate" => Demo::World(WorldDemo {
            build: |world, schedule| {
                ecs::register(world);
                schedule.add_systems((ecs::spawn_player.run_if(run_once), ecs::regenerate).chain());
            },
            systems: &["spawn_player", "regenerate"],
            startup: &["spawn_player"],
            stepping: true,
            fps: 10.0,
            also: &[],
        }),
        "damage" => Demo::World(WorldDemo {
            build: |world, schedule| {
                ecs::register(world);
                world.add_observer(ecs::on_damage);
                schedule.add_systems((ecs::spawn_player.run_if(run_once), ecs::attack).chain());
            },
            systems: &["spawn_player", "attack"],
            startup: &["spawn_player"],
            stepping: true,
            fps: 2.0,
            also: &[("attack", "on_damage")],
        }),
        "menu" => Demo::Scene(|| Box::new(bsn::menu())),
        "kit-buttons" => Demo::Scene(|| Box::new(kit::buttons())),
        "kit-tabs" => Demo::Scene(|| Box::new(kit::view_tabs())),
        "kit-badges" => Demo::Scene(|| Box::new(kit::badges())),
        "kit-form" => Demo::Scene(|| Box::new(kit::settings_card())),
        "kit-entity" => Demo::Scene(|| Box::new(kit::entity_card())),
        "wide-menu" => Demo::Scene(|| Box::new(bsn::wide_menu())),
        _ => return None,
    })
}

/// `ECS.md`
mod ecs {
    use bevy::prelude::*;

    pub fn register(world: &mut World) {
        let registry = world.resource::<AppTypeRegistry>();
        let mut registry = registry.write();
        registry.register::<Health>();
        registry.register::<Player>();
        registry.register::<Transform>();
    }

    #[derive(Component, Reflect)]
    #[reflect(Component)]
    pub struct Health(pub u32);

    #[derive(Component, Reflect)]
    #[reflect(Component)]
    pub struct Player;

    pub fn spawn_player(mut commands: Commands) {
        commands.spawn((Player, Health(50), Transform::default()));
    }

    pub fn regenerate(_time: Res<Time>, mut query: Query<&mut Health, With<Player>>) {
        for mut health in &mut query {
            health.0 = (health.0 + 1).min(100);
        }
    }

    #[derive(EntityEvent)]
    pub struct Damage {
        entity: Entity,
        amount: u32,
    }

    pub fn on_damage(damage: On<Damage>, mut health: Query<&mut Health>) {
        if let Ok(mut health) = health.get_mut(damage.entity) {
            health.0 = health.0.saturating_sub(damage.amount);
        }
    }

    pub fn attack(players: Query<Entity, With<Player>>, mut commands: Commands) {
        for player in &players {
            commands.trigger(Damage {
                entity: player,
                amount: 10,
            });
        }
    }
}

/// `Scenes and BSN.md`
mod bsn {
    use bevy::prelude::*;

    pub fn menu() -> impl Scene {
        bsn! {
            Node { flex_direction: FlexDirection::Column, row_gap: px(8) }
            Children [
                Text("Play")
                --
                Text("Quit")
            ]
        }
    }

    pub fn wide_menu() -> impl Scene {
        bsn! {
            @menu()
            Node { width: px(400) }
        }
    }
}

/// `UI kit.md`
mod kit {
    use bevsidian_ui::prelude::*;
    use bevy::{prelude::*, ui_widgets::Activate};
    use bevy_radioactive::view::{signal, view};

    pub fn buttons() -> impl Scene {
        bsn! {
            Node { flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(8), align_items: AlignItems::Center }
            Children [
                @{text_button(Variant::Primary, Size::Md, "Primary")}
                --
                @{text_button(Variant::Secondary, Size::Md, "Secondary")}
                --
                @{text_button(Variant::Outline, Size::Md, "Outline")}
                --
                @{text_button(Variant::Ghost, Size::Md, "Ghost")}
                --
                @{icon_button(Variant::Outline, Size::Icon, "search")}
                --
                @{icon_text_button(Variant::Secondary, Size::Sm, "copy", "Copy")}
                --
                @{text_button(Variant::Outline, Size::Sm, "Disabled")}
                Disabled(true)
            ]
        }
    }

    pub fn view_tabs() -> impl Scene {
        let current = signal(|| 0);
        let (note, graph) = (current.clone(), current.clone());
        view! {
            @{tab_list()}
            Children [
                @{tab("Note")}
                Selected($current == 0)
                on(move |_: On<Activate>| note.set(0))
                --
                @{tab("Graph")}
                Selected($current == 1)
                on(move |_: On<Activate>| graph.set(1))
            ]
        }
    }

    pub fn badges() -> impl Scene {
        let active = signal(|| true);
        let toggle = active.clone();
        view! {
            Node { flex_direction: FlexDirection::Column, row_gap: px(12) }
            Children [
                Node { column_gap: px(6), align_items: AlignItems::Center }
                Children [
                    @{badge(BadgeVariant::Outline, "#ecs")}
                    --
                    @{badge(BadgeVariant::Secondary, "3")}
                    --
                    @{badge_button("#beginner")}
                    Selected($active)
                    on(move |_: On<Activate>| toggle.update(|on| *on = !*on))
                ]
                --
                @{shortcuts(&[("Ctrl O", "switch"), ("Esc", "close")])}
            ]
        }
    }

    pub fn settings_card() -> impl Scene {
        let (size, speed, fps) = (signal(|| 16), signal(|| 1), signal(|| true));
        let speeds = vec![
            (1, "1×".to_owned()),
            (2, "2×".to_owned()),
            (3, "3×".to_owned()),
        ];
        bsn! {
            @{card()}
            Node { width: px(380), max_width: Val::Percent(100.0) }
            Children [
                @{field("Text size", None, stepper(size, 13..=24, 1, |v| format!("{v} px")))}
                --
                @{field("Scroll speed", Some("Wheel and trackpad"), segmented(speeds, speed))}
                --
                @{field("Show FPS", None, switch(fps))}
            ]
        }
    }

    pub fn entity_card() -> impl Scene {
        let components = ["Health(50)", "Player", "Transform (0, 0, 0)"]
            .map(ComponentValue::parse)
            .to_vec();
        bsn! { @EntityCard { @title: "Entity 7", @components: components } }
    }
}

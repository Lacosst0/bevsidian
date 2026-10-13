//! UI scenes shown above code blocks with `preview=NAME`: spawned as they would be in an app.

use bevy::prelude::*;

/// The scene named in a code block's `preview=NAME`.
pub fn find(name: &str) -> Option<fn() -> Box<dyn Scene>> {
    Some(match name {
        "menu" => || Box::new(bsn::menu()),
        "wide-menu" => || Box::new(bsn::wide_menu()),
        "kit-buttons" => || Box::new(kit::buttons()),
        "kit-tabs" => || Box::new(kit::view_tabs()),
        "kit-badges" => || Box::new(kit::badges()),
        "kit-form" => || Box::new(kit::settings_card()),
        "kit-entity" => || Box::new(kit::entity_card()),
        _ => return None,
    })
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

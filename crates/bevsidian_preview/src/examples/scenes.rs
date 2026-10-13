//! UI scenes shown above code blocks with `preview=NAME`: spawned as they would be in an app.

use bevy::prelude::*;

/// The scene named in a code block's `preview=NAME`.
pub fn find(name: &str) -> Option<fn() -> Box<dyn Scene>> {
    Some(match name {
        "menu" => || Box::new(bsn::menu()),
        "wide-menu" => || Box::new(bsn::wide_menu()),
        "bsn-buttons" => || Box::new(bsn::dialog()),
        "bsn-meters" => || Box::new(bsn::hud()),
        "kit-buttons" => || Box::new(kit::buttons()),
        "kit-tabs" => || Box::new(kit::view_tabs()),
        "kit-badges" => || Box::new(kit::badges()),
        "kit-form" => || Box::new(kit::settings_card()),
        "kit-entity" => || Box::new(kit::entity_card()),
        "flex" => || Box::new(layout::flex()),
        "grid" => || Box::new(layout::grid()),
        "grow" => || Box::new(layout::grow()),
        "colour-lab" => || Box::new(maths::colour_lab()),
        "easing" => || Box::new(maths::easing()),
        _ => return None,
    })
}

/// `Scenes and BSN.md`, `Composing scenes.md` and `Scene components.md`
mod bsn {
    use bevy::prelude::*;

    fn button() -> impl Scene {
        bsn! {
            Node { padding: UiRect::all(px(8)), border_radius: BorderRadius::all(px(6)) }
            BackgroundColor(Color::srgb(0.2, 0.2, 0.3))
        }
    }

    fn danger_button() -> impl Scene {
        bsn! {
            @button()
            BackgroundColor(Color::srgb(0.7, 0.1, 0.1))
        }
    }

    pub fn dialog() -> impl Scene {
        bsn! {
            Node { column_gap: px(8) }
            Children [
                @button() Children [ Text("Save") ]
                --
                @danger_button() Children [ Text("Delete") ]
            ]
        }
    }

    #[derive(SceneComponent, Default, Clone)]
    #[scene(MeterProps)]
    struct Meter {
        value: f32,
    }

    #[derive(Default)]
    struct MeterProps {
        percent: f32,
    }

    impl Meter {
        fn scene(props: MeterProps) -> impl Scene {
            let fill = props.percent;
            bsn! {
                Meter { value: {fill / 100.0} }
                Node {
                    width: px(160),
                    height: px(12),
                    border_radius: BorderRadius::all(px(6)),
                    overflow: Overflow::clip(),
                }
                BackgroundColor(Color::srgb(0.15, 0.15, 0.2))
                Children [
                    Node { width: percent(fill), height: percent(100) }
                    BackgroundColor(Color::srgb(0.4, 0.7, 1.0))
                ]
            }
        }
    }

    pub fn hud() -> impl Scene {
        bsn! {
            Node { flex_direction: FlexDirection::Column, row_gap: px(8) }
            Children [
                @Meter { @percent: 25.0 }
                --
                @Meter { @percent: 75.0 }
                --
                @Meter { @percent: 100.0 }
            ]
        }
    }

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

/// `UI layout.md`
mod layout {
    use bevsidian_ui::prelude::*;
    use bevy::prelude::*;
    use bevy_radioactive::view::{signal, view};

    fn named<T>(options: Vec<(T, &str)>) -> Vec<(T, String)> {
        options
            .into_iter()
            .map(|(v, n)| (v, n.to_owned()))
            .collect()
    }

    /// Three boxes in a container whose flex settings you change.
    pub fn flex() -> impl Scene {
        let direction = signal(|| FlexDirection::Row);
        let justify = signal(|| JustifyContent::Start);
        let align = signal(|| AlignItems::Start);
        let directions = named(vec![
            (FlexDirection::Row, "row"),
            (FlexDirection::Column, "column"),
        ]);
        let justifies = named(vec![
            (JustifyContent::Start, "start"),
            (JustifyContent::Center, "center"),
            (JustifyContent::End, "end"),
            (JustifyContent::SpaceBetween, "between"),
        ]);
        let aligns = named(vec![
            (AlignItems::Start, "start"),
            (AlignItems::Center, "center"),
            (AlignItems::End, "end"),
            (AlignItems::Stretch, "stretch"),
        ]);
        let square = |width: f32, height: f32, color: Color| {
            bsn! {
                Node { width: px(width), height: px(height) }
                BackgroundColor(color)
            }
        };
        view! {
            Node { flex_direction: FlexDirection::Column, row_gap: px(8), width: Val::Percent(100.0) }
            Children [
                @{field("flex-direction", None, segmented(directions, direction.clone()))}
                --
                @{field("justify-content", None, segmented(justifies, justify.clone()))}
                --
                @{field("align-items", None, segmented(aligns, align.clone()))}
                --
                Node {
                    height: px(150),
                    padding: UiRect::all(px(8)),
                    column_gap: px(8),
                    row_gap: px(8),
                    flex_direction: $direction,
                    justify_content: $justify,
                    align_items: $align,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                }
                BorderColor::all(Color::srgb(0.3, 0.3, 0.33))
                Children [
                    @{square(40.0, 30.0, Color::srgb(0.95, 0.45, 0.4))}
                    --
                    @{square(50.0, 50.0, Color::srgb(0.98, 0.75, 0.3))}
                    --
                    @{square(30.0, 40.0, Color::srgb(0.35, 0.75, 0.9))}
                ]
            ]
        }
    }

    fn cell(index: usize, height: f32) -> impl Scene {
        const COLOURS: [Color; 6] = [
            Color::srgb(0.95, 0.45, 0.4),
            Color::srgb(0.98, 0.75, 0.3),
            Color::srgb(0.55, 0.85, 0.45),
            Color::srgb(0.35, 0.75, 0.9),
            Color::srgb(0.6, 0.5, 0.95),
            Color::srgb(0.95, 0.55, 0.8),
        ];
        let color = COLOURS[index % COLOURS.len()];
        bsn! {
            Node { height: px(height), min_width: px(24) }
            BackgroundColor(color)
        }
    }

    /// Eight cells in a grid whose columns and gap you change.
    pub fn grid() -> impl Scene {
        let columns = signal(|| 3_u16);
        let gap = signal(|| 4_u32);
        let column_options = named(vec![(1, "1"), (2, "2"), (3, "3"), (4, "4")]);
        let gap_options = named(vec![(0, "0"), (4, "4"), (12, "12")]);
        let cells: Vec<_> = (0..8).map(|i| cell(i, 36.0)).collect();
        view! {
            Node { flex_direction: FlexDirection::Column, row_gap: px(8), width: Val::Percent(100.0) }
            Children [
                @{field("columns", None, segmented(column_options, columns.clone()))}
                --
                @{field("gap", None, segmented(gap_options, gap.clone()))}
                --
                Node {
                    display: Display::Grid,
                    grid_template_columns: $(vec![RepeatedGridTrack::flex($columns, 1.0)]),
                    column_gap: $(px($gap as f32)),
                    row_gap: $(px($gap as f32)),
                    padding: UiRect::all(px(8)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                }
                BorderColor::all(Color::srgb(0.3, 0.3, 0.33))
                Children [ {cells} ]
            ]
        }
    }

    /// A row of three boxes: how much the middle one grows, and whether the row wraps.
    pub fn grow() -> impl Scene {
        let grow = signal(|| 0_u32);
        let wrap = signal(|| false);
        let grow_options = named(vec![(0, "0"), (1, "1"), (2, "2")]);
        let wrap_options = named(vec![(false, "nowrap"), (true, "wrap")]);
        let fixed = |index: usize| {
            bsn! {
                @{cell(index, 36.0)}
                Node { width: px(110), flex_shrink: 0.0 }
            }
        };
        view! {
            Node { flex_direction: FlexDirection::Column, row_gap: px(8), width: Val::Percent(100.0) }
            Children [
                @{field("flex-grow (middle)", None, segmented(grow_options, grow.clone()))}
                --
                @{field("flex-wrap", None, segmented(wrap_options, wrap.clone()))}
                --
                Node {
                    column_gap: px(8),
                    row_gap: px(8),
                    padding: UiRect::all(px(8)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                    flex_wrap: $(if $wrap { FlexWrap::Wrap } else { FlexWrap::NoWrap }),
                }
                BorderColor::all(Color::srgb(0.3, 0.3, 0.33))
                Children [
                    @{fixed(0)}
                    --
                    @{cell(1, 36.0)}
                    Node { width: px(110), flex_grow: $($grow as f32) }
                    --
                    @{fixed(2)}
                    --
                    @{fixed(3)}
                ]
            ]
        }
    }
}

/// `Colors and math.md`
pub mod maths {
    use bevsidian_ui::prelude::*;
    use bevy::prelude::*;
    use bevy_radioactive::view::{signal, view};

    fn named<T>(options: Vec<(T, &str)>) -> Vec<(T, String)> {
        options
            .into_iter()
            .map(|(v, n)| (v, n.to_owned()))
            .collect()
    }

    fn base(hue: u32, lightness: u32) -> Color {
        Color::hsl(hue as f32, 0.8, lightness as f32 / 100.0)
    }

    fn mixed(hue: u32, lightness: u32, amount: u32) -> Color {
        base(hue, lightness).mix(&Color::WHITE, amount as f32 / 100.0)
    }

    fn hex(color: Color) -> String {
        Srgba::from(color).to_hex()
    }

    /// Pick a hue and a lightness, then how far to mix towards white.
    pub fn colour_lab() -> impl Scene {
        let hue = signal(|| 210_u32);
        let lightness = signal(|| 50_u32);
        let amount = signal(|| 50_u32);
        let hues = named(vec![
            (0, "0"),
            (60, "60"),
            (120, "120"),
            (210, "210"),
            (300, "300"),
        ]);
        let lightnesses = named(vec![(30, "0.3"), (50, "0.5"), (70, "0.7")]);
        let amounts = named(vec![(0, "0"), (25, "0.25"), (50, "0.5"), (100, "1")]);
        let chip = |radius: f32| {
            bsn! { Node { width: px(72), height: px(48), border_radius: BorderRadius::all(px(radius)) } }
        };
        view! {
            Node { flex_direction: FlexDirection::Column, row_gap: px(8), width: Val::Percent(100.0) }
            Children [
                @{field("hue (degrees)", None, segmented(hues, hue.clone()))}
                --
                @{field("lightness", None, segmented(lightnesses, lightness.clone()))}
                --
                @{field("mix amount", None, segmented(amounts, amount.clone()))}
                --
                Node { column_gap: px(16), align_items: AlignItems::FlexStart }
                Children [
                    Node { flex_direction: FlexDirection::Column, row_gap: px(4), align_items: AlignItems::Center }
                    Children [
                        @{chip(6.0)}
                        BackgroundColor($(base($hue, $lightness)))
                        --
                        Text($(hex(base($hue, $lightness))))
                    ]
                    --
                    Node { flex_direction: FlexDirection::Column, row_gap: px(4), align_items: AlignItems::Center }
                    Children [
                        @{chip(6.0)}
                        BackgroundColor(Color::WHITE)
                        --
                        Text("#FFFFFF")
                    ]
                    --
                    Node { flex_direction: FlexDirection::Column, row_gap: px(4), align_items: AlignItems::Center }
                    Children [
                        @{chip(6.0)}
                        BackgroundColor($(mixed($hue, $lightness, $amount)))
                        --
                        Text($(hex(mixed($hue, $lightness, $amount))))
                    ]
                ]
            ]
        }
    }

    /// Which easing curve the ball follows.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum Curve {
        #[default]
        Linear,
        QuadraticOut,
        CubicInOut,
        BackOut,
        ElasticOut,
        BounceOut,
    }

    impl Curve {
        fn ease(self) -> bevy::curve::easing::EaseFunction {
            use bevy::curve::easing::EaseFunction;
            match self {
                Curve::Linear => EaseFunction::Linear,
                Curve::QuadraticOut => EaseFunction::QuadraticOut,
                Curve::CubicInOut => EaseFunction::CubicInOut,
                Curve::BackOut => EaseFunction::BackOut,
                Curve::ElasticOut => EaseFunction::ElasticOut,
                Curve::BounceOut => EaseFunction::BounceOut,
            }
        }
    }

    /// The curve a ball follows.
    #[derive(Component, Clone, Default)]
    pub struct Ease(pub Curve);

    /// Slides every ball along its track with its curve, then pauses, forever.
    pub fn animate_easing(time: Res<Time>, mut balls: Query<(&Ease, &mut Node)>) {
        let phase = (time.elapsed_secs() % 2.0) / 1.4;
        for (curve, mut node) in &mut balls {
            let progress = {
                use bevy::curve::Curve as _;
                curve.0.ease().sample_clamped(phase.min(1.0))
            };
            node.left = percent(progress * 92.0);
        }
    }

    /// A ball that travels along a track using the curve you pick.
    pub fn easing() -> impl Scene {
        let curve = signal(Curve::default);
        let curves = named(vec![
            (Curve::Linear, "Linear"),
            (Curve::QuadraticOut, "QuadraticOut"),
            (Curve::CubicInOut, "CubicInOut"),
            (Curve::BackOut, "BackOut"),
            (Curve::ElasticOut, "ElasticOut"),
            (Curve::BounceOut, "BounceOut"),
        ]);
        view! {
            Node { flex_direction: FlexDirection::Column, row_gap: px(10), width: Val::Percent(100.0) }
            Children [
                @{field("EaseFunction", None, segmented(curves, curve.clone()))}
                --
                Node {
                    height: px(28),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(14)),
                }
                BorderColor::all(Color::srgb(0.3, 0.3, 0.33))
                Children [
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(24),
                        height: px(24),
                        top: px(1),
                        border_radius: BorderRadius::all(px(12)),
                    }
                    BackgroundColor(Color::srgb(0.95, 0.55, 0.3))
                    Ease($curve)
                ]
            ]
        }
    }
}

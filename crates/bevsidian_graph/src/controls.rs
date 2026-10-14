//! The map's controls: the zoom buttons (with folding and the map editor's toggle), the
//! legend and the edit bar.

use bevy::{prelude::*, text::FontWeight, ui_widgets::Activate};

use bevsidian_core::{
    map::{MapObject, Scale, Shape},
    nav::Nav,
};
use bevsidian_ui::prelude::*;

use crate::{
    GraphCamera, GraphNode, GraphState, GraphViewport, MAX_SCALE, MIN_SCALE, edit::EditRequest,
    fit, ortho_scale, world::MapThing,
};

/// Zoom buttons over the map (the only way to zoom on touch screens): one bordered group of
/// zoom in, zoom out and fit, unfold and fold every note, and edit the map.
pub fn zoom_controls() -> impl Scene {
    fn fold_button(icon: &'static str, tip: &'static str, all: bool) -> impl Scene {
        bsn! {
            @{icon_button(Variant::Ghost, Size::Icon, icon)}
            Tooltip::new(tip)
            Node { border: UiRect::top(px(1.0)), border_radius: BorderRadius::ZERO }
            BorderColor::all(color::BORDER)
            on(move |_: On<Activate>, mut state: ResMut<GraphState>| {
                state.fold_all = Some(all);
            })
        }
    }
    fn button(icon: &'static str, tip: &'static str, factor: f32, first: bool) -> impl Scene {
        bsn! {
            @{icon_button(Variant::Ghost, Size::Icon, icon)}
            Tooltip::new(tip)
            Node {
                border: UiRect::top(px(if first { 0.0 } else { 1.0 })),
                border_radius: BorderRadius::ZERO,
            }
            BorderColor::all(color::BORDER)
            on(move |_: On<Activate>,
                     camera: Query<(&Transform, &Projection), With<GraphCamera>>,
                     nodes: Query<(&Transform, &GraphNode), Without<GraphCamera>>,
                     viewport: Query<&ComputedNode, With<GraphViewport>>,
                     mut state: ResMut<GraphState>| {
                let Ok((transform, projection)) = camera.single() else {
                    return;
                };
                let (position, scale) = state
                    .camera_goal
                    .unwrap_or((transform.translation.truncate(), ortho_scale(projection)));
                state.pan_velocity = Vec2::ZERO;
                state.framing = false;
                if factor == 0.0 {
                    if let Ok(viewport) = viewport.single() {
                        state.camera_goal = Some(fit(nodes.iter(), viewport));
                    }
                } else {
                    state.camera_goal = Some((position, (scale * factor).clamp(MIN_SCALE, MAX_SCALE)));
                }
            })
        }
    }
    let shadow = shadow::popover();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            right: px(16),
            bottom: px(16),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::LG)),
            overflow: Overflow::clip(),
        }
        BackgroundColor(color::CARD)
        BorderColor::all(color::INPUT)
        shadow
        Children [
            @{button("plus", "Zoom in", 0.8, true)}
            --
            @{button("minus", "Zoom out", 1.25, false)}
            --
            @{button("maximize", "Fit to screen", 0.0, false)}
            --
            @{fold_button("unfold-vertical", "Expand all", true)}
            --
            @{fold_button("fold-vertical", "Collapse all", false)}
            --
            @{icon_button(Variant::Ghost, Size::Icon, "pencil-ruler")}
            EditToggle
            Tooltip::new("Edit the map")
            Node { border: UiRect::top(px(1.0)), border_radius: BorderRadius::ZERO }
            BorderColor::all(color::BORDER)
            on(|_: On<Activate>, nav: Res<Nav>| nav.map_edit.update(|on| *on = !*on))
        ]
    }
}

/// The zoom group's edit button (selected while editing).
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct EditToggle;

/// The legend (hidden while editing: the edit bar takes its place).
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct Legend;

/// What the map's shapes and roads mean, in its corner.
pub fn legend() -> impl Scene {
    fn row(sample: Box<dyn Scene>, text: &str) -> impl Scene + use<> {
        let sample = vec![sample];
        bsn! {
            Node { align_items: AlignItems::Center, column_gap: px(10) }
            Children [
                Node { width: px(24), height: px(16), align_items: AlignItems::Center, justify_content: JustifyContent::Center }
                Children [ {sample} ]
                --
                @{caption(text)}
            ]
        }
    }
    fn shape(name: &str) -> Box<dyn Scene> {
        Box::new(icon(name, 14.0, color::MUTED_FOREGROUND))
    }
    fn road(pieces: usize, width: f32, round: bool) -> Box<dyn Scene> {
        let piece = |_| -> Box<dyn Scene> {
            let node = Node {
                width: px(width),
                height: px(if round { 4.0 } else { 3.0 }),
                border_radius: BorderRadius::all(px(if round { 2.0 } else { 0.0 })),
                ..default()
            };
            Box::new(bsn! { node BackgroundColor(color::MUTED_FOREGROUND) })
        };
        let pieces: Vec<Box<dyn Scene>> = (0..pieces).map(piece).collect();
        Box::new(bsn! {
            Node { width: px(24), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween }
            Children [ {pieces} ]
        })
    }
    // A note holding folded notes: the "+N" badge.
    let badge: Box<dyn Scene> = Box::new(bsn! {
        Node {
            width: px(18),
            height: px(18),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::MAX,
        }
        BackgroundColor(color::FOREGROUND)
        Children [
            Text::new("+")
            @{insert(font(size::XS, FontWeight::SEMIBOLD))}
            TextColor(color::PRIMARY_FOREGROUND)
        ]
    });
    let tick: Box<dyn Scene> = Box::new(bsn! {
        Node {
            width: px(14),
            height: px(14),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::MAX,
        }
        BackgroundColor(color::SUCCESS)
        Children [ @{icon("check", 10.0, color::BACKGROUND)} ]
    });
    let shadow = shadow::popover();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            bottom: px(16),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            padding: UiRect::axes(px(12), px(10)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius::LG)),
        }
        Legend
        BackgroundColor(color::CARD)
        BorderColor::all(color::INPUT)
        Pickable::IGNORE
        shadow
        Children [
            @{row(shape("hexagon"), "Region: an area's town")}
            --
            @{row(shape("circle"), "Lesson")}
            --
            @{row(shape("diamond"), "Milestone")}
            --
            @{row(shape("star"), "Goal of a route")}
            --
            @{row(shape("square"), "Crate")}
            --
            @{separator()} Node { margin: UiRect::vertical(px(3)) }
            --
            @{row(road(1, 24.0, false), "Main road")}
            --
            @{row(road(3, 6.0, false), "Side quest")}
            --
            @{row(road(4, 4.0, true), "Secret")}
            --
            @{row(badge, "Holds folded notes")}
            --
            @{row(tick, "Visited")}
        ]
    }
}

/// The edit bar (shown while editing) and its row for the chosen note or object.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct EditBar;

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct Inspector;

/// The name of what is chosen.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct InspectorName;

/// Parts of the inspector shown for notes only, or objects only.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ForKind {
    #[default]
    Note,
    Object,
}

/// A shape button (`None`: the shape the note's kind gives it).
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct ShapeChoice(Option<Shape>);

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct ScaleChoice(Scale);

/// Landmarks offered by the edit bar.
const LANDMARKS: [(&str, &str); 6] = [
    ("tree-pine", "Add a forest"),
    ("mountain", "Add a mountain"),
    ("castle", "Add a castle"),
    ("tent", "Add a camp"),
    ("flag", "Add a flag"),
    ("waves", "Add waves"),
];

fn request(
    request: EditRequest,
) -> impl Fn(On<Activate>, ResMut<GraphState>) + Clone + Send + Sync + 'static {
    move |_: On<Activate>, mut state: ResMut<GraphState>| state.edit_request = Some(request.clone())
}

/// Editing the map: what to add, and the chosen note's shape and size (or deleting the chosen
/// object), centred at the top of the map.
pub fn edit_bar() -> impl Scene {
    let landmarks: Vec<Box<dyn Scene>> = LANDMARKS
        .iter()
        .map(|&(name, tip)| -> Box<dyn Scene> {
            let object = MapObject::Landmark {
                icon: name.to_owned(),
                at: Vec2::ZERO,
                size: bevsidian_core::map::LANDMARK_SIZE,
            };
            Box::new(bsn! {
                @{icon_button(Variant::Ghost, Size::IconSm, name)}
                Tooltip::new(tip)
                on(request(EditRequest::Add(object)))
            })
        })
        .collect();
    let shapes: Vec<Box<dyn Scene>> = std::iter::once(None)
        .chain(Shape::ALL.into_iter().map(Some))
        .map(|shape| -> Box<dyn Scene> {
            let (icon, tip) = match shape {
                None => ("sparkles", "The shape of its kind"),
                Some(Shape::Circle) => ("circle", "Circle: a lesson"),
                Some(Shape::Diamond) => ("diamond", "Diamond: a milestone"),
                Some(Shape::Hexagon) => ("hexagon", "Hexagon: a region's town"),
                Some(Shape::Square) => ("square", "Square: a crate"),
                Some(Shape::Pentagon) => ("pentagon", "Pentagon"),
                Some(Shape::Octagon) => ("octagon", "Octagon: a capital"),
                Some(Shape::Star) => ("star", "Star: a route's goal"),
            };
            Box::new(bsn! {
                @{icon_button(Variant::Ghost, Size::IconSm, icon)}
                ShapeChoice(shape)
                Selected(false)
                Tooltip::new(tip)
                on(request(EditRequest::Shape(shape)))
            })
        })
        .collect();
    let scales: Vec<Box<dyn Scene>> = [
        (Scale::Small, "S"),
        (Scale::Normal, "M"),
        (Scale::Large, "L"),
    ]
    .into_iter()
    .map(|(scale, text)| -> Box<dyn Scene> {
        Box::new(bsn! {
            @{text_button(Variant::Ghost, Size::Sm, text)}
            ScaleChoice(scale)
            Selected(false)
            Node { width: px(32), padding: UiRect::ZERO }
            on(request(EditRequest::Scale(scale)))
        })
    })
    .collect();
    let region = MapObject::Label {
        text: "New place".to_owned(),
        caption: String::new(),
        at: Vec2::ZERO,
        size: bevsidian_core::map::LABEL_SIZE,
    };
    let land = MapObject::Land {
        at: Vec2::ZERO,
        radius: bevsidian_core::map::LAND_RADIUS,
    };
    let shadow = shadow::popover();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(16),
            justify_content: JustifyContent::Center,
            display: Display::None,
        }
        EditBar
        Pickable::IGNORE
        Children [
            Node {
                flex_direction: FlexDirection::Column,
                max_width: Val::Percent(94.0),
                padding: UiRect::all(px(4)),
                row_gap: px(4),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(radius::LG)),
            }
            BackgroundColor(color::CARD)
            BorderColor::all(color::INPUT)
            shadow
            Children [
                Node { align_items: AlignItems::Center, column_gap: px(2), flex_wrap: FlexWrap::Wrap }
                Children [
                    Node { align_items: AlignItems::Center, column_gap: px(8), padding: UiRect::horizontal(px(8)) }
                    Children [
                        @{icon("pencil-ruler", 16.0, color::MUTED_FOREGROUND)}
                        --
                        @{label("Editing the map", size::SM, FontWeight::MEDIUM, color::FOREGROUND)}
                    ]
                    --
                    @{vertical_separator()} Node { margin: UiRect::horizontal(px(4)), height: px(20) }
                    --
                    Node { align_items: AlignItems::Center }
                    Children [ {landmarks} ]
                    --
                    @{icon_text_button(Variant::Ghost, Size::Sm, "type", "Label")}
                    Tooltip::new("Add a region's name")
                    on(request(EditRequest::Add(region)))
                    --
                    @{icon_text_button(Variant::Ghost, Size::Sm, "map", "Land")}
                    Tooltip::new("Add a piece of land")
                    on(request(EditRequest::Add(land)))
                    --
                    @{vertical_separator()} Node { margin: UiRect::horizontal(px(4)), height: px(20) }
                    --
                    @{text_button(Variant::Primary, Size::Sm, "Done")}
                    on(|_: On<Activate>, nav: Res<Nav>| nav.map_edit.set(false))
                ]
                --
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(2),
                    flex_wrap: FlexWrap::Wrap,
                    border: UiRect::top(px(1)),
                    padding: UiRect::top(px(4)),
                    display: Display::None,
                }
                BorderColor::all(color::BORDER)
                Inspector
                Children [
                    Text::new("")
                    InspectorName
                    @{insert(font(size::SM, FontWeight::MEDIUM))}
                    TextColor(color::FOREGROUND)
                    Node { margin: UiRect::horizontal(px(8)), max_width: px(220), overflow: Overflow::clip() }
                    --
                    Node { align_items: AlignItems::Center }
                    ForKind::Note
                    Children [ {shapes} ]
                    --
                    @{vertical_separator()} Node { margin: UiRect::horizontal(px(4)), height: px(20) }
                    --
                    Node { align_items: AlignItems::Center }
                    Children [ {scales} ]
                    --
                    @{icon_text_button(Variant::Ghost, Size::Sm, "x", "Delete")}
                    ForKind::Object
                    on(request(EditRequest::Delete))
                ]
            ]
        ]
    }
}

/// Shows the legend or the edit bar, and the edit bar's state: what is chosen, its shape
/// and its size.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn show_legend(
    nav: Res<Nav>,
    state: Res<GraphState>,
    vault: Res<bevsidian_core::Vault>,
    nodes: Query<&GraphNode>,
    things: Query<&MapThing>,
    mut panels: Query<
        (
            &mut Node,
            Has<Legend>,
            Has<EditBar>,
            Has<Inspector>,
            Option<&ForKind>,
        ),
        Or<(With<Legend>, With<EditBar>, With<Inspector>, With<ForKind>)>,
    >,
    mut choices: Query<(
        &mut Selected,
        Option<&ShapeChoice>,
        Option<&ScaleChoice>,
        Has<EditToggle>,
    )>,
    mut name: Query<&mut Text, With<InspectorName>>,
) {
    let editing = nav.map_edit.get_untracked();
    let data = vault.get_untracked();
    let note = state.selected.and_then(|e| nodes.get(e).ok());
    let thing = state.selected.and_then(|e| things.get(e).ok());
    let place = note.and_then(|n| data.map.as_ref()?.place(&data.notes[n.note].name));
    let kind = if note.is_some() {
        Some(ForKind::Note)
    } else {
        thing.map(|_| ForKind::Object)
    };
    for (mut node, legend, bar, inspector, for_kind) in &mut panels {
        let shows = if legend {
            !editing
        } else if bar {
            editing
        } else if inspector {
            kind.is_some()
        } else {
            for_kind.copied() == kind
        };
        let display = if shows { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
    let shape = place.and_then(|p| p.shape);
    let scale = place.map(|p| p.scale).or_else(|| {
        thing.map(|t| {
            let (size, normal) = match &t.object {
                MapObject::Label { size, .. } => (*size, bevsidian_core::map::LABEL_SIZE),
                MapObject::Landmark { size, .. } => (*size, bevsidian_core::map::LANDMARK_SIZE),
                MapObject::Land { radius, .. } => (*radius, bevsidian_core::map::LAND_RADIUS),
            };
            if size < normal * 0.95 {
                Scale::Small
            } else if size > normal * 1.05 {
                Scale::Large
            } else {
                Scale::Normal
            }
        })
    });
    for (mut selected, shape_choice, scale_choice, toggle) in &mut choices {
        let on = if toggle {
            editing
        } else if let Some(choice) = shape_choice {
            note.is_some() && choice.0 == shape
        } else if let Some(choice) = scale_choice {
            scale == Some(choice.0)
        } else {
            continue;
        };
        selected.set_if_neq(Selected(on));
    }
    let wanted = match (note, thing) {
        (Some(n), _) => data.notes[n.note].name.clone(),
        (_, Some(t)) => match &t.object {
            MapObject::Label { text, .. } => format!("Label: {text}"),
            MapObject::Landmark { icon, .. } => format!("Landmark: {icon}"),
            MapObject::Land { .. } => "Land".to_owned(),
        },
        _ => String::new(),
    };
    for mut text in &mut name {
        if text.0 != wanted {
            text.0.clone_from(&wanted);
        }
    }
}

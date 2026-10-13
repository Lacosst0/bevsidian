//! ECS identity: how entities, components and systems look wherever the app shows them.
//!
//! The shape of the icon mount says what kind of thing it is, the Lucide icon which thing:
//! a filled tile for what exists in the world (entities, scenes), an outlined tile for what
//! runs (systems, observers), a bare icon for data on an entity (components, resources,
//! events). Everything stays neutral; only a running system is highlighted.

use bevy::{prelude::*, text::FontWeight};

use crate::{
    components::{
        icon::{TileStyle, glyph, icon, icon_tile},
        typography::label,
    },
    tokens::{color, font::size, radius},
};

/// The kinds of ECS things the app draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EcsKind {
    Entity,
    Component,
    /// A component without fields, such as `Player`.
    Marker,
    Resource,
    System,
    Observer,
    Event,
    Query,
    Scene,
}

impl EcsKind {
    /// The Lucide icon of the kind.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Entity => "box",
            Self::Component => "puzzle",
            Self::Marker => "flag",
            Self::Resource => "database",
            Self::System => "square-function",
            Self::Observer => "eye",
            Self::Event => "zap",
            Self::Query => "filter",
            Self::Scene => "layers",
        }
    }

    /// The tile the icon sits on, or `None` for a bare icon.
    pub fn tile(self) -> Option<TileStyle> {
        match self {
            Self::Entity | Self::Scene => Some(TileStyle::Filled),
            Self::System | Self::Observer => Some(TileStyle::Outlined),
            _ => None,
        }
    }
}

/// The icon of a component type: well-known Bevy components have their own, other
/// components a puzzle piece, and components without data a flag.
pub fn component_icon(name: &str, has_data: bool) -> &'static str {
    match name {
        "Health" => "heart",
        "Transform" | "GlobalTransform" => "move",
        "Camera" | "Camera2d" | "Camera3d" => "video",
        "Mesh2d" | "Mesh3d" => "shapes",
        "MeshMaterial2d" | "MeshMaterial3d" => "palette",
        "Name" => "text",
        "Node" => "layout-template",
        "Text" | "Text2d" => "type",
        _ if has_data => EcsKind::Component.icon(),
        _ => EcsKind::Marker.icon(),
    }
}

/// A component as shown on an entity card: its type name and, when it has fields, a short
/// rendering of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentValue {
    pub name: String,
    pub value: Option<String>,
}

impl ComponentValue {
    /// Splits a `Debug` rendering into name and value: `Health(50)` → `Health`, `50`;
    /// `Speed { x: 1.0 }` → `Speed`, `x: 1.0`; `Player` → `Player`, no value.
    pub fn parse(debug: &str) -> Self {
        let debug = debug.trim();
        let split = debug.find(['(', '{', ' ']).unwrap_or(debug.len());
        let (name, rest) = debug.split_at(split);
        let rest = rest.trim();
        let inner = rest
            .strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
            .or_else(|| rest.strip_prefix('{').and_then(|r| r.strip_suffix('}')))
            .unwrap_or(rest)
            .trim();
        Self {
            name: name.to_owned(),
            value: (!inner.is_empty()).then(|| inner.to_owned()),
        }
    }
}

/// An entity and its components: a filled tile and the title, then one row per component
/// with its icon and name on the left and its data on the right.
#[derive(SceneComponent, Default, Clone)]
#[scene(EntityCardProps)]
pub struct EntityCard;

/// What an [`EntityCard`] shows.
#[derive(Default)]
pub struct EntityCardProps {
    /// `Entity 7`.
    pub title: String,
    pub components: Vec<ComponentValue>,
}

impl EntityCard {
    fn scene(props: EntityCardProps) -> impl Scene {
        let rows: Vec<Box<dyn Scene>> = props
            .components
            .iter()
            .enumerate()
            .map(|(i, component)| {
                let last = i + 1 == props.components.len();
                Box::new(component_row(component, last)) as Box<dyn Scene>
            })
            .collect();
        let tile = icon_tile(
            glyph(EcsKind::Entity.icon()).chars().next().unwrap_or(' '),
            28.0,
            TileStyle::Filled,
        );
        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                width: px(260),
                max_width: Val::Percent(100.0),
                padding: UiRect::new(px(14), px(14), px(12), px(6)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(radius::XL)),
            }
            BackgroundColor(color::CARD)
            BorderColor::all(color::BORDER)
            Children [
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    padding: UiRect::bottom(px(10)),
                    margin: UiRect::bottom(px(4)),
                    border: UiRect::bottom(px(1)),
                }
                BorderColor::all(color::BORDER)
                Children [
                    @{tile}
                    --
                    @{label(&props.title, size::SM, FontWeight::SEMIBOLD, color::FOREGROUND)}
                ]
                --
                {rows}
            ]
        }
    }
}

/// One component on an [`EntityCard`].
fn component_row(component: &ComponentValue, last: bool) -> impl Scene + use<> {
    let icon_name = component_icon(&component.name, component.value.is_some());
    let mono = |text: &str, color: Color| {
        let text = Text::new(text);
        let font = crate::insert(crate::mono(12.5));
        bsn! {
            text @{font} TextColor(color)
            TextLayout { linebreak: LineBreak::NoWrap }
            Node { flex_shrink: 1.0, min_width: px(0), overflow: Overflow::clip() }
            Pickable::IGNORE
        }
    };
    let value: Vec<Box<dyn Scene>> = component
        .value
        .as_deref()
        .map(|value| Box::new(mono(value, color::MUTED_FOREGROUND)) as Box<dyn Scene>)
        .into_iter()
        .collect();
    let name = mono(&component.name, color::BODY);
    bsn! {
        Node {
            height: px(32),
            align_items: AlignItems::Center,
            column_gap: px(10),
            border: UiRect::bottom(px(if last { 0.0 } else { 1.0 })),
        }
        BorderColor::all(color::HOVER)
        Children [
            @{icon(icon_name, 14.0, color::MUTED_FOREGROUND)}
            --
            @{name} Node { flex_grow: 1.0 }
            --
            {value}
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tuple_struct_unit_and_spaced_values() {
        assert_eq!(
            ComponentValue::parse("Health(50)"),
            ComponentValue {
                name: "Health".into(),
                value: Some("50".into())
            }
        );
        assert_eq!(ComponentValue::parse("Player").value, None);
        assert_eq!(
            ComponentValue::parse("Speed { x: 1.0 }").value.as_deref(),
            Some("x: 1.0")
        );
        assert_eq!(
            ComponentValue::parse("Transform (0, 0, 0)")
                .value
                .as_deref(),
            Some("0, 0, 0")
        );
        assert_eq!(ComponentValue::parse("Empty()").value, None);
        assert_eq!(ComponentValue::parse("  ").name, "");
    }

    #[test]
    fn markers_get_a_flag_and_known_components_their_own_icon() {
        assert_eq!(component_icon("Player", false), "flag");
        assert_eq!(component_icon("Score", true), "puzzle");
        assert_eq!(component_icon("Health", true), "heart");
    }

    #[test]
    fn every_identity_icon_is_in_the_font() {
        let kinds = [
            EcsKind::Entity,
            EcsKind::Component,
            EcsKind::Marker,
            EcsKind::Resource,
            EcsKind::System,
            EcsKind::Observer,
            EcsKind::Event,
            EcsKind::Query,
            EcsKind::Scene,
        ];
        let names = kinds.iter().map(|k| k.icon()).chain([
            "heart",
            "move",
            "video",
            "shapes",
            "palette",
            "text",
            "layout-template",
            "type",
        ]);
        for name in names {
            assert!(
                bevsidian_core::icons::find(name).is_some(),
                "missing icon {name}"
            );
        }
    }
}

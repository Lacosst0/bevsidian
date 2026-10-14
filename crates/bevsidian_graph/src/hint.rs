//! The hint at the bottom of the graph while a note is hovered: its name and what a click
//! and a double click do (a note holding folded notes folds on a click and opens on a double
//! click; any other note opens on a click). Not shown on touch screens, which have no hover.

use bevy::{prelude::*, text::FontWeight};

use bevsidian_core::{Vault, nav::Nav};
use bevsidian_ui::{color, font, insert, radius, shadow, size};

use crate::{GraphNode, GraphState};

/// The hint's pill.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct HintBar;

/// The parts of the hint that change: the note's name, what a click does, and the double
/// click part (shown only for notes holding folded notes).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HintPart {
    #[default]
    Name,
    Click,
    Double,
}

/// A key-like chip ("Click"), as in shadcn/ui's `Kbd`.
fn chip(text: &str) -> impl Scene + use<> {
    let text = Text::new(text);
    bsn! {
        Node {
            height: px(20),
            padding: UiRect::horizontal(px(6)),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(radius::SM)),
            flex_shrink: 0.0,
        }
        BackgroundColor(color::ACCENT)
        Children [
            text
            @{insert(font(size::XS, FontWeight::MEDIUM))}
            TextColor(color::MUTED_FOREGROUND)
        ]
    }
}

/// The hint, centred at the bottom of the graph (hidden until a note is hovered).
pub fn hover_hint() -> impl Scene {
    let lift = shadow::popover();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(16),
            justify_content: JustifyContent::Center,
        }
        Pickable::IGNORE
        Children [
            Node {
                display: Display::None,
                align_items: AlignItems::Center,
                column_gap: px(8),
                padding: UiRect::new(px(12), px(8), px(6), px(6)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(radius::LG)),
            }
            HintBar
            BackgroundColor(color::CARD)
            BorderColor::all(color::INPUT)
            lift
            Children [
                Text::new("")
                HintPart::Name
                @{insert(font(size::SM, FontWeight::MEDIUM))}
                TextColor(color::FOREGROUND)
                --
                @{chip("Click")}
                --
                Text::new("")
                HintPart::Click
                @{insert(font(size::SM, FontWeight::NORMAL))}
                TextColor(color::MUTED_FOREGROUND)
                --
                Node { column_gap: px(8), align_items: AlignItems::Center }
                HintPart::Double
                Children [
                    @{chip("Double-click")}
                    --
                    Text::new("open")
                    @{insert(font(size::SM, FontWeight::NORMAL))}
                    TextColor(color::MUTED_FOREGROUND)
                ]
            ]
        ]
    }
}

/// Fills the hint for the hovered note.
pub(crate) fn show_hint(
    nav: Res<Nav>,
    vault: Res<Vault>,
    state: Res<GraphState>,
    nodes: Query<&GraphNode>,
    mut bars: Query<&mut Node, With<HintBar>>,
    mut parts: Query<(&HintPart, Option<&mut Text>, Option<&mut Node>), Without<HintBar>>,
    mut last: Local<Option<(Entity, bool, bool)>>,
) {
    let hovered = state
        .hovered
        .filter(|_| state.dragging.is_none() && !nav.touch.get_untracked())
        .and_then(|e| nodes.get(e).ok().map(|n| (e, n)));
    let key = hovered.map(|(e, n)| (e, n.open, state.path.is_empty()));
    if *last == key && !bars.is_empty() {
        return;
    }
    *last = key;
    for mut bar in &mut bars {
        bar.display = if hovered.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
    let Some((_, node)) = hovered else {
        return;
    };
    let foldable = state.path.is_empty() && node.folds > 0;
    let name = vault.get_untracked().notes[node.note].name.clone();
    let click = match (foldable, node.open) {
        (false, _) => "open".to_owned(),
        (true, false) => format!("show {} more", node.folds),
        (true, true) => format!("fold {} away", node.folds),
    };
    for (part, text, layout) in &mut parts {
        match part {
            HintPart::Name => {
                if let Some(mut text) = text {
                    text.0.clone_from(&name);
                }
            }
            HintPart::Click => {
                if let Some(mut text) = text {
                    text.0.clone_from(&click);
                }
            }
            HintPart::Double => {
                if let Some(mut layout) = layout {
                    layout.display = if foldable {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
            }
        }
    }
}

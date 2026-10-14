//! Find in the note (`Ctrl+F`): a bar over the top right of the reading pane. Every match in
//! the note is marked and the current one stands out; Enter, F3 and the arrows step through
//! them (Shift goes back), scrolling each into view. Escape or the close button closes it.

use bevy::{prelude::*, ui::UiGlobalTransform, ui_widgets::Activate};
use bevy_radioactive::view::view;
use bevy_ui_commonmark::MdRoot;
use bevy_ui_textselect::{TextBlocks, TextMark, TextMarks};

use bevsidian_core::nav::{LayoutMode, Nav, NotePane, View};
use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<Matches>()
        .add_systems(Update, (find_matches, mark_matches).chain());
}

/// The matches of the current query in the open note: block and byte range.
#[derive(Resource, Default)]
struct Matches {
    /// What they were found for: the query (lowercase), the note's root and its block count.
    key: Option<(String, Entity, usize)>,
    found: Vec<(Entity, usize, usize)>,
    /// The match last scrolled to.
    shown: Option<usize>,
}

/// Searches the note when the query or the note changes.
fn find_matches(
    nav: Res<Nav>,
    texts: TextBlocks,
    roots: Query<Entity, With<MdRoot>>,
    panes: Query<(), With<NotePane>>,
    parents: Query<&ChildOf>,
    mut matches: ResMut<Matches>,
) {
    let query = nav.find_query.get_untracked().to_ascii_lowercase();
    let open = nav.find.get_untracked() && nav.view.get_untracked() == View::Note;
    // The note in the reading pane (other markdown, e.g. in popovers, is not searched).
    let root = roots
        .iter()
        .find(|&root| parents.iter_ancestors(root).any(|e| panes.contains(e)));
    let (Some(root), true, false) = (root, open, query.is_empty()) else {
        if matches.key.is_some() {
            *matches = Matches::default();
            nav.find_count.set_if_neq(0);
        }
        return;
    };
    let blocks = texts.blocks_in(root);
    let key = (query.clone(), root, blocks.len());
    if matches.key.as_ref() == Some(&key) {
        return;
    }
    let mut found = Vec::new();
    for block in blocks {
        // Not laid out yet: try again next frame.
        let Some(text) = texts.text(block) else {
            return;
        };
        // ASCII case folding keeps byte offsets.
        let text = text.to_ascii_lowercase();
        found.extend(
            text.match_indices(&query)
                .map(|(at, _)| (block, at, at + query.len())),
        );
    }
    // A new query starts from the first match; the same one in a re-rendered note keeps its
    // place.
    if matches.key.as_ref().is_none_or(|(old, ..)| *old != query) {
        nav.find_index.set_if_neq(0);
    }
    nav.find_count.set_if_neq(found.len());
    *matches = Matches {
        key: Some(key),
        found,
        shown: None,
    };
}

/// How far down the reading pane (share of its height) the current match is scrolled to.
const SCROLL_TO: f32 = 0.35;

/// Marks the matches and scrolls the current one into view.
fn mark_matches(
    nav: Res<Nav>,
    texts: TextBlocks,
    mut matches: ResMut<Matches>,
    mut marks: ResMut<TextMarks>,
    mut panes: Query<(&ComputedNode, &UiGlobalTransform, &mut ScrollPosition), With<NotePane>>,
) {
    let current = nav
        .find_index
        .get_untracked()
        .min(matches.found.len().saturating_sub(1));
    let wanted = TextMarks(
        matches
            .found
            .iter()
            .enumerate()
            .map(|(i, &(block, start, end))| TextMark {
                block,
                start,
                end,
                color: if i == current {
                    color::FIND_CURRENT
                } else {
                    color::FIND_MATCH
                },
            })
            .collect(),
    );
    if *marks != wanted {
        *marks = wanted;
    }
    let Some(&(block, start, end)) = matches.found.get(current) else {
        return;
    };
    if matches.shown == Some(current) {
        return;
    }
    let (Some(rect), Ok((pane, at, mut scroll))) =
        (texts.range_rect(block, start, end), panes.single_mut())
    else {
        return;
    };
    matches.shown = Some(current);
    let top = at.translation.y - pane.size.y / 2.0;
    let bottom = top + pane.size.y;
    let margin = pane.size.y * 0.15;
    if rect.min.y >= top + margin && rect.max.y <= bottom - margin {
        return;
    }
    let scale = pane.inverse_scale_factor;
    let end_y = ((pane.content_size().y - pane.size.y) * scale).max(0.0);
    let y = scroll.y + (rect.min.y - top - pane.size.y * SCROLL_TO) * scale;
    // The pane's smooth scrolling eases there.
    scroll.y = y.clamp(0.0, end_y);
}

/// The bar: the query, which match is current of how many, previous / next, close.
pub fn find_bar(nav: Nav) -> impl Scene {
    let (open, mode) = (nav.find.clone(), nav.mode.clone());
    let (query, index, count) = (
        nav.find_query.clone(),
        nav.find_index.clone(),
        nav.find_count.clone(),
    );
    let value = Some(nav.find_query.clone());
    let shadow = shadow::popover();
    view! {
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: $(if $mode == LayoutMode::Compact { px(12) } else { px(24) }),
            left: $(if $mode == LayoutMode::Compact { px(12) } else { Val::Auto }),
        }
        GlobalZIndex(layer::TOOLBAR)
        Pickable::IGNORE
        Children [
            if $open {
                let (shadow, value) = (shadow.clone(), value.clone());
                let (previous, next, close) = (nav.clone(), nav.clone(), nav.find.clone());
                @{popover()}
                shadow
                Node { flex_grow: 1.0, flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(4), padding: UiRect::all(px(4)) }
                Children [
                    @InputField { @icon: "search", @placeholder: "Find in note", @value: value, @autofocus: true }
                    Node { flex_grow: 1.0, min_width: px(0), width: px(220), border: UiRect::all(px(0)) }
                    BackgroundColor(Color::NONE)
                    --
                    Text($({
                        let (total, query) = ($count, $query);
                        if query.is_empty() {
                            String::new()
                        } else if total == 0 {
                            "No results".to_owned()
                        } else {
                            format!("{} of {total}", $index.min(total - 1) + 1)
                        }
                    }))
                    @{insert(font(size::SM, bevy::text::FontWeight::NORMAL))}
                    TextColor(color::MUTED_FOREGROUND)
                    Node { flex_shrink: 0.0, margin: UiRect::horizontal(px(4)) }
                    Pickable::IGNORE
                    --
                    @{vertical_separator()}
                    --
                    @{icon_button(Variant::Ghost, Size::Icon, "arrow-up")}
                    Tooltip::new("Previous (Shift+Enter)")
                    on(move |_: On<Activate>| previous.step_find(-1))
                    --
                    @{icon_button(Variant::Ghost, Size::Icon, "arrow-down")}
                    Tooltip::new("Next (Enter)")
                    on(move |_: On<Activate>| next.step_find(1))
                    --
                    @{icon_button(Variant::Ghost, Size::Icon, "x")}
                    Tooltip::new("Close (Esc)")
                    on(move |_: On<Activate>| close.set(false))
                ]
            }
        ]
    }
}

//! Previous / next page links (shadcn/ui `Pagination`, laid out like the pager at the end of
//! its docs pages): a bordered link with the direction above the page's title.

use bevy::{prelude::*, text::FontWeight};

use crate::{
    components::{
        button::{Size, Variant, button},
        icon::tinted_icon,
        typography::tinted,
    },
    surface::Tint,
    tokens::font::size,
};

/// Which way a [`page_link`] goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageDirection {
    Previous,
    Next,
}

/// A link to the previous or next page: `label` (e.g. "Previous") with an arrow, over the
/// page's `title`. It takes half of a row; add an `on(|_: On<Activate>| ...)` handler.
pub fn page_link(direction: PageDirection, label: &str, title: &str) -> impl Scene + use<> {
    let (icon, align) = match direction {
        PageDirection::Previous => ("chevron-left", AlignItems::FlexStart),
        PageDirection::Next => ("chevron-right", AlignItems::FlexEnd),
    };
    let arrow = tinted_icon(icon, 14.0, Tint::MUTED);
    let label = tinted(label, size::XS, FontWeight::NORMAL, Tint::MUTED);
    let caption: Vec<Box<dyn Scene>> = match direction {
        PageDirection::Previous => vec![Box::new(arrow), Box::new(label)],
        PageDirection::Next => vec![Box::new(label), Box::new(arrow)],
    };
    bsn! {
        @{button(Variant::Outline, Size::Lg)}
        Node {
            height: Val::Auto,
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            align_items: align,
            row_gap: px(4),
            padding: UiRect::axes(px(16), px(12)),
        }
        Children [
            Node { align_items: AlignItems::Center, column_gap: px(4) }
            Children [ {caption} ]
            --
            @{tinted(title, size::SM, FontWeight::MEDIUM, Tint::TEXT)}
        ]
    }
}

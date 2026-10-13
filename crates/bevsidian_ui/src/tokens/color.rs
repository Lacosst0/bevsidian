//! Semantic colours: shadcn/ui's dark "neutral" theme, plus one orange accent for links.
//!
//! Overlays (hover, selection, borders) are translucent white, so they read the same on the
//! page, on a sidebar and inside a card.
//!
//! Bevy blends in linear light, while CSS blends sRGB values, so a translucent colour looks far
//! stronger here than the same alpha in a browser: white at 10 % lands near `#595959` instead
//! of `#222222`. The alphas below are the ones that match shadcn's CSS alpha over the page
//! colour (noted next to each).

use bevy::prelude::Color;

/// `#0a0a0a`: the page.
pub const BACKGROUND: Color = Color::srgb(0.039, 0.039, 0.039);
/// `#171717`: sidebars, dialogs, popovers, cards and code headers.
pub const CARD: Color = Color::srgb(0.090, 0.090, 0.090);
/// `#262626`: secondary buttons, tab tracks, keys, inline code.
pub const ACCENT: Color = Color::srgb(0.149, 0.149, 0.149);
/// `#303030`: a hovered [`ACCENT`] fill.
pub const ACCENT_HOVER: Color = Color::srgb(0.188, 0.188, 0.188);

/// A hovered row or ghost button (CSS: white 6 %).
pub const HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.0065);
/// The selected row: the open note, the active filter (CSS: white 10 %).
pub const SELECTED: Color = Color::srgba(1.0, 1.0, 1.0, 0.0134);

/// Dividers and container borders (CSS: white 10 %).
pub const BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.0134);
/// Borders of inputs and outline buttons (CSS: white 15 %).
pub const INPUT: Color = Color::srgba(1.0, 1.0, 1.0, 0.025);
/// The fill of inputs and outline buttons (CSS: white 4.5 %).
pub const INPUT_BG: Color = Color::srgba(1.0, 1.0, 1.0, 0.0043);
/// A hovered [`INPUT_BG`] (CSS: white 8 %).
pub const INPUT_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.0097);

/// `#fafafa`: headings, labels, primary text.
pub const FOREGROUND: Color = Color::srgb(0.980, 0.980, 0.980);
/// `#e5e5e5`: note prose and list rows.
pub const BODY: Color = Color::srgb(0.898, 0.898, 0.898);
/// `#a1a1a1`: secondary text, group labels, idle icons.
pub const MUTED_FOREGROUND: Color = Color::srgb(0.631, 0.631, 0.631);
/// `#737373`: decorative glyphs only (`#` of a tag, chevrons); never body text.
pub const SUBTLE_FOREGROUND: Color = Color::srgb(0.451, 0.451, 0.451);
/// `#525252`: disabled labels and icons.
pub const DISABLED_FOREGROUND: Color = Color::srgb(0.322, 0.322, 0.322);

/// `#fafafa`: the primary button.
pub const PRIMARY: Color = FOREGROUND;
/// `#e5e5e5`: a hovered primary button.
pub const PRIMARY_HOVER: Color = BODY;
/// `#171717`: text on [`PRIMARY`].
pub const PRIMARY_FOREGROUND: Color = CARD;

/// `#fb923c` (Tailwind orange-400): links, the current graph node, checked boxes. The only hue
/// in the interface besides warnings and errors.
pub const LINK: Color = Color::srgb(0.984, 0.573, 0.235);
/// The link colour, faint: the code line that just ran (CSS: 14 %).
pub const LINK_SOFT: Color = Color::srgba(0.984, 0.573, 0.235, 0.025);
/// The link colour, stronger: text selection, the halo of the current graph node (CSS: 25 %).
pub const LINK_RING: Color = Color::srgba(0.984, 0.573, 0.235, 0.06);

/// `#fbbf24`: warning callouts.
pub const WARNING: Color = Color::srgb(0.984, 0.749, 0.141);
/// `#f87171`: caution callouts and errors.
pub const DESTRUCTIVE: Color = Color::srgb(0.973, 0.443, 0.443);

/// Behind dialogs and drawers (CSS: black 60 %).
pub const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);
/// Drop shadows.
pub const SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);

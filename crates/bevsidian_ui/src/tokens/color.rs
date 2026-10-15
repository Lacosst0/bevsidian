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
/// The link colour, stronger: text selection (CSS: 25 %).
pub const LINK_RING: Color = Color::srgba(0.984, 0.573, 0.235, 0.06);

/// `#fbbf24`: warning callouts.
pub const WARNING: Color = Color::srgb(0.984, 0.749, 0.141);
/// `#f87171`: caution callouts and errors.
pub const DESTRUCTIVE: Color = Color::srgb(0.973, 0.443, 0.443);
/// `#4ade80` (Tailwind green-400): what a code variant adds.
pub const SUCCESS: Color = Color::srgb(0.290, 0.871, 0.502);

/// Faint washes behind diff lines: added, removed, changed (CSS: 12 %).
pub const SUCCESS_SOFT: Color = Color::srgba(0.290, 0.871, 0.502, 0.02);
pub const DESTRUCTIVE_SOFT: Color = Color::srgba(0.973, 0.443, 0.443, 0.02);
pub const WARNING_SOFT: Color = Color::srgba(0.984, 0.749, 0.141, 0.02);

/// Find-in-page matches, and the current one (CSS: amber 30 %, orange 55 %).
pub const FIND_MATCH: Color = Color::srgba(0.984, 0.749, 0.141, 0.09);
pub const FIND_CURRENT: Color = Color::srgba(0.984, 0.573, 0.235, 0.3);

// The graph's star map. The sky is `BACKGROUND`; these are opaque except the nebulae,
// which lie over it.

/// `#121212`: the celestial grid (circles and spokes around the capital).
pub const MAP_GRID: Color = Color::srgb(0.071, 0.071, 0.071);
/// The faintest and brightest stars of the background field.
pub const MAP_FIELD_FAINT: Color = Color::srgb(0.3, 0.3, 0.3);
pub const MAP_FIELD_BRIGHT: Color = Color::srgb(0.85, 0.85, 0.85);
/// A nebula's middle: white over the sky (CSS: white 8 %).
pub const MAP_NEBULA: Color = Color::srgba(1.0, 1.0, 1.0, 0.009);
/// Behind a note's name: the sky, opaque, so lines pass under the name.
pub const MAP_PLATE: Color = BACKGROUND;
/// `#3a3a3a`: the outline of a note's shape.
pub const MAP_RING: Color = Color::srgb(0.227, 0.227, 0.227);
/// `#e5e5e5`: the inside of a note's shape once it is read.
pub const MAP_FILL_READ: Color = Color::srgb(0.898, 0.898, 0.898);
/// `#5a5a5a`: the outline of a note's shape once it is read.
pub const MAP_RING_LIT: Color = Color::srgb(0.353, 0.353, 0.353);
/// The colour of route `index` of `count` on the map: the routes take even steps along
/// [`MAP_ROUTES`] in the order they sit round the map, so neighbours have neighbouring hues
/// that still tell apart.
pub fn map_route(index: usize, count: usize) -> Color {
    let last = MAP_ROUTES.len() - 1;
    let step = (index * last + count.saturating_sub(1).max(1) / 2) / count.saturating_sub(1).max(1);
    MAP_ROUTES[step.min(last)]
}
/// The route colours, round the colour wheel from yellow to rose (Tailwind's 400 shades:
/// bright on the sky). No red, amber or orange: orange is the note being read.
pub const MAP_ROUTES: [Color; 14] = [
    Color::srgb(0.980, 0.800, 0.082), // yellow #facc15
    Color::srgb(0.639, 0.902, 0.208), // lime #a3e635
    Color::srgb(0.290, 0.871, 0.502), // green #4ade80
    Color::srgb(0.204, 0.827, 0.600), // emerald #34d399
    Color::srgb(0.176, 0.831, 0.749), // teal #2dd4bf
    Color::srgb(0.133, 0.827, 0.933), // cyan #22d3ee
    Color::srgb(0.220, 0.741, 0.973), // sky #38bdf8
    Color::srgb(0.376, 0.647, 0.980), // blue #60a5fa
    Color::srgb(0.506, 0.549, 0.973), // indigo #818cf8
    Color::srgb(0.655, 0.545, 0.980), // violet #a78bfa
    Color::srgb(0.753, 0.518, 0.988), // purple #c084fc
    Color::srgb(0.910, 0.475, 0.976), // fuchsia #e879f9
    Color::srgb(0.957, 0.447, 0.714), // pink #f472b6
    Color::srgb(0.984, 0.443, 0.522), // rose #fb7185
];
/// `#e5e5e5`: the main quest's lines, and the lines leaving it.
pub const MAP_LINE: Color = Color::srgb(0.898, 0.898, 0.898);
/// `#363636`: the other lines, with the whole map shown.
pub const MAP_LINE_FAINT: Color = Color::srgb(0.21, 0.21, 0.21);
/// `#333333`: landmarks drawn in the sky.
pub const MAP_INK: Color = Color::srgb(0.2, 0.2, 0.2);
/// `#595959`: map labels.
pub const MAP_INK_STRONG: Color = Color::srgb(0.35, 0.35, 0.35);
/// `#7c4a21`: a hovered note's links, the link colour dimmed over the sky.
pub const MAP_LINK: Color = Color::srgb(0.486, 0.29, 0.129);

/// Behind dialogs and drawers (CSS: black 60 %).
pub const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);
/// Drop shadows.
pub const SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);

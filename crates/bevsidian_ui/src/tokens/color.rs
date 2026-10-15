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
/// `#525252`: the inside of a note's shape once it is read (unread ones are `CARD`).
pub const MAP_FILL_READ: Color = Color::srgb(0.322, 0.322, 0.322);
/// `#5a5a5a`: the outline of a note's shape once it is read.
pub const MAP_RING_LIT: Color = Color::srgb(0.353, 0.353, 0.353);
/// The colour of route `index` of `count` on the map, for a route that names none: the
/// routes take steps along [`MAP_ROUTES`] in the order they sit round the map, every
/// second colour first, so neighbours never get two shades of one hue.
pub fn map_route(index: usize, count: usize) -> Color {
    let n = MAP_ROUTES.len();
    let spread = (index * n / count.max(1)) % n;
    let step = (spread * 2 + spread * 2 / n) % n;
    MAP_ROUTES[step].1
}
/// The route colour named `name` (a route's `color:` property), if there is one.
pub fn map_route_named(name: &str) -> Option<Color> {
    MAP_ROUTES
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name.trim()))
        .map(|&(_, c)| c)
}
/// The route colours, round the colour wheel (mostly Tailwind's 400 shades: bright on the
/// sky), each a hue of its own so routes tell apart at a glance. No orange or amber: orange
/// is the note being read.
pub const MAP_ROUTES: [(&str, Color); 14] = [
    ("yellow", Color::srgb(0.980, 0.800, 0.082)),  // #facc15
    ("sand", Color::srgb(0.851, 0.737, 0.561)),    // #d9bc8f
    ("lime", Color::srgb(0.639, 0.902, 0.208)),    // #a3e635
    ("green", Color::srgb(0.290, 0.871, 0.502)),   // #4ade80
    ("mint", Color::srgb(0.600, 0.965, 0.894)),    // #99f6e4
    ("teal", Color::srgb(0.078, 0.722, 0.651)),    // #14b8a6
    ("cyan", Color::srgb(0.133, 0.827, 0.933)),    // #22d3ee
    ("blue", Color::srgb(0.231, 0.510, 0.965)),    // #3b82f6
    ("indigo", Color::srgb(0.647, 0.706, 0.988)),  // #a5b4fc
    ("purple", Color::srgb(0.753, 0.518, 0.988)),  // #c084fc
    ("fuchsia", Color::srgb(0.910, 0.475, 0.976)), // #e879f9
    ("pink", Color::srgb(0.976, 0.659, 0.831)),    // #f9a8d4
    ("rose", Color::srgb(0.957, 0.247, 0.369)),    // #f43f5e
    ("red", Color::srgb(0.937, 0.267, 0.267)),     // #ef4444
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

/// The colours of the reader's pixel sprite and voxel model ("Make it yours"): PICO-8's
/// sixteen, made for small pixel art.
pub const PIXEL_PALETTE: [Color; 16] = [
    Color::srgb(0.000, 0.000, 0.000), // black #000000
    Color::srgb(0.114, 0.169, 0.325), // dark blue #1d2b53
    Color::srgb(0.494, 0.145, 0.325), // dark purple #7e2553
    Color::srgb(0.000, 0.529, 0.317), // dark green #008751
    Color::srgb(0.671, 0.322, 0.212), // brown #ab5236
    Color::srgb(0.373, 0.341, 0.310), // dark grey #5f574f
    Color::srgb(0.761, 0.765, 0.780), // light grey #c2c3c7
    Color::srgb(1.000, 0.945, 0.910), // white #fff1e8
    Color::srgb(1.000, 0.000, 0.302), // red #ff004d
    Color::srgb(1.000, 0.639, 0.000), // orange #ffa300
    Color::srgb(1.000, 0.925, 0.153), // yellow #ffec27
    Color::srgb(0.000, 0.894, 0.212), // green #00e436
    Color::srgb(0.161, 0.678, 1.000), // blue #29adff
    Color::srgb(0.514, 0.463, 0.612), // lavender #83769c
    Color::srgb(1.000, 0.467, 0.659), // pink #ff77a8
    Color::srgb(1.000, 0.800, 0.667), // peach #ffccaa
];

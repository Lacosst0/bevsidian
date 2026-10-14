//! The world map: where every note sits in the graph, drawn by hand.
//!
//! A note with `map: true` in its front matter holds the map in a ```` ```map ```` block, one
//! line per thing on it. Coordinates are world units, `x` to the right and `y` up, with the
//! hub at `0, 0`:
//!
//! ```text
//! # A note: its name, where it sits, and optionally its shape and size.
//! Hello Bevy            @     0, -420   diamond large
//! # Objects: a region's name (with a caption), a landmark (a Lucide icon) and a nebula.
//! label: Pixel Plains | 2D   @ -1500,  420   size 30
//! landmark: tree-pine        @ -1320,  640   size 48
//! nebula:                    @ -1500,  300   r 700
//! ```
//!
//! Lines starting with `#` are comments. The graph edits the map in place (see [`set_place`],
//! [`add_object`], [`set_object`], [`remove_object`]): only the line that changed is rewritten,
//! so comments and the order of lines stay as written.

use std::fmt::Write as _;

use bevy::math::Vec2;

/// The outline of a note on the map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shape {
    Circle,
    Diamond,
    Hexagon,
    Square,
    Star,
    Octagon,
    Pentagon,
}

impl Shape {
    /// Every shape, in the order the editor offers them.
    pub const ALL: [Shape; 7] = [
        Shape::Circle,
        Shape::Diamond,
        Shape::Hexagon,
        Shape::Square,
        Shape::Pentagon,
        Shape::Octagon,
        Shape::Star,
    ];

    /// Its name in the map, which is also its Lucide icon.
    pub fn name(self) -> &'static str {
        match self {
            Shape::Circle => "circle",
            Shape::Diamond => "diamond",
            Shape::Hexagon => "hexagon",
            Shape::Square => "square",
            Shape::Star => "star",
            Shape::Octagon => "octagon",
            Shape::Pentagon => "pentagon",
        }
    }

    pub fn parse(word: &str) -> Option<Shape> {
        Shape::ALL.into_iter().find(|s| s.name() == word)
    }
}

/// How big a note is drawn, beside what its kind gives it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Scale {
    Small,
    #[default]
    Normal,
    Large,
}

impl Scale {
    pub fn factor(self) -> f32 {
        match self {
            Scale::Small => 0.8,
            Scale::Normal => 1.0,
            Scale::Large => 1.35,
        }
    }
}

/// Where a note sits.
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub name: String,
    pub at: Vec2,
    /// `None`: the shape its kind gives it (see the graph).
    pub shape: Option<Shape>,
    pub scale: Scale,
}

/// Something on the map that is not a note.
#[derive(Clone, Debug, PartialEq)]
pub enum MapObject {
    /// A region's name in large letters, with a smaller caption under it (may be empty).
    Label {
        text: String,
        caption: String,
        at: Vec2,
        size: f32,
    },
    /// A Lucide icon drawn as map art: trees, mountains, a castle.
    Landmark { icon: String, at: Vec2, size: f32 },
    /// A cloud of nebula, `radius` across, fading out to its edge (`land:` in older maps).
    Nebula { at: Vec2, radius: f32 },
}

impl MapObject {
    pub fn at(&self) -> Vec2 {
        match self {
            MapObject::Label { at, .. }
            | MapObject::Landmark { at, .. }
            | MapObject::Nebula { at, .. } => *at,
        }
    }

    pub fn set_at(&mut self, to: Vec2) {
        match self {
            MapObject::Label { at, .. }
            | MapObject::Landmark { at, .. }
            | MapObject::Nebula { at, .. } => *at = to,
        }
    }
}

pub const LABEL_SIZE: f32 = 30.0;
pub const LANDMARK_SIZE: f32 = 48.0;
pub const NEBULA_RADIUS: f32 = 400.0;

/// The parsed map of the vault.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldMap {
    /// The note holding it.
    pub note: usize,
    pub places: Vec<Place>,
    /// In the order of their lines (the index [`set_object`] and [`remove_object`] take).
    pub objects: Vec<MapObject>,
}

impl WorldMap {
    /// Reads the ```` ```map ```` block of a note's source.
    pub fn parse(note: usize, source: &str) -> Self {
        let mut map = WorldMap {
            note,
            ..Self::default()
        };
        let Some((start, end)) = block(source) else {
            return map;
        };
        for line in source.lines().skip(start).take(end - start) {
            match parse_line(line) {
                Some(Entry::Place(place)) => map.places.push(place),
                Some(Entry::Object(object)) => map.objects.push(object),
                None => {}
            }
        }
        map
    }

    pub fn place(&self, name: &str) -> Option<&Place> {
        self.places
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    }
}

enum Entry {
    Place(Place),
    Object(MapObject),
}

/// The lines of the map block (between its fences), as `start..end` line indices.
fn block(source: &str) -> Option<(usize, usize)> {
    let mut start = None;
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        match start {
            None if trimmed.starts_with("```") && trimmed[3..].trim() == "map" => {
                start = Some(i + 1);
            }
            Some(s) if trimmed.starts_with("```") => return Some((s, i)),
            _ => {}
        }
    }
    None
}

fn parse_line(line: &str) -> Option<Entry> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (left, right) = line.rsplit_once('@')?;
    let mut words = right
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|w| !w.is_empty());
    let x: f32 = words.next()?.parse().ok()?;
    let y: f32 = words.next()?.parse().ok()?;
    let at = Vec2::new(x, y);
    let rest: Vec<&str> = words.collect();
    let number = |key: &str| {
        rest.iter()
            .position(|w| *w == key)
            .and_then(|i| rest.get(i + 1)?.parse::<f32>().ok())
    };
    let left = left.trim();
    let object = |kind: &str| {
        left.strip_prefix(kind)
            .and_then(|r| r.strip_prefix(':'))
            .map(str::trim)
    };
    if let Some(text) = object("label") {
        let (text, caption) = text.split_once('|').unwrap_or((text, ""));
        return Some(Entry::Object(MapObject::Label {
            text: text.trim().to_owned(),
            caption: caption.trim().to_owned(),
            at,
            size: number("size").unwrap_or(LABEL_SIZE),
        }));
    }
    if let Some(icon) = object("landmark") {
        return Some(Entry::Object(MapObject::Landmark {
            icon: icon.to_owned(),
            at,
            size: number("size").unwrap_or(LANDMARK_SIZE),
        }));
    }
    if object("nebula").is_some() || object("land").is_some() {
        return Some(Entry::Object(MapObject::Nebula {
            at,
            radius: number("r").unwrap_or(NEBULA_RADIUS),
        }));
    }
    if left.is_empty() {
        return None;
    }
    let shape = rest.iter().find_map(|w| Shape::parse(w));
    let scale = if rest.contains(&"large") {
        Scale::Large
    } else if rest.contains(&"small") {
        Scale::Small
    } else {
        Scale::Normal
    };
    Some(Entry::Place(Place {
        name: left.to_owned(),
        at,
        shape,
        scale,
    }))
}

/// A coordinate as written in the map: whole units.
fn coords(at: Vec2) -> String {
    format!("{:>6}, {:>6}", at.x.round() as i32, at.y.round() as i32)
}

/// The line of a place.
pub fn place_line(place: &Place) -> String {
    let mut options: Vec<&str> = place.shape.map(Shape::name).into_iter().collect();
    match place.scale {
        Scale::Small => options.push("small"),
        Scale::Large => options.push("large"),
        Scale::Normal => {}
    }
    let mut line = format!("{:<30}@ {}", place.name, coords(place.at));
    if !options.is_empty() {
        line.push_str("   ");
        line.push_str(&options.join(" "));
    }
    line
}

/// The line of an object.
pub fn object_line(object: &MapObject) -> String {
    match object {
        MapObject::Label {
            text,
            caption,
            at,
            size,
        } => {
            let text = if caption.is_empty() {
                text.clone()
            } else {
                format!("{text} | {caption}")
            };
            let mut line = format!("{:<30}@ {}", format!("label: {text}"), coords(*at));
            if (*size - LABEL_SIZE).abs() > f32::EPSILON {
                let _ = write!(line, "   size {}", size.round() as i32);
            }
            line
        }
        MapObject::Landmark { icon, at, size } => {
            let mut line = format!("{:<30}@ {}", format!("landmark: {icon}"), coords(*at));
            if (*size - LANDMARK_SIZE).abs() > f32::EPSILON {
                let _ = write!(line, "   size {}", size.round() as i32);
            }
            line
        }
        MapObject::Nebula { at, radius } => format!(
            "{:<30}@ {}   r {}",
            "nebula:",
            coords(*at),
            radius.round() as i32
        ),
    }
}

/// The source with its map block's lines replaced by `edit` (given the block's lines, it
/// changes them in place). Without a block, one is added at the end.
fn edit_block(source: &str, edit: impl FnOnce(&mut Vec<String>)) -> String {
    let mut lines: Vec<String> = source.lines().map(str::to_owned).collect();
    let (start, end) = block(source).unwrap_or_else(|| {
        lines.extend([String::new(), "```map".to_owned(), "```".to_owned()]);
        (lines.len() - 1, lines.len() - 1)
    });
    let mut inside: Vec<String> = lines[start..end].to_vec();
    edit(&mut inside);
    lines.splice(start..end, inside);
    let mut out = lines.join("\n");
    if source.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Moves (or reshapes) a note: rewrites its line, or adds one at the end of the block.
pub fn set_place(source: &str, place: &Place) -> String {
    edit_block(source, |lines| {
        let line = place_line(place);
        let at = lines.iter().position(|l| {
            matches!(parse_line(l), Some(Entry::Place(p)) if p.name.eq_ignore_ascii_case(&place.name))
        });
        match at {
            Some(i) => lines[i] = line,
            None => lines.push(line),
        }
    })
}

/// The block line of the `index`-th object.
fn object_index(lines: &[String], index: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(parse_line(l), Some(Entry::Object(_))))
        .nth(index)
        .map(|(i, _)| i)
}

pub fn add_object(source: &str, object: &MapObject) -> String {
    edit_block(source, |lines| lines.push(object_line(object)))
}

pub fn set_object(source: &str, index: usize, object: &MapObject) -> String {
    edit_block(source, |lines| {
        if let Some(i) = object_index(lines, index) {
            lines[i] = object_line(object);
        }
    })
}

pub fn remove_object(source: &str, index: usize) -> String {
    edit_block(source, |lines| {
        if let Some(i) = object_index(lines, index) {
            lines.remove(i);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "---\nmap: true\n---\n# Map\n\nText.\n\n```map\n# The hub\nBevy @ 0, 0 octagon large\nHello Bevy        @ 10, -420\nlabel: Pixel Plains | 2D @ -1500, 420 size 40\nlandmark: tree-pine @ 3, 4\nland: @ -1500, 300 r 700\nnot a line\n```\n\nAfter.\n";

    #[test]
    fn places_and_objects_are_read() {
        let map = WorldMap::parse(7, SOURCE);
        assert_eq!(map.note, 7);
        assert_eq!(
            map.places,
            vec![
                Place {
                    name: "Bevy".into(),
                    at: Vec2::ZERO,
                    shape: Some(Shape::Octagon),
                    scale: Scale::Large
                },
                Place {
                    name: "Hello Bevy".into(),
                    at: Vec2::new(10.0, -420.0),
                    shape: None,
                    scale: Scale::Normal
                },
            ]
        );
        assert_eq!(
            map.objects,
            vec![
                MapObject::Label {
                    text: "Pixel Plains".into(),
                    caption: "2D".into(),
                    at: Vec2::new(-1500.0, 420.0),
                    size: 40.0
                },
                MapObject::Landmark {
                    icon: "tree-pine".into(),
                    at: Vec2::new(3.0, 4.0),
                    size: LANDMARK_SIZE
                },
                MapObject::Nebula {
                    at: Vec2::new(-1500.0, 300.0),
                    radius: 700.0
                },
            ]
        );
        assert!(map.place("hello bevy").is_some());
    }

    #[test]
    fn edits_rewrite_one_line() {
        let moved = set_place(
            SOURCE,
            &Place {
                name: "Hello Bevy".into(),
                at: Vec2::new(-20.4, 100.6),
                shape: Some(Shape::Diamond),
                scale: Scale::Normal,
            },
        );
        assert!(moved.contains("# The hub\nBevy @ 0, 0 octagon large\nHello Bevy"));
        assert!(moved.ends_with("```\n\nAfter.\n"));
        let map = WorldMap::parse(0, &moved);
        let place = map.place("Hello Bevy").unwrap();
        assert_eq!(place.at, Vec2::new(-20.0, 101.0));
        assert_eq!(place.shape, Some(Shape::Diamond));
        assert_eq!(map.places.len(), 2);

        let added = set_place(
            &moved,
            &Place {
                name: "Sprites".into(),
                at: Vec2::new(5.0, 5.0),
                shape: None,
                scale: Scale::Small,
            },
        );
        let map = WorldMap::parse(0, &added);
        assert_eq!(map.places.len(), 3);
        assert_eq!(map.place("Sprites").unwrap().scale, Scale::Small);

        let flag = MapObject::Landmark {
            icon: "flag".into(),
            at: Vec2::new(1.0, 2.0),
            size: 60.0,
        };
        let with_flag = add_object(&added, &flag);
        assert_eq!(WorldMap::parse(0, &with_flag).objects[3], flag);
        let mut land = WorldMap::parse(0, &with_flag).objects[2].clone();
        land.set_at(Vec2::new(9.0, 9.0));
        let moved_land = set_object(&with_flag, 2, &land);
        assert_eq!(WorldMap::parse(0, &moved_land).objects[2], land);
        let removed = remove_object(&moved_land, 0);
        let objects = WorldMap::parse(0, &removed).objects;
        assert_eq!(objects.len(), 3);
        assert!(matches!(objects[0], MapObject::Landmark { .. }));
    }

    #[test]
    fn a_note_without_a_map_gets_one() {
        let source = "# Empty";
        let added = add_object(
            source,
            &MapObject::Nebula {
                at: Vec2::ZERO,
                radius: 10.0,
            },
        );
        assert_eq!(WorldMap::parse(0, &added).objects.len(), 1);
        assert!(added.starts_with("# Empty\n\n```map\n"));
        assert!(added.contains("nebula:"), "written with its current name");
    }
}

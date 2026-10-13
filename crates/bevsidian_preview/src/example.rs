//! Examples: small Bevy programs that code blocks show excerpts of, and run.
//!
//! An [`Example`] is set up once: its types, its resources, the systems every preview of it
//! runs (`base`) and the named functions blocks can bring in (`functions`). A code block with
//! `example=NAME` in its info string runs the example with the functions its code names, so
//! several blocks of a note build on one example, each showing (and running) one part of it.
//!
//! Functions are listed with [`startup!`], [`update!`] and [`observer!`], which take the
//! function itself: its name (what a block's code must mention) comes from the identifier.

use std::ops::Range;

use bevy::prelude::*;

/// When Bevy runs a function of an example.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Once, on the first frame.
    Startup,
    /// Every frame.
    Update,
    /// When the event it observes is triggered.
    Observer,
}

/// A system or observer of an example.
pub struct Function {
    /// The function's name, as code blocks mention it.
    pub name: &'static str,
    pub kind: Kind,
    /// Adds it to a preview: observers to the world, systems to the schedule (in `step`).
    pub add: fn(&mut World, &mut Schedule, Step),
}

/// Orders an example's systems: base systems first, then the block's, as listed.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Step(pub usize);

/// Prepares a fresh preview world: registers types and inserts resources.
pub type Setup = Box<dyn Fn(&mut World) + Send + Sync>;

/// A small program that code blocks run parts of.
pub struct Example {
    /// Given the app's world (to share its assets), returns the setup of a preview world.
    pub setup: fn(&World) -> Setup,
    /// Run by every preview, before the block's functions: the rest of the program.
    pub base: &'static [Function],
    /// Run by the blocks whose code names them.
    pub functions: &'static [Function],
    /// Extra functions highlighted when a system runs (e.g. the observers it triggers).
    pub also: &'static [(&'static str, &'static str)],
    pub view: View,
}

/// How a preview shows its world.
#[derive(Clone, Copy, Debug)]
pub enum View {
    /// Entity cards with their component values. Blocks that run more than startup code step
    /// one system at a time, playing at `fps` frames per second.
    Cards { fps: f32 },
    /// A 2D game drawn from its sprites, `size` world units across with the origin in the
    /// middle, playing in real time. Pointer input over it becomes picking events.
    Game { size: Vec2, background: Color },
}

impl Example {
    /// The functions a block runs: the base, then those its code names, as the example lists
    /// them.
    pub fn functions_for(&self, code: &str) -> Vec<&'static Function> {
        self.base
            .iter()
            .chain(self.functions.iter().filter(|f| mentions(code, f.name)))
            .collect()
    }
}

/// A code block's `example=NAME` (and optional `fps=N`) options.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Options {
    pub name: String,
    pub fps: Option<f32>,
}

impl Options {
    pub fn parse(info: &str) -> Option<Self> {
        let option = |key: &str| {
            info.split_whitespace()
                .find_map(|word| word.strip_prefix(key)?.strip_prefix('='))
        };
        Some(Self {
            name: option("example")?.to_owned(),
            fps: option("fps").and_then(|fps| fps.parse().ok()),
        })
    }
}

/// Whether `code` mentions `word` as a whole identifier.
pub fn mentions(code: &str, word: &str) -> bool {
    code.lines().any(|line| line_mentions(line, word))
}

fn line_mentions(line: &str, word: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    line.match_indices(word).any(|(at, _)| {
        !line[..at].chars().next_back().is_some_and(ident)
            && !line[at + word.len()..].chars().next().is_some_and(ident)
    })
}

/// Lines of `fn name` (up to its closing brace), or else the first line naming it.
pub fn code_lines(code: &str, name: &str) -> Option<Range<usize>> {
    let lines: Vec<&str> = code.lines().collect();
    let definition = format!("fn {name}");
    if let Some(start) = lines.iter().position(|l| line_mentions(l, &definition)) {
        let mut depth = 0i32;
        for (i, line) in lines.iter().enumerate().skip(start) {
            for c in line.chars() {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if depth <= 0 && line.contains('}') {
                return Some(start..i + 1);
            }
        }
        return Some(start..lines.len());
    }
    let line = lines.iter().position(|l| line_mentions(l, name))?;
    Some(line..line + 1)
}

/// A system that runs once, on the first frame (`add_systems(Startup, f)`).
macro_rules! startup {
    ($f:ident) => {
        $crate::example::Function {
            name: stringify!($f),
            kind: $crate::example::Kind::Startup,
            add: |_, schedule, step| {
                schedule.add_systems(
                    $f.run_if(::bevy::ecs::schedule::common_conditions::run_once)
                        .in_set(step),
                );
            },
        }
    };
}

/// A system that runs every frame (`add_systems(Update, f)`).
macro_rules! update {
    ($f:ident) => {
        $crate::example::Function {
            name: stringify!($f),
            kind: $crate::example::Kind::Update,
            add: |_, schedule, step| {
                schedule.add_systems($f.in_set(step));
            },
        }
    };
}

/// An observer (`add_observer(f)`).
macro_rules! observer {
    ($f:ident) => {
        $crate::example::Function {
            name: stringify!($f),
            kind: $crate::example::Kind::Observer,
            add: |world, _, _| {
                world.add_observer($f);
            },
        }
    };
}

pub(crate) use {observer, startup, update};

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = "fn a() {\n    x();\n}\n\nfn main() {\n    run(a);\n    b();\n}";

    #[test]
    fn code_lines_find_functions_and_mentions() {
        assert_eq!(code_lines(CODE, "a"), Some(0..3));
        assert_eq!(code_lines(CODE, "main"), Some(4..8));
        assert_eq!(code_lines(CODE, "b"), Some(6..7));
        assert_eq!(code_lines(CODE, "missing"), None);
    }

    #[test]
    fn mentions_are_whole_identifiers() {
        assert!(mentions(CODE, "run"));
        assert!(!mentions(CODE, "ru"));
        assert!(!mentions("spawn_players()", "spawn_player"));
    }

    #[test]
    fn options_come_from_the_info_string() {
        assert_eq!(
            Options::parse("rust example=ecs fps=10"),
            Some(Options {
                name: "ecs".into(),
                fps: Some(10.0)
            })
        );
        assert_eq!(Options::parse("rust preview=menu"), None);
    }
}

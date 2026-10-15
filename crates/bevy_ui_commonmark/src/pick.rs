//! `pick` code blocks: a program whose parts the reader switches on and off.
//!
//! ````text
//! ```rust pick
//! fn main() {
//!     //@ anchor body
//! }
//! //@ option shout Say hello
//! //@ at body
//! println!("hello");
//! ```
//! ````
//!
//! Everything before the first `//@ option` is the base program. `//@ anchor NAME` marks a
//! line where options insert code (the marker itself is not shown). An option is a chip;
//! below its `//@ option ID Label` line, each `//@ at NAME` starts the lines it inserts at
//! that anchor, indented like the anchor. Chosen options are inserted in the order they are
//! written, and the lines they add are highlighted.

use core::ops::Range;
use std::sync::Arc;

use bevy::{prelude::*, ui_widgets::Activate};

use crate::{
    insert,
    scene::{CodeHighlight, CodeText, CopySource, code_spans},
    theme::MdTheme,
};

/// One line of the base program.
#[derive(Clone, Debug, PartialEq, Eq)]
enum BaseLine {
    Code(String),
    Anchor { name: String, indent: String },
}

/// A chip: code to add at some anchors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickOption {
    /// Short id, e.g. `watch`.
    pub id: String,
    /// What the chip says.
    pub label: String,
    inserts: Vec<(String, Vec<String>)>,
}

/// A parsed `pick` block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick {
    base: Vec<BaseLine>,
    /// The chips, in order.
    pub options: Vec<PickOption>,
}

impl Pick {
    /// Reads the block's code.
    pub fn parse(code: &str) -> Self {
        let mut pick = Self {
            base: Vec::new(),
            options: Vec::new(),
        };
        for line in code.lines() {
            let marker = line.trim_start().strip_prefix("//@").map(str::trim);
            let (word, rest) = marker
                .map(|m| m.split_once(char::is_whitespace).unwrap_or((m, "")))
                .unwrap_or_default();
            let rest = rest.trim();
            match (marker.is_some(), word) {
                (true, "anchor") => pick.base.push(BaseLine::Anchor {
                    name: rest.to_owned(),
                    indent: line[..line.len() - line.trim_start().len()].to_owned(),
                }),
                (true, "option") => {
                    let (id, label) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
                    pick.options.push(PickOption {
                        id: id.to_owned(),
                        label: label.trim().to_owned(),
                        inserts: Vec::new(),
                    });
                }
                (true, "at") => {
                    if let Some(option) = pick.options.last_mut() {
                        option.inserts.push((rest.to_owned(), Vec::new()));
                    }
                }
                _ => match pick.options.last_mut() {
                    None => pick.base.push(BaseLine::Code(line.to_owned())),
                    Some(option) => {
                        if let Some((_, lines)) = option.inserts.last_mut() {
                            lines.push(line.to_owned());
                        }
                    }
                },
            }
        }
        // Blank lines closing a fragment belong to the gap between options, not the code.
        for option in &mut pick.options {
            for (_, lines) in &mut option.inserts {
                while lines.last().is_some_and(|l| l.trim().is_empty()) {
                    lines.pop();
                }
            }
        }
        pick
    }

    /// The program with the options in `mask` (bit `i` is option `i`) added, and the line
    /// ranges they added.
    pub fn compose(&self, mask: u32) -> (String, Vec<Range<usize>>) {
        let mut lines: Vec<String> = Vec::new();
        let mut added: Vec<Range<usize>> = Vec::new();
        for line in &self.base {
            match line {
                BaseLine::Code(text) => lines.push(text.clone()),
                BaseLine::Anchor { name, indent } => {
                    for (index, option) in self.options.iter().enumerate() {
                        if mask >> index & 1 == 0 {
                            continue;
                        }
                        for (at, code) in &option.inserts {
                            if at != name || code.is_empty() {
                                continue;
                            }
                            let start = lines.len();
                            for text in code {
                                lines.push(if text.is_empty() {
                                    String::new()
                                } else {
                                    format!("{indent}{text}")
                                });
                            }
                            added.push(start..lines.len());
                        }
                    }
                }
            }
        }
        (lines.join("\n"), added)
    }

    /// Every program that has to compile: the base, each option alone, and all of them.
    pub fn variants(&self) -> Vec<(String, String)> {
        let mut out = vec![("base".to_owned(), self.compose(0).0)];
        for (index, option) in self.options.iter().enumerate() {
            out.push((option.id.clone(), self.compose(1 << index).0));
        }
        let all = (1u32 << self.options.len()) - 1;
        out.push(("all".to_owned(), self.compose(all).0));
        out
    }
}

/// The chosen options of a `pick` block, on its code group.
#[derive(Component, Clone, Debug)]
pub struct MdPick {
    pick: Arc<Pick>,
    lang: Option<String>,
    mask: u32,
}

impl MdPick {
    pub(crate) fn new(pick: Pick, lang: Option<&str>) -> Self {
        Self {
            pick: Arc::new(pick),
            lang: lang.map(str::to_owned),
            mask: 0,
        }
    }
}

/// A chip that toggles option `.0`.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PickChip(pub usize);

/// The row of chips above a pick block's code.
pub(crate) fn chips(pick: &Pick, theme: &MdTheme) -> impl Scene + use<> {
    let font = theme.body.clone();
    let size = theme.base_size * 0.8125;
    let color = theme.text;
    let chips: Vec<Box<dyn Scene>> = pick
        .options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let label = Text::new(option.label.clone());
            let text_font = TextFont {
                font: font.clone(),
                font_size: size.into(),
                ..default()
            };
            let chip = PickChip(index);
            let border = BorderColor::all(theme.border);
            Box::new(bsn! {
                chip
                Node {
                    padding: UiRect::axes(px(10), px(4)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(999)),
                    flex_shrink: 0.0,
                }
                border
                BackgroundColor(Color::NONE)
                bevy::ui_widgets::Button
                bevy::picking::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
                on(move |activate: On<Activate>,
                         parents: Query<&ChildOf>,
                         mut picks: Query<&mut MdPick>| {
                    let group = parents.iter_ancestors(activate.entity).find(|&e| picks.contains(e));
                    if let Some(mut pick) = group.and_then(|g| picks.get_mut(g).ok()) {
                        pick.mask ^= 1 << index;
                    }
                })
                Children [
                    label
                    @{insert(text_font)}
                    TextColor(color)
                    TextLayout::no_wrap()
                    Pickable::IGNORE
                ]
            }) as Box<dyn Scene>
        })
        .collect();
    let surface = BackgroundColor(theme.surface);
    bsn! {
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(8),
            padding: UiRect::all(px(12)),
        }
        surface
        Children [ {chips} ]
    }
}

/// Redraws a pick block's code and chips after a chip is toggled.
pub(crate) fn apply_picks(
    picks: Query<(Entity, &MdPick), Changed<MdPick>>,
    children: Query<&Children>,
    texts: Query<(), With<CodeText>>,
    chips: Query<&PickChip>,
    mut highlights: Query<&mut CodeHighlight>,
    mut sources: Query<&mut CopySource>,
    mut backgrounds: Query<&mut BackgroundColor>,
    mut borders: Query<&mut BorderColor>,
    theme: Res<MdTheme>,
    mut commands: Commands,
) {
    for (group, pick) in &picks {
        let (code, added) = pick.pick.compose(pick.mask);
        for entity in children.iter_descendants(group) {
            if texts.contains(entity) {
                commands.entity(entity).despawn_children();
                for span in code_spans(&code, pick.lang.as_deref(), &theme) {
                    commands.spawn_scene(span).insert(ChildOf(entity));
                }
            }
            if let Ok(mut highlight) = highlights.get_mut(entity) {
                highlight.set_if_neq(CodeHighlight(added.clone()));
            }
            if let Ok(mut source) = sources.get_mut(entity) {
                source.0.clone_from(&code);
            }
            if let Ok(chip) = chips.get(entity) {
                let on = pick.mask >> chip.0 & 1 == 1;
                if let Ok(mut background) = backgrounds.get_mut(entity) {
                    background.0 = if on { theme.highlight } else { Color::NONE };
                }
                if let Ok(mut border) = borders.get_mut(entity) {
                    *border = BorderColor::all(if on { theme.link } else { theme.border });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK: &str = "fn main() {\n    //@ anchor body\n}\n\n//@ anchor items\n\n//@ option a Say a\n//@ at body\nprintln!(\"a\");\nif true {\n    b();\n}\n//@ at items\nfn a() {}\n\n//@ option b Say b\n//@ at body\nprintln!(\"b\");\n";

    #[test]
    fn base_without_options() {
        let pick = Pick::parse(BLOCK);
        assert_eq!(pick.options.len(), 2);
        let (code, added) = pick.compose(0);
        assert_eq!(code, "fn main() {\n}\n\n");
        assert_eq!(added, Vec::<std::ops::Range<usize>>::new());
    }

    #[test]
    fn options_are_indented_and_marked() {
        let pick = Pick::parse(BLOCK);
        let (code, added) = pick.compose(0b11);
        assert!(code.contains(
            "    println!(\"a\");\n    if true {\n        b();\n    }\n    println!(\"b\");\n}"
        ));
        assert!(code.contains("fn a() {}"));
        assert_eq!(added[0], 1..5);
    }

    #[test]
    fn variants_cover_each_option() {
        let names: Vec<String> = Pick::parse(BLOCK)
            .variants()
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        assert_eq!(names, ["base", "a", "b", "all"]);
    }

    #[test]
    fn toggling_a_chip_rewrites_the_code() {
        use crate::{MdDocument, RenderOptions, markdown_scene};
        use bevy::{asset::AssetPlugin, scene::ScenePlugin};

        let note = format!("```rust pick\n{BLOCK}```\n");
        let doc = MdDocument::parse(&note);
        let theme = MdTheme::default();
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin))
            .insert_resource(theme.clone())
            .add_systems(Update, apply_picks);
        let options = RenderOptions {
            theme: &theme,
            resolve: &|_| true,
            title_badge: None,
            code_preview: None,
            icon: None,
            variant_picker: None,
        };
        app.world_mut()
            .spawn_scene(markdown_scene(&doc, &options))
            .unwrap();
        app.update();

        let code = |app: &mut App| -> String {
            let world = app.world_mut();
            let root = world
                .query_filtered::<Entity, With<CodeText>>()
                .single(world)
                .unwrap();
            let children = world.get::<Children>(root).unwrap().to_vec();
            children
                .iter()
                .map(|&c| world.get::<TextSpan>(c).unwrap().0.clone())
                .collect()
        };
        assert!(!code(&mut app).contains("println"));

        let chip = app
            .world_mut()
            .query::<(Entity, &PickChip)>()
            .iter(app.world())
            .find(|(_, chip)| chip.0 == 0)
            .map(|(e, _)| e)
            .unwrap();
        app.world_mut().trigger(Activate { entity: chip });
        app.update();
        assert!(code(&mut app).contains("println!(\"a\")"));
        let highlight = app
            .world_mut()
            .query::<&CodeHighlight>()
            .single(app.world())
            .unwrap()
            .clone();
        assert_eq!(highlight.0, vec![1..5, 7..8]);
    }
}

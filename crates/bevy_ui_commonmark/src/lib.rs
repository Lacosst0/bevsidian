//! `CommonMark` rendered as Bevy UI.
//!
//! - [`MdDocument::parse`] parses `CommonMark` with GFM tables, task lists, strikethrough,
//!   footnotes and callouts, plus Obsidian `[[wikilinks]]` and `#tags`.
//! - [`markdown_scene`] turns a document into a BSN [`Scene`] of rich `Text`/`TextSpan`
//!   blocks that are selectable and copyable through [`bevy_ui_textselect`].
//! - Clicking a link, tag or footnote reference triggers [`MdLinkActivated`].
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_ui_commonmark::prelude::*;
//!
//! fn setup(mut commands: Commands, theme: Res<MdTheme>) {
//!     let doc = MdDocument::parse("# Hello\n\nSee [[Other note]] and #bevy.");
//!     let options = RenderOptions { theme: &theme, resolve: &|_| true };
//!     commands.spawn(Camera2d);
//!     commands.spawn_scene(markdown_scene(&doc, &options));
//! }
//!
//! fn on_link(link: On<MdLinkActivated>) {
//!     info!("clicked {:?}", link.target);
//! }
//! ```

use bevy::prelude::*;
use bevy_ui_textselect::{TextSelectPlugin, TextSpanClick};

pub mod document;
pub mod highlight;
pub mod scene;
pub mod theme;

pub use document::*;
pub use scene::{
    MdBlock, MdCopyButton, MdFootnote, MdHeading, MdLink, MdRoot, RenderOptions, markdown_scene,
};
pub use theme::MdTheme;

/// Common imports.
pub mod prelude {
    pub use crate::{
        CommonMarkPlugin, LinkTarget, MdDocument, MdHeading, MdLinkActivated, MdRoot, MdTheme,
        RenderOptions, WikiLink, markdown_scene,
    };
}

/// A scene that inserts a clone of `value`.
///
/// Useful in `bsn!` for components such as [`TextFont`] whose `FromTemplate` is a separate
/// template type, so a ready-made value cannot be inserted directly: `@{insert(font)}`.
pub fn insert<C: Component + Clone>(value: C) -> impl Scene {
    bevy::ecs::template::template(move |_: &mut bevy::ecs::template::TemplateContext| {
        Ok(value.clone())
    })
}

/// Registers the theme, link activation and (if missing) [`TextSelectPlugin`].
#[derive(Default)]
pub struct CommonMarkPlugin;

impl Plugin for CommonMarkPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TextSelectPlugin>() {
            app.add_plugins(TextSelectPlugin);
        }
        app.init_resource::<MdTheme>()
            .add_observer(activate_links)
            .add_systems(
                PostUpdate,
                // Visibility propagates before UI layout, so changes show a frame later; the
                // margin in `cull_offscreen_blocks` covers that.
                cull_offscreen_blocks.after(bevy::ui::UiSystems::Layout),
            );
    }
}

/// Hides top-level blocks that are scrolled more than half a screen out of view.
///
/// Scrolling moves every node of a document, so without this each frame re-extracts and
/// re-clips every glyph of the whole note. Hidden blocks keep their layout and text, so
/// scroll extents, anchors, selection and copy are unaffected.
fn cull_offscreen_blocks(
    roots: Query<(Entity, &Children), With<MdRoot>>,
    parents: Query<&ChildOf>,
    viewports: Query<(&Node, &ComputedNode, &UiGlobalTransform)>,
    mut blocks: Query<(&ComputedNode, &UiGlobalTransform, &mut Visibility), With<MdBlock>>,
) {
    for (root, children) in &roots {
        // The nearest vertically scrolling ancestor is the viewport.
        let Some((viewport, transform)) = parents.iter_ancestors(root).find_map(|e| {
            viewports
                .get(e)
                .ok()
                .filter(|(node, ..)| node.overflow.y == OverflowAxis::Scroll)
                .map(|(_, computed, transform)| (computed.size(), transform.translation))
        }) else {
            continue;
        };
        // Visible band, extended by half a viewport height on each side so blocks are already
        // shown when a fast fling reaches them.
        let (top, bottom) = (transform.y - viewport.y, transform.y + viewport.y);
        for child in children {
            let Ok((computed, transform, mut visibility)) = blocks.get_mut(*child) else {
                continue;
            };
            let half = computed.size().y / 2.0;
            let center = transform.translation.y;
            let visible = center + half >= top && center - half <= bottom;
            visibility.set_if_neq(if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

/// Triggered when a link, `#tag` or footnote reference in rendered markdown is clicked.
#[derive(Event, Clone, Debug)]
pub struct MdLinkActivated {
    /// What was clicked.
    pub target: LinkTarget,
    /// The span entity.
    pub span: Entity,
}

fn activate_links(click: On<TextSpanClick>, links: Query<&MdLink>, mut commands: Commands) {
    // Only react once, on the original target, not on every bubbling step.
    if click.entity != click.span {
        return;
    }
    if let Ok(link) = links.get(click.span) {
        commands.trigger(MdLinkActivated {
            target: link.0.clone(),
            span: click.span,
        });
    }
}

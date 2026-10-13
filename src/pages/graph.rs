//! The graph page: the rendered graph with its legend and zoom controls.

use bevy::prelude::*;

use bevsidian_graph::{GraphViewport, legend, zoom_controls};

pub fn graph_page() -> impl Scene {
    bsn! {
        Node { flex_grow: 1.0, flex_basis: px(0), overflow: Overflow::clip() }
        GraphViewport
        Children [ @{legend()} -- @{zoom_controls()} ]
    }
}

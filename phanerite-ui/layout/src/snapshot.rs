use taffy::{NodeId, Point, Size};

use crate::LayoutNodeKind;

/// An owned, renderer-independent result of one viewport's layout.
///
/// All coordinates are fractional logical pixels. Only nodes reachable from
/// the viewport root are included; `display: none` subtrees are omitted.
/// Nodes are in parent-before-child tree order, without CSS stacking semantics.
/// No component values, borrows, GPU resources or scale factors cross this boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutSnapshot {
    pub viewport: Size<f32>,
    pub nodes: Vec<NodeLayout>,
}

/// A border box in absolute viewport coordinates, before physical pixel snapping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeLayout {
    /// CPU-side identity for inspection; renderers need not upload it.
    pub node: NodeId,
    pub kind: LayoutNodeKind,
    pub position: Point<f32>,
    pub size: Size<f32>,
    pub depth: usize,
}

/// The three kinds of nodes understood by the box-layout tree.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LayoutNodeKind {
    /// A rectangular box. Only box nodes may have children in this tree.
    #[default]
    Box,
    /// A measured text-flow leaf in the box-layout tree, not one source `text`
    /// or `span` element. Rich-text runs, inline-box placement, measurement and
    /// rendering belong to a future text-layout integration.
    Text,
    /// A leaf whose content is supplied by an external renderer, such as an
    /// image, SVG or 3D scene. This discriminator does not allocate resources
    /// or determine whether the leaf participates in block or inline flow.
    Replaced,
}

/// Fixed node metadata stored in Taffy's node context.
///
/// There is no component marker here: presence in the component table identifies
/// component roots. Application-specific context fields remain inside the
/// concrete component types supplied by generated Rust code.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeMetadata {
    kind: LayoutNodeKind,
}

impl NodeMetadata {
    /// Creates metadata for a box, text or replaced node.
    #[must_use]
    pub const fn new(kind: LayoutNodeKind) -> Self {
        Self { kind }
    }

    /// Returns the node's kind.
    #[must_use]
    pub const fn kind(&self) -> LayoutNodeKind {
        self.kind
    }
}

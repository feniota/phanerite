/// The three kinds of actual UI nodes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// A rectangular container. Only div nodes may have children.
    #[default]
    Div,
    /// A text leaf. Text measurement and rendering are separate integrations.
    Text,
    /// An external visual leaf, such as an image, SVG or 3D scene.
    /// This discriminator does not allocate any GPU resources.
    Glyph,
}

/// Fixed node metadata stored in Taffy's node context.
///
/// There is no component marker here: presence in the component table identifies
/// component roots. Application-specific context fields remain inside the
/// concrete component types supplied by generated Rust code.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeMetadata {
    kind: NodeKind,
}

impl NodeMetadata {
    /// Creates metadata for a div, text or glyph node.
    #[must_use]
    pub const fn new(kind: NodeKind) -> Self {
        Self { kind }
    }

    /// Returns the node's kind.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }
}

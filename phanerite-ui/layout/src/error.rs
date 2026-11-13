use std::fmt;

use taffy::{NodeId, TaffyError};

/// Errors from checked operations on the retained UI data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeError {
    /// The ID is absent or its generation is no longer current.
    InvalidNode(NodeId),
    /// The permanent viewport root cannot be detached, reparented or removed.
    CannotMoveOrRemoveRoot(NodeId),
    /// Text and glyph nodes cannot have children.
    NotContainer(NodeId),
    /// A child was listed more than once for the same parent.
    DuplicateChild(NodeId),
    /// The new relationship would make a node its own ancestor.
    Cycle { parent: NodeId, child: NodeId },
    /// Components must be associated with a div root.
    ComponentRootMustBeDiv(NodeId),
    /// Logical viewport dimensions must be finite and nonnegative.
    InvalidViewportSize,
    /// An operation delegated to Taffy failed.
    Taffy(TaffyError),
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNode(node) => write!(f, "invalid UI node {node:?}"),
            Self::CannotMoveOrRemoveRoot(node) => {
                write!(f, "cannot move or remove the viewport root {node:?}")
            }
            Self::NotContainer(node) => write!(f, "node {node:?} cannot have children"),
            Self::DuplicateChild(node) => write!(f, "duplicate child {node:?}"),
            Self::Cycle { parent, child } => {
                write!(f, "attaching {child:?} to {parent:?} would create a cycle")
            }
            Self::ComponentRootMustBeDiv(node) => {
                write!(f, "component root {node:?} must be a div")
            }
            Self::InvalidViewportSize => {
                write!(f, "viewport dimensions must be finite and nonnegative")
            }
            Self::Taffy(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for TreeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Taffy(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TaffyError> for TreeError {
    fn from(error: TaffyError) -> Self {
        Self::Taffy(error)
    }
}

/// Result of a retained-tree operation.
pub type TreeResult<T> = Result<T, TreeError>;

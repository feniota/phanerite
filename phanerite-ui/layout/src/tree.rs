use std::collections::HashSet;

use slotmap::{DefaultKey, SparseSecondaryMap};
use taffy::{Dimension, NodeId, Size, Style, TaffyTree};

use crate::{Component, NodeKind, NodeMetadata, TreeError, TreeResult};

/// An empty component set for a tree with no component instances.
#[derive(Debug)]
pub enum NoComponents {}

/// Values removed by a storage-level subtree deletion.
///
/// Nodes and component entries are ordered descendants before ancestors.
/// These owned component values let the caller inspect or release its data;
/// this operation does not run frontend lifecycle callbacks. Callbacks that
/// require live nodes must eventually run before the storage deletion.
#[derive(Debug)]
#[must_use = "removed component values may need application cleanup"]
pub struct RemovedSubtree<C: Component> {
    pub nodes: Vec<NodeId>,
    pub components: Vec<(NodeId, C)>,
}

/// A retained UI tree for one logical viewport, without an OS window.
///
/// `C` is an application-generated closed enum of concrete component instances
/// implementing [`crate::Component`]. Exported contexts retain their actual
/// Rust types and need neither trait objects nor `Any`. Gabbro can generate an
/// ordinary enum and its lifecycle dispatch later.
///
/// Taffy's opaque IDs are local to this tree. Do not mix IDs from different
/// windows, interpret their integer encoding, or persist them as application
/// identities. Values stay at the same ID when updated or reparented.
#[derive(Debug)]
pub struct Window<C: Component = NoComponents> {
    tree: TaffyTree<NodeMetadata>,
    root_node: NodeId,
    components: SparseSecondaryMap<DefaultKey, C>,
}

impl<C: Component> Window<C> {
    /// Creates a permanent div root sized in logical pixels.
    ///
    /// Layout rounding is disabled; conversion and snapping to physical pixels
    /// belong at the future rendering boundary.
    pub fn new(width: f32, height: f32) -> TreeResult<Self> {
        Self::validate_size(width, height)?;
        let mut tree = TaffyTree::new();
        tree.disable_rounding();
        let style = Style {
            size: Size {
                width: Dimension::length(width),
                height: Dimension::length(height),
            },
            ..Style::default()
        };
        let root_node = tree.new_leaf_with_context(style, NodeMetadata::default())?;
        Ok(Self {
            tree,
            root_node,
            components: SparseSecondaryMap::new(),
        })
    }

    /// The permanent viewport root. This ID remains stable across resizes.
    #[must_use]
    pub fn root_node(&self) -> NodeId {
        self.root_node
    }

    /// Read-only access to Taffy's topology, styles and cached layout data.
    ///
    /// Mutable access is intentionally withheld so removals cannot bypass the
    /// component table and tree invariants.
    #[must_use]
    pub fn tree(&self) -> &TaffyTree<NodeMetadata> {
        &self.tree
    }

    /// Checks presence, including the generation, without indexing stale IDs.
    #[must_use]
    pub fn contains_node(&self, node: NodeId) -> bool {
        self.tree.get_node_context(node).is_some()
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.tree.total_node_count()
    }

    pub fn metadata(&self, node: NodeId) -> TreeResult<&NodeMetadata> {
        self.tree
            .get_node_context(node)
            .ok_or(TreeError::InvalidNode(node))
    }

    pub fn parent(&self, node: NodeId) -> TreeResult<Option<NodeId>> {
        self.require_node(node)?;
        Ok(self.tree.parent(node))
    }

    pub fn children(&self, node: NodeId) -> TreeResult<Vec<NodeId>> {
        self.require_node(node)?;
        Ok(self.tree.children(node)?)
    }

    /// Allocates a node, optionally attaching it after its parent's last child.
    /// A detached node remains owned by this container until explicitly removed.
    pub fn create_node(
        &mut self,
        kind: NodeKind,
        style: Style,
        parent: Option<NodeId>,
    ) -> TreeResult<NodeId> {
        if let Some(parent) = parent {
            self.require_container(parent)?;
        }
        let node = self
            .tree
            .new_leaf_with_context(style, NodeMetadata::new(kind))?;
        if let Some(parent) = parent {
            self.tree.add_child(parent, node)?;
        }
        Ok(node)
    }

    /// Moves a node to the end of a div's children, retaining its identity and
    /// component data. Appending an existing child moves it rather than duplicating it.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) -> TreeResult<()> {
        self.require_container(parent)?;
        self.validate_child(parent, child)?;
        if let Some(previous) = self.tree.parent(child) {
            self.tree.remove_child(previous, child)?;
        }
        self.tree.add_child(parent, child)?;
        Ok(())
    }

    /// Replaces a div's ordered child list. Excluded children are detached, not
    /// destroyed. Included children are detached from any previous parent.
    /// All relationships are validated before any mutation takes place.
    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) -> TreeResult<()> {
        self.require_container(parent)?;
        let mut seen = HashSet::with_capacity(children.len());
        for &child in children {
            self.validate_child(parent, child)?;
            if !seen.insert(child) {
                return Err(TreeError::DuplicateChild(child));
            }
        }
        self.tree.set_children(parent, children)?;
        Ok(())
    }

    /// Detaches a node without removing it or its associated component value.
    /// Returns its previous parent, if it had one.
    pub fn detach_node(&mut self, node: NodeId) -> TreeResult<Option<NodeId>> {
        self.require_node(node)?;
        self.require_non_root(node)?;
        let parent = self.tree.parent(node);
        if let Some(parent) = parent {
            self.tree.remove_child(parent, node)?;
        }
        Ok(parent)
    }

    /// Changes layout input without changing the ID. Taffy invalidates caches
    /// on the node and its ancestors; no component update or redraw is scheduled.
    pub fn set_style(&mut self, node: NodeId, style: Style) -> TreeResult<()> {
        self.require_node(node)?;
        self.tree.set_style(node, style)?;
        Ok(())
    }

    pub fn resize(&mut self, width: f32, height: f32) -> TreeResult<()> {
        Self::validate_size(width, height)?;
        let mut style = self.tree.style(self.root_node)?.clone();
        style.size = Size {
            width: Dimension::length(width),
            height: Dimension::length(height),
        };
        self.set_style(self.root_node, style)
    }

    /// Associates a concrete component enum value with a live div.
    /// Returns the previous value, if any; no lifecycle callbacks are invoked.
    pub fn insert_component(&mut self, root: NodeId, component: C) -> TreeResult<Option<C>> {
        if self.metadata(root)?.kind() != NodeKind::Div {
            return Err(TreeError::ComponentRootMustBeDiv(root));
        }
        Ok(self.components.insert(root.into(), component))
    }

    /// Presence in this table identifies component roots; no metadata flag is needed.
    #[must_use]
    pub fn component(&self, root: NodeId) -> Option<&C> {
        self.components.get(root.into())
    }

    pub fn component_mut(&mut self, root: NodeId) -> Option<&mut C> {
        self.components.get_mut(root.into())
    }

    /// Removes the component value, retaining the node and its descendants.
    pub fn remove_component(&mut self, root: NodeId) -> Option<C> {
        self.components.remove(root.into())
    }

    pub fn components(&self) -> impl Iterator<Item = (NodeId, &C)> {
        self.components
            .iter()
            .map(|(key, value)| (key.into(), value))
    }

    #[must_use]
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Removes a subtree in post-order, synchronously cleaning the external
    /// component table. Secondary maps do not observe Taffy removals themselves.
    /// This only changes storage; it does not run frontend unmount callbacks.
    pub fn destroy_subtree(&mut self, root: NodeId) -> TreeResult<RemovedSubtree<C>> {
        self.require_node(root)?;
        self.require_non_root(root)?;
        let mut nodes = Vec::new();
        let mut stack = vec![(root, false)];
        while let Some((node, visited)) = stack.pop() {
            if visited {
                nodes.push(node);
                continue;
            }
            stack.push((node, true));
            for child in self.tree.children(node)?.into_iter().rev() {
                stack.push((child, false));
            }
        }
        let mut components = Vec::new();
        for &node in &nodes {
            if let Some(component) = self.components.remove(node.into()) {
                components.push((node, component));
            }
            self.tree.remove(node)?;
        }
        Ok(RemovedSubtree { nodes, components })
    }

    /// Check if a node exists in the tree and is valid
    fn require_node(&self, node: NodeId) -> TreeResult<()> {
        self.metadata(node).map(|_| ())
    }

    fn require_container(&self, node: NodeId) -> TreeResult<()> {
        if self.metadata(node)?.kind() == NodeKind::Div {
            Ok(())
        } else {
            Err(TreeError::NotContainer(node))
        }
    }

    fn require_non_root(&self, node: NodeId) -> TreeResult<()> {
        if node == self.root_node {
            Err(TreeError::CannotMoveOrRemoveRoot(node))
        } else {
            Ok(())
        }
    }

    fn validate_child(&self, parent: NodeId, child: NodeId) -> TreeResult<()> {
        self.require_node(child)?;
        self.require_non_root(child)?;
        let mut ancestor = Some(parent);
        while let Some(node) = ancestor {
            if node == child {
                return Err(TreeError::Cycle { parent, child });
            }
            ancestor = self.tree.parent(node);
        }
        Ok(())
    }

    fn validate_size(width: f32, height: f32) -> TreeResult<()> {
        if width.is_finite() && height.is_finite() && width >= 0.0 && height >= 0.0 {
            Ok(())
        } else {
            Err(TreeError::InvalidViewportSize)
        }
    }
}

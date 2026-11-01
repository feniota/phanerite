use phanerite_layout::{Component, LayoutNodeKind, LayoutTree, NodeId, TreeError, taffy::Style};

fn box_node<C: Component>(layout: &mut LayoutTree<C>, parent: NodeId) -> NodeId {
    layout
        .create_node(LayoutNodeKind::Box, Style::default(), Some(parent))
        .unwrap()
}

#[derive(Debug, PartialEq, Eq)]
struct CounterContext {
    count: u32,
}

#[derive(Debug, PartialEq, Eq)]
enum Components {
    Counter(CounterContext),
    Label(String),
}

impl Component for Components {}

#[test]
fn logical_sizes_remain_fractional_and_resize_keeps_identity() {
    let mut layout = LayoutTree::<Components>::new(800.25, 600.75).unwrap();
    let root = layout.root_node();
    let style = layout.tree().style(root).unwrap();
    assert_eq!(
        style.size.width,
        phanerite_layout::taffy::Dimension::length(800.25)
    );
    assert_eq!(
        style.size.height,
        phanerite_layout::taffy::Dimension::length(600.75)
    );
    layout.resize(900.5, 700.125).unwrap();
    assert_eq!(layout.root_node(), root);
    assert_eq!(
        layout.tree().style(root).unwrap().size.height,
        phanerite_layout::taffy::Dimension::length(700.125)
    );
    assert_eq!(layout.node_count(), 1);
}

#[test]
fn component_values_are_concrete_and_updates_keep_node_identity() {
    let mut layout = LayoutTree::<Components>::new(800.0, 600.0).unwrap();
    let parent = layout.root_node();
    let root = box_node(&mut layout, parent);
    layout
        .insert_component(root, Components::Counter(CounterContext { count: 0 }))
        .unwrap();
    let Some(Components::Counter(counter)) = layout.component_mut(root) else {
        panic!("wrong component type")
    };
    counter.count += 1;
    layout.set_style(root, Style::default()).unwrap();
    assert_eq!(
        layout.component(root),
        Some(&Components::Counter(CounterContext { count: 1 }))
    );
    assert_eq!(
        layout
            .components()
            .map(|(node, _)| node)
            .collect::<Vec<_>>(),
        vec![root]
    );
    assert_eq!(layout.parent(root).unwrap(), Some(parent));
}

#[test]
fn moving_and_detaching_nodes_retains_subtrees_and_component_values() {
    let mut layout = LayoutTree::<Components>::new(800.0, 600.0).unwrap();
    let root = layout.root_node();
    let left = box_node(&mut layout, root);
    let right = box_node(&mut layout, root);
    let child = box_node(&mut layout, left);
    let grandchild = box_node(&mut layout, child);
    layout
        .insert_component(child, Components::Label("child".into()))
        .unwrap();
    layout.append_child(right, child).unwrap();
    layout.append_child(right, child).unwrap();
    assert!(layout.children(left).unwrap().is_empty());
    assert_eq!(layout.children(right).unwrap(), vec![child]);
    assert_eq!(layout.children(child).unwrap(), vec![grandchild]);
    assert_eq!(layout.parent(child).unwrap(), Some(right));
    assert_eq!(layout.detach_node(child).unwrap(), Some(right));
    assert_eq!(layout.parent(child).unwrap(), None);
    assert!(layout.component(child).is_some());
    assert_eq!(layout.node_count(), 5);
}

#[test]
fn set_children_reorders_reparents_and_only_detaches_excluded_nodes() {
    let mut layout = LayoutTree::<Components>::new(800.0, 600.0).unwrap();
    let root = layout.root_node();
    let left = box_node(&mut layout, root);
    let right = box_node(&mut layout, root);
    let a = box_node(&mut layout, left);
    let b = box_node(&mut layout, left);
    let c = box_node(&mut layout, right);
    layout.set_children(left, &[c, a]).unwrap();
    assert_eq!(layout.children(left).unwrap(), vec![c, a]);
    assert!(layout.children(right).unwrap().is_empty());
    assert_eq!(layout.parent(c).unwrap(), Some(left));
    assert_eq!(layout.parent(b).unwrap(), None);
    assert!(layout.contains_node(b));
    layout.set_children(left, &[a, c]).unwrap();
    assert_eq!(layout.children(left).unwrap(), vec![a, c]);
}

#[test]
fn invalid_relationships_do_not_partially_mutate_the_tree() {
    let mut layout = LayoutTree::<Components>::new(800.0, 600.0).unwrap();
    let root = layout.root_node();
    let parent = box_node(&mut layout, root);
    let child = box_node(&mut layout, parent);
    let sibling = box_node(&mut layout, root);
    assert_eq!(
        layout.set_children(parent, &[sibling, child, child]),
        Err(TreeError::DuplicateChild(child))
    );
    assert_eq!(layout.children(root).unwrap(), vec![parent, sibling]);
    assert_eq!(layout.children(parent).unwrap(), vec![child]);
    assert_eq!(
        layout.append_child(child, parent),
        Err(TreeError::Cycle {
            parent: child,
            child: parent
        })
    );
    assert_eq!(
        layout.append_child(parent, parent),
        Err(TreeError::Cycle {
            parent,
            child: parent
        })
    );
    assert_eq!(layout.parent(parent).unwrap(), Some(root));
}

#[test]
fn text_and_replaced_are_leaves_and_cannot_be_component_roots() {
    let mut layout = LayoutTree::<Components>::new(800.0, 600.0).unwrap();
    let root = layout.root_node();
    let child = box_node(&mut layout, root);
    for kind in [LayoutNodeKind::Text, LayoutNodeKind::Replaced] {
        let leaf = layout
            .create_node(kind, Style::default(), Some(root))
            .unwrap();
        let count = layout.node_count();
        assert_eq!(
            layout.create_node(LayoutNodeKind::Box, Style::default(), Some(leaf)),
            Err(TreeError::NotContainer(leaf))
        );
        assert_eq!(layout.node_count(), count);
        assert_eq!(
            layout.append_child(leaf, child),
            Err(TreeError::NotContainer(leaf))
        );
        assert_eq!(
            layout.insert_component(leaf, Components::Label("invalid".into())),
            Err(TreeError::ComponentRootMustBeBox(leaf))
        );
    }
    assert_eq!(layout.parent(child).unwrap(), Some(root));
    assert_eq!(layout.component_count(), 0);
}

#[test]
fn subtree_removal_returns_components_in_post_order_and_cleans_the_table() {
    let mut layout = LayoutTree::<Components>::new(800.0, 600.0).unwrap();
    let root = layout.root_node();
    let parent = box_node(&mut layout, root);
    let child = box_node(&mut layout, parent);
    let text = layout
        .create_node(LayoutNodeKind::Text, Style::default(), Some(child))
        .unwrap();
    let sibling = box_node(&mut layout, root);
    layout
        .insert_component(parent, Components::Counter(CounterContext { count: 7 }))
        .unwrap();
    layout
        .insert_component(child, Components::Label("child".into()))
        .unwrap();
    layout
        .insert_component(sibling, Components::Label("sibling".into()))
        .unwrap();
    let removed = layout.destroy_subtree(parent).unwrap();
    assert_eq!(removed.nodes, vec![text, child, parent]);
    assert_eq!(
        removed.components,
        vec![
            (child, Components::Label("child".into())),
            (parent, Components::Counter(CounterContext { count: 7 }))
        ]
    );
    assert_eq!(layout.children(root).unwrap(), vec![sibling]);
    assert_eq!(layout.component_count(), 1);
    assert!(!layout.contains_node(parent));
    assert!(layout.component(parent).is_none());
    assert!(layout.component(child).is_none());
}

#[test]
fn stale_keys_are_rejected_even_after_a_slot_is_reused() {
    let mut layout = LayoutTree::<Components>::new(800.0, 600.0).unwrap();
    let root = layout.root_node();
    let old = box_node(&mut layout, root);
    layout
        .insert_component(old, Components::Label("old".into()))
        .unwrap();
    drop(layout.destroy_subtree(old).unwrap());
    let replacement = box_node(&mut layout, root);
    assert_ne!(old, replacement);
    assert!(!layout.contains_node(old));
    assert!(layout.component(old).is_none());
    assert_eq!(layout.parent(old), Err(TreeError::InvalidNode(old)));
    assert_eq!(layout.children(old), Err(TreeError::InvalidNode(old)));
    assert_eq!(
        layout.set_style(old, Style::default()),
        Err(TreeError::InvalidNode(old))
    );
    assert_eq!(
        layout.create_node(LayoutNodeKind::Box, Style::default(), Some(old)),
        Err(TreeError::InvalidNode(old))
    );
    assert_eq!(
        layout.set_children(root, &[replacement, old]),
        Err(TreeError::InvalidNode(old))
    );
    assert!(
        matches!(layout.destroy_subtree(old), Err(TreeError::InvalidNode(node)) if node == old)
    );
    assert_eq!(layout.children(root).unwrap(), vec![replacement]);
    assert_eq!(
        layout.insert_component(old, Components::Label("stale".into())),
        Err(TreeError::InvalidNode(old))
    );
    assert_eq!(layout.component_count(), 0);
}

#[test]
fn viewport_root_is_permanent_and_dimensions_are_checked() {
    let mut layout = LayoutTree::<Components>::new(0.0, 0.0).unwrap();
    let root = layout.root_node();
    let child = box_node(&mut layout, root);
    assert_eq!(
        layout.append_child(child, root),
        Err(TreeError::CannotMoveOrRemoveRoot(root))
    );
    assert_eq!(
        layout.detach_node(root),
        Err(TreeError::CannotMoveOrRemoveRoot(root))
    );
    assert!(
        matches!(layout.destroy_subtree(root), Err(TreeError::CannotMoveOrRemoveRoot(node)) if node == root)
    );
    for size in [-1.0, f32::NAN, f32::INFINITY] {
        assert!(matches!(
            LayoutTree::<Components>::new(size, 600.0),
            Err(TreeError::InvalidViewportSize)
        ));
        assert_eq!(
            layout.resize(800.0, size),
            Err(TreeError::InvalidViewportSize)
        );
    }
    assert_eq!(layout.root_node(), root);
}

#[test]
fn replacing_and_removing_component_data_does_not_delete_nodes() {
    let mut layout = LayoutTree::<Components>::new(800.0, 600.0).unwrap();
    let root = layout.root_node();
    assert_eq!(
        layout
            .insert_component(root, Components::Label("first".into()))
            .unwrap(),
        None
    );
    assert_eq!(
        layout
            .insert_component(root, Components::Label("second".into()))
            .unwrap(),
        Some(Components::Label("first".into()))
    );
    assert_eq!(
        layout.remove_component(root),
        Some(Components::Label("second".into()))
    );
    assert!(layout.contains_node(root));
    assert!(layout.component(root).is_none());
}

use phanerite_layout::{Component, NodeId, NodeKind, TreeError, Window, taffy::Style};

fn div<C: Component>(window: &mut Window<C>, parent: NodeId) -> NodeId {
    window
        .create_node(NodeKind::Div, Style::default(), Some(parent))
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
    let mut window = Window::<Components>::new(800.25, 600.75).unwrap();
    let root = window.root_node();
    let style = window.tree().style(root).unwrap();
    assert_eq!(
        style.size.width,
        phanerite_layout::taffy::Dimension::length(800.25)
    );
    assert_eq!(
        style.size.height,
        phanerite_layout::taffy::Dimension::length(600.75)
    );
    window.resize(900.5, 700.125).unwrap();
    assert_eq!(window.root_node(), root);
    assert_eq!(
        window.tree().style(root).unwrap().size.height,
        phanerite_layout::taffy::Dimension::length(700.125)
    );
    assert_eq!(window.node_count(), 1);
}

#[test]
fn component_values_are_concrete_and_updates_keep_node_identity() {
    let mut window = Window::<Components>::new(800.0, 600.0).unwrap();
    let parent = window.root_node();
    let root = div(&mut window, parent);
    window
        .insert_component(root, Components::Counter(CounterContext { count: 0 }))
        .unwrap();
    let Some(Components::Counter(counter)) = window.component_mut(root) else {
        panic!("wrong component type")
    };
    counter.count += 1;
    window.set_style(root, Style::default()).unwrap();
    assert_eq!(
        window.component(root),
        Some(&Components::Counter(CounterContext { count: 1 }))
    );
    assert_eq!(
        window
            .components()
            .map(|(node, _)| node)
            .collect::<Vec<_>>(),
        vec![root]
    );
    assert_eq!(window.parent(root).unwrap(), Some(parent));
}

#[test]
fn moving_and_detaching_nodes_retains_subtrees_and_component_values() {
    let mut window = Window::<Components>::new(800.0, 600.0).unwrap();
    let root = window.root_node();
    let left = div(&mut window, root);
    let right = div(&mut window, root);
    let child = div(&mut window, left);
    let grandchild = div(&mut window, child);
    window
        .insert_component(child, Components::Label("child".into()))
        .unwrap();
    window.append_child(right, child).unwrap();
    window.append_child(right, child).unwrap();
    assert!(window.children(left).unwrap().is_empty());
    assert_eq!(window.children(right).unwrap(), vec![child]);
    assert_eq!(window.children(child).unwrap(), vec![grandchild]);
    assert_eq!(window.parent(child).unwrap(), Some(right));
    assert_eq!(window.detach_node(child).unwrap(), Some(right));
    assert_eq!(window.parent(child).unwrap(), None);
    assert!(window.component(child).is_some());
    assert_eq!(window.node_count(), 5);
}

#[test]
fn set_children_reorders_reparents_and_only_detaches_excluded_nodes() {
    let mut window = Window::<Components>::new(800.0, 600.0).unwrap();
    let root = window.root_node();
    let left = div(&mut window, root);
    let right = div(&mut window, root);
    let a = div(&mut window, left);
    let b = div(&mut window, left);
    let c = div(&mut window, right);
    window.set_children(left, &[c, a]).unwrap();
    assert_eq!(window.children(left).unwrap(), vec![c, a]);
    assert!(window.children(right).unwrap().is_empty());
    assert_eq!(window.parent(c).unwrap(), Some(left));
    assert_eq!(window.parent(b).unwrap(), None);
    assert!(window.contains_node(b));
    window.set_children(left, &[a, c]).unwrap();
    assert_eq!(window.children(left).unwrap(), vec![a, c]);
}

#[test]
fn invalid_relationships_do_not_partially_mutate_the_tree() {
    let mut window = Window::<Components>::new(800.0, 600.0).unwrap();
    let root = window.root_node();
    let parent = div(&mut window, root);
    let child = div(&mut window, parent);
    let sibling = div(&mut window, root);
    assert_eq!(
        window.set_children(parent, &[sibling, child, child]),
        Err(TreeError::DuplicateChild(child))
    );
    assert_eq!(window.children(root).unwrap(), vec![parent, sibling]);
    assert_eq!(window.children(parent).unwrap(), vec![child]);
    assert_eq!(
        window.append_child(child, parent),
        Err(TreeError::Cycle {
            parent: child,
            child: parent
        })
    );
    assert_eq!(
        window.append_child(parent, parent),
        Err(TreeError::Cycle {
            parent,
            child: parent
        })
    );
    assert_eq!(window.parent(parent).unwrap(), Some(root));
}

#[test]
fn text_and_glyph_are_leaves_and_cannot_be_component_roots() {
    let mut window = Window::<Components>::new(800.0, 600.0).unwrap();
    let root = window.root_node();
    let child = div(&mut window, root);
    for kind in [NodeKind::Text, NodeKind::Glyph] {
        let leaf = window
            .create_node(kind, Style::default(), Some(root))
            .unwrap();
        let count = window.node_count();
        assert_eq!(
            window.create_node(NodeKind::Div, Style::default(), Some(leaf)),
            Err(TreeError::NotContainer(leaf))
        );
        assert_eq!(window.node_count(), count);
        assert_eq!(
            window.append_child(leaf, child),
            Err(TreeError::NotContainer(leaf))
        );
        assert_eq!(
            window.insert_component(leaf, Components::Label("invalid".into())),
            Err(TreeError::ComponentRootMustBeDiv(leaf))
        );
    }
    assert_eq!(window.parent(child).unwrap(), Some(root));
    assert_eq!(window.component_count(), 0);
}

#[test]
fn subtree_removal_returns_components_in_post_order_and_cleans_the_table() {
    let mut window = Window::<Components>::new(800.0, 600.0).unwrap();
    let root = window.root_node();
    let parent = div(&mut window, root);
    let child = div(&mut window, parent);
    let text = window
        .create_node(NodeKind::Text, Style::default(), Some(child))
        .unwrap();
    let sibling = div(&mut window, root);
    window
        .insert_component(parent, Components::Counter(CounterContext { count: 7 }))
        .unwrap();
    window
        .insert_component(child, Components::Label("child".into()))
        .unwrap();
    window
        .insert_component(sibling, Components::Label("sibling".into()))
        .unwrap();
    let removed = window.destroy_subtree(parent).unwrap();
    assert_eq!(removed.nodes, vec![text, child, parent]);
    assert_eq!(
        removed.components,
        vec![
            (child, Components::Label("child".into())),
            (parent, Components::Counter(CounterContext { count: 7 }))
        ]
    );
    assert_eq!(window.children(root).unwrap(), vec![sibling]);
    assert_eq!(window.component_count(), 1);
    assert!(!window.contains_node(parent));
    assert!(window.component(parent).is_none());
    assert!(window.component(child).is_none());
}

#[test]
fn stale_keys_are_rejected_even_after_a_slot_is_reused() {
    let mut window = Window::<Components>::new(800.0, 600.0).unwrap();
    let root = window.root_node();
    let old = div(&mut window, root);
    window
        .insert_component(old, Components::Label("old".into()))
        .unwrap();
    drop(window.destroy_subtree(old).unwrap());
    let replacement = div(&mut window, root);
    assert_ne!(old, replacement);
    assert!(!window.contains_node(old));
    assert!(window.component(old).is_none());
    assert_eq!(window.parent(old), Err(TreeError::InvalidNode(old)));
    assert_eq!(window.children(old), Err(TreeError::InvalidNode(old)));
    assert_eq!(
        window.set_style(old, Style::default()),
        Err(TreeError::InvalidNode(old))
    );
    assert_eq!(
        window.create_node(NodeKind::Div, Style::default(), Some(old)),
        Err(TreeError::InvalidNode(old))
    );
    assert_eq!(
        window.set_children(root, &[replacement, old]),
        Err(TreeError::InvalidNode(old))
    );
    assert!(
        matches!(window.destroy_subtree(old), Err(TreeError::InvalidNode(node)) if node == old)
    );
    assert_eq!(window.children(root).unwrap(), vec![replacement]);
    assert_eq!(
        window.insert_component(old, Components::Label("stale".into())),
        Err(TreeError::InvalidNode(old))
    );
    assert_eq!(window.component_count(), 0);
}

#[test]
fn viewport_root_is_permanent_and_dimensions_are_checked() {
    let mut window = Window::<Components>::new(0.0, 0.0).unwrap();
    let root = window.root_node();
    let child = div(&mut window, root);
    assert_eq!(
        window.append_child(child, root),
        Err(TreeError::CannotMoveOrRemoveRoot(root))
    );
    assert_eq!(
        window.detach_node(root),
        Err(TreeError::CannotMoveOrRemoveRoot(root))
    );
    assert!(
        matches!(window.destroy_subtree(root), Err(TreeError::CannotMoveOrRemoveRoot(node)) if node == root)
    );
    for size in [-1.0, f32::NAN, f32::INFINITY] {
        assert!(matches!(
            Window::<Components>::new(size, 600.0),
            Err(TreeError::InvalidViewportSize)
        ));
        assert_eq!(
            window.resize(800.0, size),
            Err(TreeError::InvalidViewportSize)
        );
    }
    assert_eq!(window.root_node(), root);
}

#[test]
fn replacing_and_removing_component_data_does_not_delete_nodes() {
    let mut window = Window::<Components>::new(800.0, 600.0).unwrap();
    let root = window.root_node();
    assert_eq!(
        window
            .insert_component(root, Components::Label("first".into()))
            .unwrap(),
        None
    );
    assert_eq!(
        window
            .insert_component(root, Components::Label("second".into()))
            .unwrap(),
        Some(Components::Label("first".into()))
    );
    assert_eq!(
        window.remove_component(root),
        Some(Components::Label("second".into()))
    );
    assert!(window.contains_node(root));
    assert!(window.component(root).is_none());
}

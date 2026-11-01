use phanerite_layout::{
    LayoutNodeKind, LayoutTree,
    taffy::{AlignItems, Dimension, Display, FlexDirection, Point, Rect, Size, Style},
};

#[test]
fn snapshots_accumulate_fractional_positions_and_are_independent_of_the_tree() {
    let mut tree: LayoutTree = LayoutTree::new(320.5, 200.25).unwrap();
    let root = tree.root_node();
    tree.set_style(
        root,
        Style {
            padding: Rect::length(10.25),
            align_items: AlignItems::START,
            ..Style::default()
        },
    )
    .unwrap();
    let parent = tree
        .create_node(
            LayoutNodeKind::Box,
            Style {
                size: Size {
                    width: Dimension::length(100.5),
                    height: Dimension::length(80.5),
                },
                padding: Rect::length(3.5),
                ..Style::default()
            },
            Some(root),
        )
        .unwrap();
    let child = tree
        .create_node(
            LayoutNodeKind::Box,
            Style {
                size: Size {
                    width: Dimension::length(20.25),
                    height: Dimension::length(12.75),
                },
                ..Style::default()
            },
            Some(parent),
        )
        .unwrap();
    assert!(tree.needs_layout().unwrap());
    let snapshot = tree.compute_layout().unwrap();
    assert!(!tree.needs_layout().unwrap());
    assert_eq!(
        snapshot.viewport,
        Size {
            width: 320.5,
            height: 200.25
        }
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.node)
            .collect::<Vec<_>>(),
        vec![root, parent, child]
    );
    assert_eq!(snapshot.nodes[1].position, Point { x: 10.25, y: 10.25 });
    assert_eq!(snapshot.nodes[2].position, Point { x: 13.75, y: 13.75 });
    assert_eq!(
        snapshot.nodes[2].size,
        Size {
            width: 20.25,
            height: 12.75
        }
    );
    assert_eq!(snapshot.nodes[2].depth, 2);
    drop(tree.destroy_subtree(parent).unwrap());
    assert_eq!(tree.compute_layout().unwrap().nodes.len(), 1);
    assert_eq!(snapshot.nodes.len(), 3);
    fn assert_send_sync<T: Send + Sync>(_: &T) {}
    assert_send_sync(&snapshot);
}

#[test]
fn resize_recomputes_percentages_without_rounding_logical_units() {
    let mut tree: LayoutTree = LayoutTree::new(301.5, 200.0).unwrap();
    let root = tree.root_node();
    tree.create_node(
        LayoutNodeKind::Box,
        Style {
            size: Size {
                width: Dimension::percent(0.5),
                height: Dimension::length(40.0),
            },
            ..Style::default()
        },
        Some(root),
    )
    .unwrap();
    assert_eq!(tree.compute_layout().unwrap().nodes[1].size.width, 150.75);
    tree.resize(401.25, 200.0).unwrap();
    assert!(tree.needs_layout().unwrap());
    let next = tree.compute_layout().unwrap();
    assert_eq!(next.nodes[0].size.width, 401.25);
    assert_eq!(next.nodes[1].size.width, 200.625);
    tree.resize(401.25, 200.0).unwrap();
    assert!(!tree.needs_layout().unwrap());
}

#[test]
fn detached_and_hidden_subtrees_are_not_published() {
    let mut tree: LayoutTree = LayoutTree::new(300.0, 200.0).unwrap();
    let root = tree.root_node();
    let hidden = tree
        .create_node(
            LayoutNodeKind::Box,
            Style {
                display: Display::None,
                ..Style::default()
            },
            Some(root),
        )
        .unwrap();
    let child = tree
        .create_node(LayoutNodeKind::Box, Style::default(), Some(hidden))
        .unwrap();
    tree.create_node(LayoutNodeKind::Box, Style::default(), None)
        .unwrap();
    assert_eq!(
        tree.compute_layout()
            .unwrap()
            .nodes
            .iter()
            .map(|node| node.node)
            .collect::<Vec<_>>(),
        vec![root]
    );
    tree.set_style(hidden, Style::default()).unwrap();
    assert_eq!(
        tree.compute_layout()
            .unwrap()
            .nodes
            .iter()
            .map(|node| node.node)
            .collect::<Vec<_>>(),
        vec![root, hidden, child]
    );
    tree.detach_node(hidden).unwrap();
    assert_eq!(tree.compute_layout().unwrap().nodes.len(), 1);
}

#[test]
fn content_measurement_is_no_op_but_explicit_leaf_sizes_still_apply() {
    let mut tree: LayoutTree = LayoutTree::new(300.0, 200.0).unwrap();
    let root = tree.root_node();
    tree.set_style(
        root,
        Style {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::START,
            ..Style::default()
        },
    )
    .unwrap();
    for kind in [LayoutNodeKind::Text, LayoutNodeKind::Replaced] {
        tree.create_node(kind, Style::default(), Some(root))
            .unwrap();
        tree.create_node(
            kind,
            Style {
                size: Size {
                    width: Dimension::length(90.25),
                    height: Dimension::length(20.5),
                },
                ..Style::default()
            },
            Some(root),
        )
        .unwrap();
    }
    let snapshot = tree.compute_layout().unwrap();
    for index in [1, 3] {
        assert_eq!(snapshot.nodes[index].size, Size::ZERO);
    }
    for index in [2, 4] {
        assert_eq!(
            snapshot.nodes[index].size,
            Size {
                width: 90.25,
                height: 20.5
            }
        );
    }
}

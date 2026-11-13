use slotmap::DefaultKey;
use taffy::prelude::*;
use taffy::{Baselines, LayoutOutput};

use crate::{Aspect, Context, NodeMetadata, World};

struct Pos(f32);
impl Aspect for Pos {}
struct Vel(f32);
impl Aspect for Vel {}
struct Tag(u32);
impl Aspect for Tag {}

fn world_with_two() -> (World, DefaultKey, DefaultKey) {
    let mut world = World::new();
    world.register_dense::<Pos>();
    world.register_dense::<Vel>();
    let a = world.spawn(Style::default(), NodeMetadata);
    let b = world.spawn(Style::default(), NodeMetadata);
    world.insert_aspect(a, Pos(1.0));
    world.insert_aspect(a, Vel(10.0));
    world.insert_aspect(b, Pos(2.0));
    world.insert_aspect(b, Vel(20.0));
    (world, a, b)
}

#[test]
fn system_iterates_with_context() {
    let (mut world, a, b) = world_with_two();
    let key = world.insert_system(|ctx: &mut Context, pos: &mut Pos, vel: &mut Vel| {
        assert!(ctx.metadata(ctx.node()).is_some());
        pos.0 += vel.0;
    });
    world.execute_system(key);
    assert_eq!(world.storage::<Pos>().unwrap().get(a).unwrap().0, 11.0);
    assert_eq!(world.storage::<Pos>().unwrap().get(b).unwrap().0, 22.0);
}

#[test]
fn context_node_matches_entity() {
    let (mut world, a, b) = world_with_two();
    let key = world.insert_system(move |ctx: &mut Context, pos: &mut Pos| {
        pos.0 = if ctx.node() == a { 100.0 } else { 200.0 };
    });
    world.execute_system(key);
    assert_eq!(world.storage::<Pos>().unwrap().get(a).unwrap().0, 100.0);
    assert_eq!(world.storage::<Pos>().unwrap().get(b).unwrap().0, 200.0);
}

#[test]
fn context_random_access() {
    let (mut world, a, b) = world_with_two();
    world.insert_aspect(b, Tag(7));
    let key = world.insert_system(move |ctx: &mut Context, pos: &mut Pos| {
        if ctx.node() == a {
            assert_eq!(ctx.get::<Vel>(b).unwrap().0, 20.0);
            *ctx.get_mut::<Tag>(b).unwrap() = Tag(8);
            pos.0 = ctx.get::<Tag>(b).unwrap().0 as f32;
        }
    });
    world.execute_system(key);
    assert_eq!(world.storage::<Pos>().unwrap().get(a).unwrap().0, 8.0);
}

#[test]
fn context_insert_remove() {
    let (mut world, a, b) = world_with_two();
    let key = world.insert_system(move |ctx: &mut Context, _pos: &mut Pos| {
        if ctx.node() == a {
            ctx.insert(b, Tag(5));
            assert_eq!(ctx.remove::<Tag>(b).unwrap().0, 5);
        }
    });
    world.execute_system(key);
    assert!(world.storage::<Tag>().unwrap().get(b).is_none());
}

#[test]
#[should_panic(expected = "part of the current system's query")]
fn context_query_conflict_panics() {
    let (mut world, _a, _b) = world_with_two();
    let key = world.insert_system(|ctx: &mut Context, _pos: &mut Pos| {
        let _ = ctx.get::<Pos>(ctx.node());
    });
    world.execute_system(key);
}

#[test]
fn despawn_removes_subtree() {
    let mut world = World::new();
    world.register_dense::<Pos>();
    let root = world.spawn(Style::default(), NodeMetadata);
    let child = world.spawn(Style::default(), NodeMetadata);
    let grand = world.spawn(Style::default(), NodeMetadata);
    world
        .taffy_tree_mut()
        .add_child(root.into(), child.into())
        .unwrap();
    world
        .taffy_tree_mut()
        .add_child(child.into(), grand.into())
        .unwrap();
    for k in [root, child, grand] {
        world.insert_aspect(k, Pos(0.0));
    }
    world.despawn(root);
    assert_eq!(world.taffy_tree().total_node_count(), 0);
    assert!(world.metadata(root).is_none());
    assert!(world.storage::<Pos>().unwrap().get(grand).is_none());
}

#[test]
fn commands_despawn_is_deferred() {
    let (mut world, a, b) = world_with_two();
    let key = world.insert_system(move |ctx: &mut Context, pos: &mut Pos| {
        if ctx.node() == a {
            let node = ctx.node();
            ctx.commands().despawn(node);
            assert!(ctx.metadata(node).is_some());
        }
        pos.0 += 1.0;
    });
    world.execute_system(key);
    assert!(world.storage::<Pos>().unwrap().get(a).is_none());
    assert!(world.metadata(a).is_none());
    assert_eq!(world.storage::<Pos>().unwrap().get(b).unwrap().0, 3.0);
    assert_eq!(world.taffy_tree().total_node_count(), 1);
}

#[test]
fn batch_inserts_bundle() {
    let mut world = World::new();
    world.register_dense::<Pos>();
    world.register_dense::<Vel>();
    let keys: Vec<_> = (0..3)
        .map(|_| world.spawn(Style::default(), NodeMetadata))
        .collect();
    {
        let mut batch = world.batch::<(Pos, Vel)>().unwrap();
        for (i, k) in keys.iter().enumerate() {
            batch.insert(*k, (Pos(i as f32), Vel(i as f32 * 10.0)));
        }
    }
    for (i, k) in keys.iter().enumerate() {
        assert_eq!(world.storage::<Pos>().unwrap().get(*k).unwrap().0, i as f32);
        assert_eq!(
            world.storage::<Vel>().unwrap().get(*k).unwrap().0,
            i as f32 * 10.0
        );
    }
}

#[test]
fn batch_requires_registered_storages() {
    let mut world = World::new();
    assert!(world.batch::<(Pos,)>().is_none());
}

#[test]
fn resources_roundtrip() {
    struct Scale(f32);
    let mut world = World::new();
    world.insert_resource(Scale(2.0));
    world.register_dense::<Pos>();
    let a = world.spawn(Style::default(), NodeMetadata);
    world.insert_aspect(a, Pos(3.0));
    let key = world.insert_system(|ctx: &mut Context, pos: &mut Pos| {
        let scale = ctx.resource::<Scale>().0;
        pos.0 *= scale;
        ctx.resource_mut::<Scale>().0 = 5.0;
    });
    world.execute_system(key);
    assert_eq!(world.storage::<Pos>().unwrap().get(a).unwrap().0, 6.0);
    assert_eq!(world.resource::<Scale>().0, 5.0);
}

#[test]
fn split_allows_measure_closure_to_read_aspects() {
    let mut world = World::new();
    world.register_dense::<Pos>();
    world.register_dense::<Tag>();
    let root = world.spawn(
        Style {
            size: Size {
                width: length(100.0),
                height: length(50.0),
            },
            ..Style::default()
        },
        NodeMetadata,
    );
    let leaf = world.spawn(Style::default(), NodeMetadata);
    world
        .taffy_tree_mut()
        .add_child(root.into(), leaf.into())
        .unwrap();
    world.insert_aspect(root, Pos(0.0));
    world.insert_aspect(leaf, Tag(42));
    let key = world.insert_system(move |ctx: &mut Context, _pos: &mut Pos| {
        let (taffy, shared) = ctx.split();
        let tags = shared.storage::<Tag>().unwrap();
        taffy
            .compute_layout_with_measure(
                root.into(),
                Size {
                    width: AvailableSpace::Definite(100.0),
                    height: AvailableSpace::Definite(50.0),
                },
                |_input, node, _node_ctx, _style| {
                    let tag = tags.get(node.into()).unwrap();
                    LayoutOutput::from_sizes_and_baselines(
                        Size {
                            width: tag.0 as f32,
                            height: 10.0,
                        },
                        Rect::ZERO,
                        Baselines {
                            first: None,
                            last: None,
                        },
                    )
                },
            )
            .unwrap();
    });
    world.execute_system(key);
    assert_eq!(
        world.taffy_tree().layout(leaf.into()).unwrap().size.width,
        42.0
    );
}

#[test]
fn derive_aspect() {
    #[derive(Aspect)]
    struct Derived(u32);

    #[derive(Aspect)]
    struct Generic<T>(T);

    let mut world = World::new();
    world.register_dense::<Derived>();
    world.register_dense::<Generic<u32>>();
    let k = world.spawn(Style::default(), NodeMetadata);
    world.insert_aspect(k, Derived(1));
    world.insert_aspect(k, Generic(2u32));
    assert_eq!(world.storage::<Derived>().unwrap().get(k).unwrap().0, 1);
    assert_eq!(
        world.storage::<Generic<u32>>().unwrap().get(k).unwrap().0,
        2
    );
}

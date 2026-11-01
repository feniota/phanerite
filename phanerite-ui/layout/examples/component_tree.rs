//! Plain Rust equivalent of the component data a Gabbro entry point can generate.

use phanerite_layout::{Component, LayoutNodeKind, LayoutTree, NodeId, TreeResult, taffy::Style};

#[derive(Debug)]
struct CounterContext {
    count: u16,
}

#[derive(Debug)]
struct CounterInstance {
    context: CounterContext,
    label: NodeId,
}

#[derive(Debug)]
struct SettingsInstance {
    // Static children and dynamic lists can retain roots into the same table.
    counter: NodeId,
    rows: Vec<NodeId>,
}

// Generated for the reachable Gabbro component types; payloads stay concrete.
#[derive(Debug)]
enum Components {
    Counter(CounterInstance),
    Settings(SettingsInstance),
}

impl Component for Components {}

fn main() -> TreeResult<()> {
    let mut ui = LayoutTree::<Components>::new(800.0, 600.0)?;
    let root = ui.root_node();
    let counter = ui.create_node(LayoutNodeKind::Box, Style::default(), Some(root))?;
    let label = ui.create_node(LayoutNodeKind::Text, Style::default(), Some(counter))?;
    ui.insert_component(
        counter,
        Components::Counter(CounterInstance {
            context: CounterContext { count: 0 },
            label,
        }),
    )?;
    ui.insert_component(
        root,
        Components::Settings(SettingsInstance {
            counter,
            rows: Vec::new(),
        }),
    )?;

    // Generated field access uses an enum match, without Any or a vtable.
    if let Some(Components::Counter(instance)) = ui.component_mut(counter) {
        instance.context.count += 1;
        assert_eq!(instance.label, label);
    }
    if let Some(Components::Settings(instance)) = ui.component(root) {
        assert_eq!(instance.counter, counter);
        assert!(instance.rows.is_empty());
    }

    let removed = ui.destroy_subtree(counter)?;
    assert_eq!(removed.nodes, vec![label, counter]);
    assert_eq!(removed.components.len(), 1);
    assert!(ui.component(counter).is_none());
    Ok(())
}

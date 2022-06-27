use anyhow::{Context, Result};

use hwloc2::{topology, Object, Topology};

fn main() -> Result<()> {
    let topo = Topology::builder()
        .with_context(|| "failed to create the TopologyBuilder")?
        .all_types_filter(topology::Filter::KeepAll)?
        .io_types_filter(topology::Filter::KeepAll)?
        .build()
        .with_context(|| "failed to build the Topology")?;
    print_children_attrs(
        topo.root_object()
            .expect("failed to get topology's root object"),
        0,
    );
    Ok(())
}

fn print_children_attrs(obj: Object, depth: usize) {
    let padding = " ".repeat(4 * depth);
    eprintln!(
        "\n\n{}{} ({}): #{}(L#{})\n{}└-attributes: {:?}",
        padding,
        obj,
        obj.object_type(),
        obj.os_index(),
        obj.logical_index(),
        padding,
        obj.attributes(),
    );

    if obj.memory_arity() > 0 {
        let mem_child = obj
            .memory_first_child()
            .expect("failed to retrieve first memory child");
        eprintln!(
            "{}└-{} ({}): #{}(L#{}) ({} children)\n{}  └-attributes: {:?}",
            padding,
            mem_child,
            mem_child.object_type(),
            mem_child.os_index(),
            mem_child.logical_index(),
            mem_child.arity(),
            padding,
            mem_child.attributes(),
        );
    }
    for i in 0..obj.arity() {
        print_children_attrs(obj.children()[i as usize], depth + 1)
    }
}

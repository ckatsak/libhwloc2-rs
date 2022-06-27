pub mod bitmap;
mod error;
pub mod object;
pub mod topology;
pub mod types;

pub use error::Error;
pub use object::Object;
pub use topology::Topology;
pub use topology::TopologyBuilder;
pub use types::ObjectType;
pub use types::TypeDepth;

// Shamelessly stolen from:
//  https://internals.rust-lang.org/t/casting-constness-can-be-risky-heres-a-simple-fix/15933
//
// XXX: Remove when  https://github.com/rust-lang/rust/issues/92675  is stable.
pub(crate) fn ptr_mut_to_const<T>(ptr: *mut T) -> *const T {
    ptr as _
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::{Context, Result};

    use super::*;

    const ALL_OBJECT_TYPES: [ObjectType; 20] = [
        ObjectType::Machine,
        ObjectType::Package,
        ObjectType::Core,
        ObjectType::PU,
        ObjectType::L1Cache,
        ObjectType::L2Cache,
        ObjectType::L3Cache,
        ObjectType::L4Cache,
        ObjectType::L5Cache,
        ObjectType::L1ICache,
        ObjectType::L2ICache,
        ObjectType::L3ICache,
        ObjectType::Group,
        ObjectType::NumaNode,
        ObjectType::Bridge,
        ObjectType::PciDevice,
        ObjectType::OsDevice,
        ObjectType::Misc,
        ObjectType::MemCache,
        ObjectType::Die,
    ];

    fn all_filters(topo: &Topology) -> Result<HashMap<ObjectType, topology::Filter>> {
        let mut ret = HashMap::with_capacity(ALL_OBJECT_TYPES.len());
        for ot in ALL_OBJECT_TYPES {
            ret.insert(ot, topo.type_filter(ot)?);
        }
        Ok(ret)
    }

    fn print_topo(topo: &Topology) -> Result<()> {
        topo.abi_check()?;
        eprintln!("** {:?}", topo);
        eprintln!("** Flags: {:#?}", topo.flags());
        eprintln!("** Filters: {:?}", all_filters(topo)?);
        eprintln!();
        Ok(())
    }

    #[test]
    fn build_default_topology() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .build()
            .with_context(|| "failed to build the Topology")?;
        print_topo(&topo)
    }

    #[test]
    fn build_topology_flags() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .flags(topology::Flags::IS_THISSYSTEM | topology::Flags::RESTRICT_TO_CPUBINDING)
            .with_context(|| "failed to set IS_THISSYSTEM | RESTRICT_TO_CPUBINDING flags")?
            .build()
            .with_context(|| "failed to build the Topology")?;
        print_topo(&topo)
    }

    #[test]
    fn get_root_obj() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .flags(topology::Flags::IS_THISSYSTEM | topology::Flags::RESTRICT_TO_CPUBINDING)
            .with_context(|| "failed to set IS_THISSYSTEM | RESTRICT_TO_CPUBINDING flags")?
            .build()
            .with_context(|| "failed to build the Topology")?;

        let root_obj = topo.root_object().expect("failed retrieving root object!");
        //std::mem::drop(topo);
        // Dropping `topo` should fail to compile (unless I messed up `Object`'s lifetimes): CHECK

        eprintln!("** Root object: {:#?}", root_obj);
        eprintln!("  * type: {:#?}", root_obj.object_type());
        eprintln!("  * subtype: {:#?}", root_obj.subtype());
        eprintln!("  * symmetric subtree? {:#?}", root_obj.symmetric_subtree());
        eprintln!("  * arity: {:#?}", root_obj.arity());
        eprintln!("  * memory_arity: {:#?}", root_obj.memory_arity());

        Ok(())
    }

    #[test]
    fn root_and_children() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .build()
            .with_context(|| "failed to build the Topology")?;

        let root_obj = topo.root_object().expect("failed retrieving root object!");
        //std::mem::drop(topo);
        // Dropping `topo` should fail to compile (unless I messed up `Object`'s lifetimes): CHECK

        eprintln!("** Root object: {:#?}", root_obj);

        for (i, obj) in root_obj.children().iter().enumerate() {
            eprintln!("   ** child {}: {:#?}", i, obj);
        }

        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  Print topology tree walk
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    fn print_children(obj: Object, depth: usize) {
        let padding = " ".repeat(2 * depth);
        eprintln!(
            "{}{} ({}): #{}(L#{}) ({} mem children)",
            padding,
            obj,
            obj.object_type(),
            obj.os_index(),
            obj.logical_index(),
            obj.memory_arity(),
        );

        if obj.memory_arity() > 0 {
            let mem_child = obj
                .memory_first_child()
                .expect("failed to retrieve first memory child");
            eprintln!(
                "{}└-{} ({}): #{}(L#{}) ({} children)",
                padding,
                mem_child,
                mem_child.object_type(),
                mem_child.os_index(),
                mem_child.logical_index(),
                mem_child.arity(),
            );
        }
        for i in 0..obj.arity() {
            print_children(obj.children()[i as usize], depth + 1)
        }
    }

    #[test]
    fn print_tree() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .build()
            .with_context(|| "failed to build the Topology")?;
        print_children(
            topo.root_object()
                .expect("failed to get topology's root object"),
            0,
        );
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  Print Unions
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_union_access() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .build()
            .with_context(|| "failed to build the Topology")?;

        let root_obj = topo.root_object().expect("failed retrieving root object!");
        //std::mem::drop(topo);
        // Dropping `topo` should fail to compile (unless I messed up `Object`'s lifetimes): CHECK

        eprintln!("** Root object: {:#?}", root_obj);
        eprintln!("   ** attrs = {:#?}", root_obj.attr());
        if !root_obj.attr().is_null() {
            eprintln!("   ** attrs.numanode = {:#?}", unsafe {
                (*root_obj.attr()).numanode
            });
            eprintln!("   ** attrs.cache = {:#?}", unsafe {
                (*root_obj.attr()).cache
            });
            eprintln!("   ** attrs.group = {:#?}", unsafe {
                (*root_obj.attr()).group
            });
            eprintln!("   ** attrs.pcidev = {:#?}", unsafe {
                (*root_obj.attr()).pcidev
            });
            eprintln!("   ** attrs.osdev = {:#?}", unsafe {
                (*root_obj.attr()).osdev
            });
        }

        for (i, obj) in root_obj.children().iter().enumerate() {
            eprintln!("   ** child {}: {:#?}", i, obj);
            if !obj.attr().is_null() {
                eprintln!("   ** attrs.numanode = {:#?}", unsafe {
                    (*obj.attr()).numanode
                });
                eprintln!("   ** attrs.cache = {:#?}", unsafe { (*obj.attr()).cache });
                eprintln!("   ** attrs.group = {:#?}", unsafe { (*obj.attr()).group });
                eprintln!("   ** attrs.pcidev = {:#?}", unsafe {
                    (*obj.attr()).pcidev
                });
                eprintln!("   ** attrs.osdev = {:#?}", unsafe { (*obj.attr()).osdev });
            }
        }

        Ok(())
    }

    fn print_children_unions(obj: Object, depth: usize) {
        let padding = " ".repeat(2 * depth);
        eprintln!(
            "{}{} ({}): #{}(L#{}) ({} mem children)",
            padding,
            obj,
            obj.object_type(),
            obj.os_index(),
            obj.logical_index(),
            obj.memory_arity(),
        );
        if !obj.attr().is_null() {
            eprintln!(
                "{}{}.attrs.numanode = {:#?}",
                padding,
                obj.object_type(),
                unsafe { (*obj.attr()).numanode }
            );
            eprintln!(
                "{}{}.attrs.cache = {:#?}",
                padding,
                obj.object_type(),
                unsafe { (*obj.attr()).cache }
            );
            eprintln!(
                "{}{}.attrs.group = {:#?}",
                padding,
                obj.object_type(),
                unsafe { (*obj.attr()).group }
            );
            eprintln!(
                "{}{}.attrs.pcidev = {:#?}",
                padding,
                obj.object_type(),
                unsafe { (*obj.attr()).pcidev }
            );
            eprintln!(
                "{}{}.attrs.osdev = {:#?}",
                padding,
                obj.object_type(),
                unsafe { (*obj.attr()).osdev }
            );
        }

        if obj.memory_arity() > 0 {
            let mem_child = obj
                .memory_first_child()
                .expect("failed to retrieve first memory child");
            eprintln!(
                "{}└-{} ({}): #{}(L#{}) ({} children)",
                padding,
                mem_child,
                mem_child.object_type(),
                mem_child.os_index(),
                mem_child.logical_index(),
                mem_child.arity(),
            );
            if !mem_child.attr().is_null() {
                eprintln!(
                    "{}  └-{}.attrs.numanode = {:#?}",
                    padding,
                    mem_child.object_type(),
                    unsafe { (*mem_child.attr()).numanode }
                );
            }
        }
        for i in 0..obj.arity() {
            print_children_unions(obj.children()[i as usize], depth + 1)
        }
    }

    #[test]
    fn print_tree_unions() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .build()
            .with_context(|| "failed to build the Topology")?;
        print_children_unions(
            topo.root_object()
                .expect("failed to get topology's root object"),
            0,
        );
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  Print Attributes
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

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

    #[test]
    fn print_tree_attrs() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .build()
            .with_context(|| "failed to build the Topology")?;
        print_children_attrs(
            topo.root_object()
                .expect("failed to get topology's root object"),
            0,
        );
        Ok(())
    }

    // The following test fails to compile, as it should, because an `Object<'topo>` is attempted
    // to be used after `Topology` has been dropped.
    //#[test]
    //fn test_drop() -> Result<()> {
    //    let root = {
    //        let topo = Topology::builder()
    //            .with_context(|| "failed to create the TopologyBuilder")?
    //            .build()
    //            .with_context(|| "failed to build the Topology")?;
    //        topo.root_object()
    //            .expect("failed to get topology's root object")
    //    };
    //    eprintln!("root: {:?}", root);
    //    Ok(())
    //}

    #[test]
    fn test_get_next_obj_by_depth() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .build()
            .with_context(|| "failed to build the Topology")?;

        //let mut o = topo.next_object_by_depth(1, None);
        //while let Some(obj) = o {
        //    eprintln!("==> obj = {:?}", obj);
        //
        //    let next = topo.next_object_by_depth(1, Some(obj));
        //    eprintln!("    - obj.next = {:?}", next);
        //
        //    if let Some(next) = next {
        //        let prev = o.replace(next);
        //        eprintln!("    - obj.prev = {:?}", prev);
        //    }
        //}

        let mut o = None;
        while let Some(obj) = topo.next_object_by_depth(1, o) {
            eprintln!("==> obj = {:?}", obj);
            let _prev = o.replace(obj);
        }

        Ok(())
    }

    #[test]
    fn test_get_next_obj_by_type() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .build()
            .with_context(|| "failed to build the Topology")?;

        let mut o = None;
        while let Some(obj) = topo.next_object_by_type(ObjectType::NumaNode, o) {
            eprintln!("==> NumaNode = {:?}", obj);
            let _prev = o.replace(obj);
        }

        Ok(())
    }

    #[test]
    fn next_pci_1() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .io_types_filter(topology::Filter::KeepAll)?
            .build()
            .with_context(|| "failed to build the Topology")?;

        let mut o = None;
        while let Some(obj) = topo.next_object_by_type(ObjectType::PciDevice, o) {
            eprintln!("==> PciDevice = {:#?}", obj);
            let _prev = o.replace(obj);

            eprintln!(
                "\n{} ({}): #{}(L#{})\n\t└-attributes: {:?}\n",
                obj,
                obj.object_type(),
                obj.os_index(),
                obj.logical_index(),
                obj.attributes(),
            );
        }

        Ok(())
    }

    #[test]
    fn next_pci_2() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .io_types_filter(topology::Filter::KeepAll)?
            .build()
            .with_context(|| "failed to build the Topology")?;

        let mut o = None;
        while let Some(obj) = topo.next_pcidev(o) {
            eprintln!("==> PciDevice = {:#?}", obj);
            let _prev = o.replace(obj);

            eprintln!(
                "\n{} ({}): #{}(L#{})\n\t└-attributes: {:?}\n\t└-non-io ancestor: {:?}\n",
                obj,
                obj.object_type(),
                obj.os_index(),
                obj.logical_index(),
                obj.attributes(),
                Topology::non_io_ancestor_object(obj),
            );
        }

        Ok(())
    }

    #[test]
    fn next_bridge_1() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .io_types_filter(topology::Filter::KeepAll)?
            .build()
            .with_context(|| "failed to build the Topology")?;

        let mut o = None;
        while let Some(obj) = topo.next_object_by_type(ObjectType::Bridge, o) {
            eprintln!("==> Bridge = {:#?}", obj);
            let _prev = o.replace(obj);

            eprintln!(
                "\n{} ({}): #{}(L#{})\n\t└-attributes: {:?}\n",
                obj,
                obj.object_type(),
                obj.os_index(),
                obj.logical_index(),
                obj.attributes(),
            );
        }

        Ok(())
    }

    #[test]
    fn next_bridge_2() -> Result<()> {
        let topo = Topology::builder()
            .with_context(|| "failed to create the TopologyBuilder")?
            .io_types_filter(topology::Filter::KeepAll)?
            .build()
            .with_context(|| "failed to build the Topology")?;

        let mut o = None;
        while let Some(obj) = topo.next_bridge(o) {
            eprintln!("==> Bridge = {:#?}", obj);
            let _prev = o.replace(obj);

            eprintln!(
                "\n{} ({}): #{}(L#{})\n\t└-attributes: {:?}\n\t└-non-io ancestor: {:?}\n",
                obj,
                obj.object_type(),
                obj.os_index(),
                obj.logical_index(),
                obj.attributes(),
                Topology::non_io_ancestor_object(obj),
            );
        }

        Ok(())
    }

    // FIXME(ckatsak): This triggers double-free, for some reason. Try:
    //  $ valgrind \
    //      --leak-check=full --track-origins=yes \
    //      target/debug/deps/<TEST_BIN> 'tests::next_osdev'
    // for more information, although that didn't really help me.
    //#[test]
    //fn next_osdev() -> Result<()> {
    //    let topo = Topology::builder()
    //        .with_context(|| "failed to create the TopologyBuilder")?
    //        .io_types_filter(topology::Filter::KeepAll)?
    //        .build()
    //        .with_context(|| "failed to build the Topology")?;

    //    let mut o = None;
    //    while let Some(obj) = topo.next_osdev(o) {
    //        eprintln!("==> OSDevice = {:#?}", obj);
    //        eprintln!(
    //            "\n{} ({}): #{}(L#{})\n└-attributes: {:?}\n\t└-non-io ancestor: {:?}\n",
    //            obj,
    //            obj.object_type(),
    //            obj.os_index(),
    //            obj.logical_index(),
    //            obj.attributes(),
    //            Topology::non_io_ancestor_object(obj),
    //        );
    //        let _prev = o.replace(obj);
    //    }

    //    Ok(())
    //}
}

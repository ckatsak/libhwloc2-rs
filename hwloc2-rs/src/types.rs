use std::{cmp::Ordering, fmt};

use enum_primitive_derive::Primitive;

/// Type of topology object.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Primitive)]
#[repr(u32)]
pub enum ObjectType {
    /// Machine. A set of processors and memory with cache coherency.
    ///
    /// This type is always used for the root object of a topology, and never used anywhere else.
    /// Hence its parent is always NULL.
    Machine = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_MACHINE,

    /// Physical package. The physical package that usually gets inserted into a socket on the
    /// motherboard. A processor package usually contains multiple cores, and possibly some dies.
    Package = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_PACKAGE,

    /// A computation unit (may be shared by several PUs, aka logical processors).
    Core = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_CORE,

    /// Processing Unit, or (Logical) Processor. An execution unit (may share a core with some
    /// other logical processors, e.g. in the case of an SMT core).
    ///
    /// This is the smallest object representing CPU resources, it cannot have any child except
    /// Misc objects.
    ///
    /// Objects of this kind are always reported and can thus be used as fallback when others are
    /// not.
    PU = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_PU,

    /// Level 1 Data (or Unified) Cache.
    L1Cache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_L1CACHE,

    /// Level 2 Data (or Unified) Cache.
    L2Cache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_L2CACHE,

    /// Level 3 Data (or Unified) Cache.
    L3Cache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_L3CACHE,

    /// Level 4 Data (or Unified) Cache.
    L4Cache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_L4CACHE,

    /// Level 5 Data (or Unified) Cache.
    L5Cache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_L5CACHE,

    /// Level 1 instruction Cache (filtered out by default).
    L1ICache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_L1ICACHE,

    /// Level 2 instruction Cache (filtered out by default).
    L2ICache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_L2ICACHE,

    /// Level 3 instruction Cache (filtered out by default).
    L3ICache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_L3ICACHE,

    /// Group objects. Objects which do not fit in the above but are detected by hwloc and are
    /// useful to take into account for affinity. For instance, some operating systems expose their
    /// arbitrary processors aggregation this way. And hwloc may insert such objects to group NUMA
    /// nodes according to their distances. See also [What are these Group objects in my
    /// topology?](https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00373.php#faq_groups).
    ///
    /// These objects are removed when they do not bring any structure (see
    /// [`Filter::KeepStructure`]).
    ///
    /// [`Filter::KeepStructure`]: crate::topology::filters::Filter::KeepStructure
    Group = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_GROUP,

    /// NUMA node. An object that contains memory that is directly and byte-accessible to the host
    /// processors. It is usually close to some cores (the corresponding objects are descendants of
    /// the NUMA node object in the hwloc tree).
    ///
    /// This is the smallest object representing Memory resources, it cannot have any child except
    /// Misc objects. However it may have Memory-side cache parents.
    ///
    /// There is always at least one such object in the topology even if the machine is not NUMA.
    ///
    /// Memory objects are not listed in the main children list, but rather in the dedicated Memory
    /// children list.
    ///
    /// NUMA nodes have a special depth `HWLOC_TYPE_DEPTH_NUMANODE` instead of a normal depth just
    /// like other objects in the main tree.
    NumaNode = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_NUMANODE,

    /// Bridge (filtered out by default). Any bridge (or PCI switch) that connects the host or an
    /// I/O bus, to another I/O bus.
    ///
    /// Bridges are not added to the topology unless their filtering is changed (see
    /// [`TopologyBuilder::type_filter`] and [`TopologyBuilder::io_types_filter`]).
    ///
    /// I/O objects are not listed in the main children list, but rather in the dedicated io
    /// children list. I/O objects have NULL CPU and node sets.
    ///
    /// [`TopologyBuilder::type_filter`]: crate::TopologyBuilder::type_filter
    /// [`TopologyBuilder::io_types_filter`]: crate::TopologyBuilder::io_types_filter
    Bridge = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_BRIDGE,

    /// PCI device (filtered out by default).
    ///
    /// PCI devices are not added to the topology unless their filtering is changed (see
    /// [`TopologyBuilder::type_filter`] and [`TopologyBuilder::io_types_filter`]).
    ///
    /// I/O objects are not listed in the main children list, but rather in the dedicated io
    /// children list. I/O objects have NULL CPU and node sets.
    ///
    /// [`TopologyBuilder::type_filter`]: crate::TopologyBuilder::type_filter
    /// [`TopologyBuilder::io_types_filter`]: crate::TopologyBuilder::io_types_filter
    PciDevice = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_PCI_DEVICE,

    /// Operating system device (filtered out by default).
    ///
    /// OS devices are not added to the topology unless their filtering is changed (see
    /// [`TopologyBuilder::type_filter`] and [`TopologyBuilder::io_types_filter`]).
    ///
    /// I/O objects are not listed in the main children list, but rather in the dedicated io
    /// children list. I/O objects have NULL CPU and node sets.
    ///
    /// [`TopologyBuilder::type_filter`]: crate::TopologyBuilder::type_filter
    /// [`TopologyBuilder::io_types_filter`]: crate::TopologyBuilder::io_types_filter
    OsDevice = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_OS_DEVICE,

    /// Miscellaneous objects (filtered out by default). Objects without particular meaning, that
    /// can e.g. be added by the application for its own use, or by hwloc for miscellaneous objects
    /// such as MemoryModule (DIMMs).
    ///
    /// They are not added to the topology unless their filtering is changed (see
    /// [`TopologyBuilder::type_filter`]).
    ///
    /// These objects are not listed in the main children list, but rather in the dedicated misc
    /// children list. Misc objects may only have Misc objects as children, and those are in the
    /// dedicated misc children list as well. Misc objects have NULL CPU and node sets.
    ///
    /// [`TopologyBuilder::type_filter`]: crate::TopologyBuilder::type_filter
    Misc = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_MISC,

    /// Memory-side cache (filtered out by default). A cache in front of a specific NUMA node.
    ///
    /// This object always has at least one NUMA node as a memory child.
    ///
    /// Memory objects are not listed in the main children list, but rather in the dedicated Memory
    /// children list.
    ///
    /// Memory-side cache have a special depth `HWLOC_TYPE_DEPTH_MEMCACHE` instead of a normal
    /// depth just like other objects in the main tree.
    MemCache = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_MEMCACHE,

    /// Die within a physical package. A subpart of the physical package, that contains multiple
    /// cores.
    Die = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_DIE,
}
//pub const TYPE_MAX: isize = hwloc2_sys::hwloc_obj_type_t_HWLOC_OBJ_TYPE_MAX as isize;

impl ObjectType {
    /// Compare the depth of two object types.
    ///
    /// Types shouldn't be compared as they are, since newer ones may be added in the future.
    /// This function returns [`Ordering::Less`], [`Ordering::Equal`], or [`Ordering::Greater`]
    /// respectively if `self` objects usually include `other` objects, are the same as `other`
    /// objects, or are included in `other` objects.
    /// If the types can not be compared (because neither is usually contained in the other),
    /// `None` is returned.
    /// Object types containing CPUs can always be compared (usually, a system contains machines
    /// which contain nodes which contain packages which contain caches, which contain cores, which
    /// contain processors).
    ///
    /// # Note
    ///
    /// [`ObjectType::PU`] will always be the deepest, while [`ObjectType::Machine`] is always the
    /// highest.
    ///
    /// This does not mean that the actual topology will respect that order: e.g. as of today cores
    /// may also contain caches, and packages may also contain nodes. This is thus just to be seen
    /// as a fallback comparison method.
    pub fn compare(&self, other: ObjectType) -> Option<Ordering> {
        match unsafe { hwloc2_sys::hwloc_compare_types(*self as _, other as _) } {
            i32::MAX => None, // originally `HWLOC_TYPE_UNORDERED` is #defined equal to `INT_MAX`
            0 => Some(Ordering::Equal),
            i32::MIN..=-1i32 => Some(Ordering::Less),
            _ => Some(Ordering::Greater),
        }
    }

    /// Check whether an object type is Normal.
    ///
    /// Normal objects are objects of the main CPU hierarchy (Machine, Package, Core, PU, CPU
    /// caches, etc.), but they are not NUMA nodes, I/O devices or Misc objects.
    ///
    /// They are attached to parent as Normal children, not as Memory, I/O or Misc children.
    ///
    /// Returns `true` if this object type is a Normal object, `false` otherwise.
    pub fn is_normal(&self) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_obj_type_is_normal(*self as _) }
    }

    /// Check whether an object type is I/O.
    ///
    /// I/O objects are objects attached to their parents in the I/O children list. This current
    /// includes Bridges, PCI and OS devices.
    ///
    /// Returns `true` if this object type is a I/O object, `false` otherwise.
    pub fn is_io(&self) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_obj_type_is_io(*self as _) }
    }

    /// Check whether an object type is Memory.
    ///
    /// Memory objects are objects attached to their parents in the Memory children list. This
    /// current includes NUMA nodes and Memory-side caches.
    ///
    /// Returns `true` if this object type is a Memory object, `false` otherwise.
    pub fn is_memory(&self) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_obj_type_is_memory(*self as _) }
    }

    /// Check whether an object type is a CPU Cache (Data, Unified or Instruction).
    ///
    /// Memory-side caches are not CPU caches.
    ///
    /// Returns `true` if this object type is a Cache, `false` otherwise.
    pub fn is_cache(&self) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_obj_type_is_cache(*self as _) }
    }

    /// Check whether an object type is a CPU Data or Unified Cache.
    ///
    /// Memory-side caches are not CPU caches.
    ///
    /// Returns `true` if this object type is a CPU Data or Unified Cache, `false` otherwise.
    pub fn is_dcache(&self) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_obj_type_is_dcache(*self as _) }
    }

    /// Check whether an object type is a CPU Instruction Cache,.
    ///
    /// Memory-side caches are not CPU caches.
    ///
    /// Returns `true` if this object type is a CPU Instruction Cache, `false` otherwise.
    pub fn is_icache(&self) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_obj_type_is_icache(*self as _) }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ObjectType::*;
        match self {
            Machine => write!(f, "Machine"),
            Package => write!(f, "Package"),
            Core => write!(f, "Core"),
            PU => write!(f, "PU"),
            L1Cache => write!(f, "L1Cache"),
            L2Cache => write!(f, "L2Cache"),
            L3Cache => write!(f, "L3Cache"),
            L4Cache => write!(f, "L4Cache"),
            L5Cache => write!(f, "L5Cache"),
            L1ICache => write!(f, "L1ICache"),
            L2ICache => write!(f, "L2ICache"),
            L3ICache => write!(f, "L3ICache"),
            Group => write!(f, "Group"),
            NumaNode => write!(f, "NumaNode"),
            Bridge => write!(f, "Bridge"),
            PciDevice => write!(f, "PciDevice"),
            OsDevice => write!(f, "OsDevice"),
            Misc => write!(f, "Misc"),
            MemCache => write!(f, "MemCache"),
            Die => write!(f, "Die"),
        }
    }
}

/// Cache type.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Primitive)]
#[repr(u32)]
pub enum CacheType {
    /// Unified cache.
    Unified = hwloc2_sys::hwloc_obj_cache_type_e_HWLOC_OBJ_CACHE_UNIFIED,
    /// Data cache.
    Data = hwloc2_sys::hwloc_obj_cache_type_e_HWLOC_OBJ_CACHE_DATA,
    /// Instruction cache (filtered out by default).
    Instruction = hwloc2_sys::hwloc_obj_cache_type_e_HWLOC_OBJ_CACHE_INSTRUCTION,
}

/// Type of one side (upstream or downstream) of an I/O bridge.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Primitive)]
#[repr(u32)]
pub enum BridgeType {
    /// Host-side of a bridge, only possible upstream.
    Host = hwloc2_sys::hwloc_obj_bridge_type_e_HWLOC_OBJ_BRIDGE_HOST,
    /// PCI-side of a bridge.
    Pci = hwloc2_sys::hwloc_obj_bridge_type_e_HWLOC_OBJ_BRIDGE_PCI,
}

/// Type of a OS device.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Primitive)]
#[repr(u32)]
pub enum OsDevType {
    /// Operating system block device, or non-volatile memory device. For instance "sda" or
    /// "dax2.0" on Linux.
    Block = hwloc2_sys::hwloc_obj_osdev_type_e_HWLOC_OBJ_OSDEV_BLOCK,
    /// Operating system GPU device. For instance ":0.0" for a GL display, "card0" for a Linux DRM
    /// device.
    Gpu = hwloc2_sys::hwloc_obj_osdev_type_e_HWLOC_OBJ_OSDEV_GPU,
    /// Operating system network device. For instance the "eth0" interface on Linux.
    Network = hwloc2_sys::hwloc_obj_osdev_type_e_HWLOC_OBJ_OSDEV_NETWORK,
    /// Operating system openfabrics device. For instance the "mlx4_0" InfiniBand HCA, "hfi1_0"
    /// Omni-Path interface, or "bxi0" Atos/Bull BXI HCA on Linux.
    OpenFabrics = hwloc2_sys::hwloc_obj_osdev_type_e_HWLOC_OBJ_OSDEV_OPENFABRICS,
    /// Operating system dma engine device. For instance the "dma0chan0" DMA channel on Linux.
    Dma = hwloc2_sys::hwloc_obj_osdev_type_e_HWLOC_OBJ_OSDEV_DMA,
    /// Operating system co-processor device. For instance "opencl0d0" for a OpenCL device, "cuda0"
    /// for a CUDA device.
    CoProc = hwloc2_sys::hwloc_obj_osdev_type_e_HWLOC_OBJ_OSDEV_COPROC,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Primitive)]
#[repr(i32)]
pub enum TypeDepth {
    /// No object of given type exists in the topology.
    Unknown = hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_UNKNOWN,
    /// Objects of given type exist at different depth in the topology (only for Groups).
    Multiple = hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_MULTIPLE,
    /// Virtual depth for NUMA nodes.
    NumaNode = hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_NUMANODE,
    /// Virtual depth for bridge object level.
    Bridge = hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_BRIDGE,
    /// Virtual depth for PCI device object level.
    PciDevice = hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_PCI_DEVICE,
    /// Virtual depth for software device object level.
    OsDevice = hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_OS_DEVICE,
    /// Virtual depth for Misc object.
    Misc = hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_MISC,
    /// Virtual depth for MemCache object.
    MemCache = hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_MEMCACHE,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_type_depth_values() {
        assert_eq!(
            TypeDepth::Unknown as i32,
            hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_UNKNOWN
        );
        assert_eq!(
            TypeDepth::Multiple as i32,
            hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_MULTIPLE
        );
        assert_eq!(
            TypeDepth::NumaNode as i32,
            hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_NUMANODE
        );
        assert_eq!(
            TypeDepth::Bridge as i32,
            hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_BRIDGE
        );
        assert_eq!(
            TypeDepth::PciDevice as i32,
            hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_PCI_DEVICE
        );
        assert_eq!(
            TypeDepth::OsDevice as i32,
            hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_OS_DEVICE
        );
        assert_eq!(
            TypeDepth::Misc as i32,
            hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_MISC
        );
        assert_eq!(
            TypeDepth::MemCache as i32,
            hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_MEMCACHE
        );
    }
}

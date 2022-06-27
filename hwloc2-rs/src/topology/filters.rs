use std::fmt;

use enum_primitive_derive::Primitive;

/// Type filtering flags.
///
/// By default, most objects are kept ([`Filter::KeepAll`]). Instruction caches, I/O and
/// Misc objects are ignored by default ([`Filter::KeepNone`]). Die and Group levels are
/// ignored unless they bring structure ([`Filter::KeepStructure`]).
///
/// Note that group objects are also ignored individually (without the entire level) when they do
/// not bring structure.
#[derive(Debug, Clone, Copy, Primitive)]
#[repr(u32)]
pub enum Filter {
    /// Keep all objects of this type.
    ///
    /// Cannot be set for [`ObjectType::Group`] (groups are designed only to add more structure to
    /// the topology).
    ///
    /// [`ObjectType::Group`]: crate::types::ObjectType::Group
    KeepAll = hwloc2_sys::hwloc_type_filter_e_HWLOC_TYPE_FILTER_KEEP_ALL,

    /// Ignore all objects of this type.
    ///
    /// The bottom-level type [`ObjectType::PU`], the [`ObjectType::NumaNode`] type, and the
    /// top-level type [`ObjectType::Machine`] may not be ignored.
    ///
    /// [`ObjectType::PU`]: crate::types::ObjectType::PU
    /// [`ObjectType::NumaNode`]: crate::types::ObjectType::NumaNode
    /// [`ObjectType::Machine`]: crate::types::ObjectType::Machine
    KeepNone = hwloc2_sys::hwloc_type_filter_e_HWLOC_TYPE_FILTER_KEEP_NONE,

    /// Only ignore objects if their entire level does not bring any structure.
    ///
    /// Keep the entire level of objects if at least one of these objects adds structure to the
    /// topology. An object brings structure when it has multiple children and it is not the only
    /// child of its parent.
    ///
    /// If all objects in the level are the only child of their parent, and if none of them has
    /// multiple children, the entire level is removed.
    ///
    /// Cannot be set for I/O and Misc objects since the topology structure does not matter there.
    KeepStructure = hwloc2_sys::hwloc_type_filter_e_HWLOC_TYPE_FILTER_KEEP_STRUCTURE,

    /// Only keep likely-important objects of the given type.
    ///
    /// It is only useful for I/O object types. For [`ObjectType::PciDevice`] and
    /// [`ObjectType::OsDevice`], it means that only objects of major/common kinds are kept
    /// (storage, network, OpenFabrics, CUDA, OpenCL, RSMI, NVML, and displays).
    /// Also, only OS devices directly attached on PCI (e.g. no USB) are reported. For
    /// [`ObjectType::Bridge`], it means that bridges are kept only if they have children.
    ///
    /// This flag equivalent to [`Filter::KeepAll`] for Normal, Memory and Misc types
    /// since they are likely important.
    ///
    /// [`ObjectType::PciDevice`]: crate::types::ObjectType::PciDevice
    /// [`ObjectType::OsDevice`]: crate::types::ObjectType::OsDevice
    /// [`ObjectType::Bridge`]: crate::types::ObjectType::Bridge
    KeepImportant = hwloc2_sys::hwloc_type_filter_e_HWLOC_TYPE_FILTER_KEEP_IMPORTANT,
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Filter::*;
        match self {
            KeepAll => write!(f, "Filter::KeepAll"),
            KeepNone => write!(f, "Filter::KeepNone"),
            KeepStructure => write!(f, "Filter::KeepStructure"),
            KeepImportant => write!(f, "Filter::KeepImportant"),
        }
    }
}

use std::ptr;

pub mod filters;
pub mod flags;
pub mod support;

pub use filters::Filter;
pub use flags::Flags;
pub use support::Support;

use num_traits::FromPrimitive;

use crate::{
    bitmap::{Bitmap, CpuSet, NodeSet},
    error::Error,
    object::{Attributes, Object},
    ptr_mut_to_const,
    types::{BridgeType, ObjectType, TypeDepth},
};

#[derive(Debug)]
pub struct Topology {
    topo: *mut hwloc2_sys::hwloc_topology,
    support: support::Support,
}

unsafe impl Sync for Topology {}
unsafe impl Send for Topology {}

impl Topology {
    /// Create a new [`TopologyBuilder`] to configure and create a new [`Topology`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologyInit`] if hwloc fails to initialize topology's context.
    ///
    /// [`Error::TopologyInit`]: crate::error::Error::TopologyInit
    pub fn builder() -> Result<TopologyBuilder, Error> {
        let mut topo = ptr::null_mut();

        // Initialize a new topology context.
        // SAFETY: `topo` is a freshly allocated pointer, of the correct type, set to NULL.
        match unsafe { hwloc2_sys::hwloc_topology_init(&mut topo) } {
            -1 => Err(Error::TopologyInit),
            _ => Ok(TopologyBuilder { topo, built: false }),
        }
    }

    /// Retrieve the topology support.
    ///
    /// Each flag indicates whether a feature is supported. If set to 0, the feature is not
    /// supported. If set to 1, the feature is supported, but the corresponding call may still fail
    /// in some corner cases.
    ///
    /// These features are also listed by `hwloc-info --support`.
    ///
    /// The reported features are what the current topology supports on the current machine. If the
    /// topology was exported to XML from another machine and later imported here, support still
    /// describes what is supported for this imported topology after import. By default, binding
    /// will be reported as unsupported in this case (see [`Flags::IS_THISSYSTEM`]).
    ///
    /// Topology flag [`Flags::IMPORT_SUPPORT`] may be used to report the supported features of the
    /// original remote machine instead. If it was successfully imported, `imported_support` will
    /// be set in [`support::Misc`].
    ///
    /// [`Flags::IS_THISSYSTEM`]: crate::topology::flags::Flags::IS_THISSYSTEM
    /// [`Flags::IMPORT_SUPPORT`]: crate::topology::flags::Flags::IMPORT_SUPPORT
    /// [`support::Misc`]: crate::topology::support::Misc
    pub fn support(&self) -> &support::Support {
        &self.support
    }

    /// Retrieve the OR'ed flags of the topology.
    ///
    /// # Note
    ///
    /// The implementation truncates any bits returned by the bindings unless they correspond to a
    /// known valid flag.
    pub fn flags(&self) -> flags::Flags {
        flags::Flags::from_bits_truncate(
            // SAFETY: `self.topo` has been private since it was created via `Self::new`, therefore
            // it remains valid.
            unsafe { hwloc2_sys::hwloc_topology_get_flags(self.topo) },
        )

        //unsafe {
        //    flags::Flags::from_bits_unchecked(hwloc2_sys::hwloc_topology_get_flags(self.topo))
        //}
    }

    /// Get the current filtering for the given object type.
    ///
    /// # Errors
    ///
    /// - [`Error::TopologyGetFilter`] for an error returned by hwloc.
    /// - [`Error::UnknownFilter`] if the `i32` value returned by hwloc does not correspond to a
    /// known [`Filter`].
    ///
    /// [`Error::TopologyGetFilter`]: crate::error::Error::TopologyGetFilter
    /// [`Error::UnknownFilter`]: crate::error::Error::UnknownFilter
    /// [`Filter`]: crate::topology::filters::Filter
    pub fn type_filter(&self, obj_type: ObjectType) -> Result<filters::Filter, Error> {
        let mut out = 0u32;
        match unsafe {
            hwloc2_sys::hwloc_topology_get_type_filter(
                self.topo,
                obj_type as u32,
                ptr::addr_of_mut!(out),
            )
        } {
            -1 => Err(Error::TopologyGetFilter(obj_type)),
            f => filters::Filter::try_from(out).map_err(|_| Error::UnknownFilter(obj_type, f)),
        }
    }

    /// Get the depth of the hierarchical tree of objects.
    ///
    /// This is the depth of [`ObjectType::PU`] objects plus one.
    ///
    /// # Note
    ///
    /// NUMA nodes, I/O and Misc objects are ignored when computing the depth of the tree (they are
    /// placed on special levels).
    ///
    /// [`ObjectType::PU`]: crate::types::ObjectType::PU
    pub fn depth(&self) -> i32 {
        unsafe { hwloc2_sys::hwloc_topology_get_depth(self.topo) }
    }

    /// Returns the depth of objects of type `obj_type`.
    ///
    /// If no object of this type is present on the underlying architecture, or if the OS doesn't
    /// provide this kind of information, the function returns `HWLOC_TYPE_DEPTH_UNKNOWN` (i.e.,
    /// [`TypeDepth::Unknown`]` as i32`).
    ///
    /// If `obj_type` is absent but a similar type is acceptable, see also
    /// [`Topology::type_or_below_depth`] and [`Topology::type_or_above_depth`].
    ///
    /// If [`ObjectType::Group`] is given, the function may return `HWLOC_TYPE_DEPTH_MULTIPLE`
    /// (i.e., [`TypeDepth::Multiple`]` as i32`) if multiple levels of Groups exist.
    ///
    /// If a NUMA node, I/O or Misc object type is given, the function returns a virtual value
    /// because these objects are stored in special levels that are not CPU-related. This virtual
    /// depth may be passed to other hwloc functions such as [`Topology::object_by_depth`]
    /// but it should not be considered as an actual depth by the application. In particular, it
    /// should not be compared with any other object depth or with the entire topology depth.
    ///
    /// See also:
    /// - [`Topology::memory_parents_depth`]
    /// - `hwloc_type_sscanf_as_depth()` for returning the depth of objects whose type is given as
    /// a string FIXME doclink
    ///
    /// [`TypeDepth::Unknown`]: crate::types::TypeDepth::Unknown
    /// [`TypeDepth::Multiple`]: crate::types::TypeDepth::Multiple
    /// [`ObjectType::Group`]: crate::types::ObjectType::Group
    pub fn type_depth(&self, obj_type: ObjectType) -> i32 {
        unsafe { hwloc2_sys::hwloc_get_type_depth(self.topo, obj_type as u32) }
    }

    /// Return the depth of parents where memory objects are attached.
    ///
    /// Memory objects have virtual negative depths because they are not part of the main CPU-side
    /// hierarchy of objects. This depth should not be compared with other level depths.
    ///
    /// If all Memory objects are attached to Normal parents at the same depth, this parent depth
    /// may be compared to other as usual, for instance for knowing whether NUMA nodes is attached
    /// above or below Packages.
    ///
    /// Returns:
    /// - The depth of Normal parents of all memory children if all these parents have the same
    /// depth. For instance the depth of the Package level if all NUMA nodes are attached to
    /// Package objects.
    /// - `HWLOC_TYPE_DEPTH_MULTIPLE` (i.e., [`TypeDepth::Multiple`]` as i32`) if Normal parents
    /// of all memory children do not have the same depth. For instance if some NUMA nodes are
    /// attached to Packages while others are attached to Groups.
    ///
    /// [`TypeDepth::Multiple`]: crate::types::TypeDepth::Multiple
    pub fn memory_parents_depth(&self) -> i32 {
        unsafe { hwloc2_sys::hwloc_get_memory_parents_depth(self.topo) }
    }

    /// Returns the depth of objects of type `type` or below.
    ///
    /// If no object of this type is present on the underlying architecture, the function returns
    /// the depth of the first "present" object typically found inside type.
    ///
    /// This function is only meaningful for normal object types. If a memory, I/O or Misc object
    /// type is given, the corresponding virtual depth is always returned (see
    /// [`Topology::type_depth`]).
    ///
    /// May return `HWLOC_TYPE_DEPTH_MULTIPLE` (i.e., [`TypeDepth::Multiple`]` as i32`) for
    /// [`ObjectType::Group`] just like [`Topology::type_depth`].
    ///
    /// [`TypeDepth::Multiple`]: crate::types::TypeDepth::Multiple
    /// [`ObjectType::Group`]: crate::types::ObjectType::Group
    // Implementation port from C (file `_build/include/hwloc/inlines.h`).
    pub fn type_or_below_depth(&self, obj_type: ObjectType) -> i32 {
        let d = self.type_depth(obj_type);
        if d != hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_UNKNOWN {
            return d;
        }

        // Find the highest existing level with type order >=
        let mut d = self.type_depth(ObjectType::PU);
        loop {
            let tmp_type = unsafe { hwloc2_sys::hwloc_get_depth_type(self.topo, d) };
            if unsafe { hwloc2_sys::hwloc_compare_types(tmp_type, obj_type as u32) } < 0 {
                return d + 1;
            }
            d -= 1;
        }
        // The loop finishes, as there is always a Machine level with lower order and known depth.
    }

    /// Returns the depth of objects of type `obj_type` or above.
    ///
    /// If no object of this type is present on the underlying architecture, the function returns
    /// the depth of the first "present" object typically containing type.
    ///
    /// This function is only meaningful for normal object types. If a memory, I/O or Misc object
    /// type is given, the corresponding virtual depth is always returned (see
    /// [`Topology::type_depth`]).
    ///
    /// May return `HWLOC_TYPE_DEPTH_MULTIPLE` (i.e., [`TypeDepth::Multiple`]` as i32`) for
    /// [`ObjectType::Group`] just like [`Topology::type_depth`].
    ///
    /// [`TypeDepth::Multiple`]: crate::types::TypeDepth::Multiple
    /// [`ObjectType::Group`]: crate::types::ObjectType::Group
    // Implementation port from C (file `_build/include/hwloc/inlines.h`).
    pub fn type_or_above_depth(&self, obj_type: ObjectType) -> i32 {
        let d = self.type_depth(obj_type);
        if d != hwloc2_sys::hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_UNKNOWN {
            return d;
        }

        // Find the lowest existing level with type order <=
        let mut d = 0;
        loop {
            let tmp_type = unsafe { hwloc2_sys::hwloc_get_depth_type(self.topo, d) };
            if unsafe { hwloc2_sys::hwloc_compare_types(tmp_type, obj_type as u32) } > 0 {
                return d - 1;
            }
            d += 1;
        }
        // The loop finishes, as there is always a PU level with higherorder and known depth.
    }

    /// Returns the type of objects at depth `depth`.
    ///
    /// `depth` should be between `0` and [`Topology::depth`]` - 1`, or a virtual depth such as
    /// `HWLOC_TYPE_DEPTH_NUMANODE` (i.e., [`TypeDepth::NumaNode`]` as i32`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologyDepthDoesNotExist`] if depth `depth` does not exist.
    ///
    /// [`TypeDepth::NumaNode`]: crate::types::TypeDepth::NumaNode
    /// [`Error::TopologyDepthDoesNotExist`]: crate::error::Error::TopologyDepthDoesNotExist
    pub fn depth_type(&self, depth: i32) -> Result<ObjectType, Error> {
        ObjectType::from_u32(unsafe { hwloc2_sys::hwloc_get_depth_type(self.topo, depth) })
            .ok_or(Error::TopologyDepthDoesNotExist(depth))
    }

    /// Returns the width of level at depth `depth`.
    pub fn nbobjs_by_depth(&self, depth: i32) -> u32 {
        unsafe { hwloc2_sys::hwloc_get_nbobjs_by_depth(self.topo, depth) }
    }

    /// Returns the width of level type `type`.
    ///
    /// If no object for that type exists, `0` is returned. If there are several levels with
    /// objects of that type, `-1` is returned.
    pub fn nbobjs_by_type(&self, obj_type: ObjectType) -> i32 {
        match self.type_depth(obj_type) {
            d if d == TypeDepth::Unknown as i32 => 0,
            d if d == TypeDepth::Multiple as i32 => -1,
            depth => self.nbobjs_by_depth(depth) as i32,
        }
    }

    /// Get the object at the root of the topology.
    pub fn root_object(&self) -> Option<Object> {
        self.object_by_depth(0, 0)
    }

    /// Returns the topology object at logical index `idx` from depth `depth`.
    pub fn object_by_depth(&self, depth: i32, idx: u32) -> Option<Object> {
        let obj = unsafe { hwloc2_sys::hwloc_get_obj_by_depth(self.topo, depth, idx) };
        if obj.is_null() {
            return None;
        }
        Some(unsafe { Object::new(ptr_mut_to_const(obj)) })
    }

    /// Returns the topology object at logical index `idx` with type `obj_type`.
    ///
    /// If no object for that type exists, `None` is returned. If there are several levels with
    /// objects of that type ([`ObjectType::Group`]), `None` is returned and the caller may
    /// fallback to [`Topology::object_by_depth`].
    ///
    /// [`ObjectType::Group`]: crate::types::ObjectType::Group
    pub fn object_by_type(&self, obj_type: ObjectType, idx: u32) -> Option<Object> {
        match self.type_depth(obj_type) {
            d if d == TypeDepth::Unknown as i32 || d == TypeDepth::Multiple as i32 => None,
            depth => self.object_by_depth(depth, idx),
        }
    }

    /// Returns the next object at depth `depth`.
    ///
    /// If `prev` is `None`, return the first object at depth `depth`.
    pub fn next_object_by_depth<'topo: 'prev, 'prev: 'next, 'next>(
        &'topo self,
        depth: i32,
        prev: Option<Object<'prev>>,
    ) -> Option<Object<'next>> {
        if let Some(prev) = prev {
            if prev.depth() != depth {
                None
            } else {
                prev.next_cousin()
            }
        } else {
            self.object_by_depth(depth, 0)
        }
    }

    /// Returns the next object of type `type`.
    ///
    /// If `prev` is `None`, return the first object at type `type`. If there are multiple or no
    /// depth for given type, return `None` and let the caller fallback to
    /// [`Topology::next_object_by_depth`].
    pub fn next_object_by_type<'topo, 'prev, 'next>(
        &'topo self,
        obj_type: ObjectType,
        prev: Option<Object<'prev>>,
    ) -> Option<Object<'next>>
    where
        'topo: 'prev,
        'prev: 'next,
    {
        match self.type_depth(obj_type) {
            d if d == TypeDepth::Unknown as i32 || d == TypeDepth::Multiple as i32 => None,
            depth => self.next_object_by_depth(depth, prev),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  Finding objects, miscellaneous helpers
    /////
    /////  https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00176.php
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    /// TODO: UNTESTED
    ///
    /// Remove simultaneous multithreading PUs from a CPU set.
    ///
    /// For each core in topology, if cpuset contains some PUs of that core, modify `cpuset` to
    /// only keep a single PU for that core.
    ///
    /// `which` specifies which PU will be kept. PU are considered in physical index order. If `0`,
    /// for each core, the function keeps the first PU that was originally set in `cpuset`.
    ///
    /// If `which` is larger than the number of PUs in a core there were originally set in
    /// `cpuset`, no PU is kept for that core.
    ///
    /// # Note
    ///
    /// PUs that are not below a Core object are ignored (for instance if the topology does not
    /// contain any Core object). None of them is removed from `cpuset`.
    pub fn bitmap_singlify_per_core(&self, cpuset: CpuSet, which: u32) -> i32 {
        unsafe { hwloc2_sys::hwloc_bitmap_singlify_per_core(self.topo, cpuset.as_ptr(), which) }
    }

    /// Returns the object of type [`ObjectType::NumaNode`] with `os_index`.
    ///
    /// This function is useful for converting a nodeset into the NUMA node objects it contains.
    /// When retrieving the current binding (e.g. with `hwloc_get_membind()` with
    /// `HWLOC_MEMBIND_BYNODESET`), one may iterate over the bits of the resulting nodeset with
    /// `hwloc_bitmap_foreach_begin()`, and find the corresponding NUMA nodes with this function.
    ///
    /// (FIXME: Documentation not updated & 3 doclinks missing)
    ///
    /// [`ObjectType::NumaNode`]: crate::types::ObjectType::NumaNode
    // Implementation port from C (file `include/hwloc/helper.h`).
    pub fn numanode_object_by_os_index<'o, 't: 'o>(&'t self, os_index: u32) -> Option<Object<'o>> {
        let mut ret = None;
        while let Some(obj) = self.next_object_by_type(ObjectType::NumaNode, ret) {
            if obj.os_index() == os_index {
                return ret;
            }
            ret.replace(obj);
        }
        None
    }

    /// Returns the object of type [`ObjectType::PU`] with os_index.
    ///
    /// This function is useful for converting a CPU set into the PU objects it contains. When
    /// retrieving the current binding (e.g. with `hwloc_get_cpubind()`), one may iterate over the
    /// bits of the resulting CPU set with `hwloc_bitmap_foreach_begin()`, and find the
    /// corresponding PUs with this function.
    ///
    /// (FIXME: Documentation not updated & 2 doclinks missing)
    ///
    /// [`ObjectType::PU`]: crate::types::ObjectType::PU
    // Implementation port from C (file `include/hwloc/helper.h`).
    pub fn pu_object_by_os_index<'o, 't: 'o>(&'t self, os_index: u32) -> Option<Object<'o>> {
        let mut ret = None;
        while let Some(obj) = self.next_object_by_type(ObjectType::PU, ret) {
            if obj.os_index() == os_index {
                return ret;
            }
            ret.replace(obj);
        }
        None
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  CPU binding
    /////
    /////  https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00166.php
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    /// TODO: UNTESTED
    ///
    /// Bind current process or thread on CPUs given in physical bitmap set.
    ///
    /// # Errors
    ///
    /// Returns [`Error::CpuBindSet`] in case of failure.
    ///
    /// [`Error::CpuBindSet`]: crate::error::Error::CpuBindSet
    pub fn set_cpubind(&self, cpuset: CpuSet, flags: i32) -> Result<(), Error> {
        match unsafe { hwloc2_sys::hwloc_set_cpubind(self.topo, cpuset.as_ptr(), flags as i32) } {
            -1 => Err(Error::CpuBindSet),
            _ => Ok(()),
        }
    }

    // TODO
    //pub fn cpubind(&self, cpuset: CpuSet, flags: i32) {
    //    unsafe { hwloc2_sys::hwloc_get_cpubind(self.topo, cpuset, flags) }
    //}

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  CPU and node sets of entire topologies
    /////
    /////  https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00178.php
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    /// TODO: UNTESTED
    ///
    /// Get complete CPU set.
    ///
    /// Returns the complete CPU set of processors of the system.
    ///
    /// # Notes
    ///
    /// - The returned cpuset is not newly allocated and should thus not be changed or freed;
    /// [`CpuSet::clone`] must be used to obtain a local copy.
    /// - This is equivalent to retrieving the root object complete CPU-set.
    ///
    /// [`CpuSet::clone`]: crate::bitmap::Bitmap::clone
    pub fn complete_cpuset(&self) -> Result<CpuSet, Error> {
        let bmptr = unsafe { hwloc2_sys::hwloc_topology_get_complete_cpuset(self.topo) };
        unsafe { Bitmap::from_raw(bmptr as *mut _, false) }
    }

    /// TODO: UNTESTED
    ///
    /// Get topology CPU set.
    ///
    /// Returns the CPU set of processors of the system for which hwloc provides topology
    /// information. This is equivalent to the cpuset of the system object.
    ///
    /// # Notes
    ///
    /// - The returned cpuset is not newly allocated and should thus not be changed or freed;
    /// [`CpuSet::clone`] must be used to obtain a local copy.
    /// - This is equivalent to retrieving the root object CPU-set.
    ///
    /// [`CpuSet::clone`]: crate::bitmap::Bitmap::clone
    pub fn topology_cpuset(&self) -> Result<CpuSet, Error> {
        let bmptr = unsafe { hwloc2_sys::hwloc_topology_get_topology_cpuset(self.topo) };
        unsafe { Bitmap::from_raw(bmptr as *mut _, false) }
    }

    /// TODO: UNTESTED
    ///
    /// Get allowed CPU set.
    ///
    /// Returns the CPU set of allowed processors of the system.
    ///
    /// # Notes
    ///
    /// - If the topology flag [`Flags::INCLUDE_DISALLOWED`] was not set, this is identical to
    /// [`Topology::topology_cpuset`], which means all PUs are allowed.
    /// - If [`Flags::INCLUDE_DISALLOWED`] was set, applying `hwloc_bitmap_intersects()` on the
    // FIXME doclink:                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// result of this function and on an object cpuset checks whether there are allowed PUs inside
    /// that object. Applying `hwloc_bitmap_and()` returns the list of these allowed PUs.
    // FIXME doclink:         ^^^^^^^^^^^^^^^^^^^^
    /// - The returned cpuset is not newly allocated and should thus not be changed or freed,
    /// [`CpuSet::clone`] must be used to obtain a local copy.
    ///
    /// [`Flags::INCLUDE_DISALLOWED`]: crate::topology::flags::Flags::INCLUDE_DISALLOWED
    /// [`CpuSet::clone`]: crate::bitmap::Bitmap::clone
    pub fn allowed_cpuset(&self) -> Result<CpuSet, Error> {
        let bmptr = unsafe { hwloc2_sys::hwloc_topology_get_allowed_cpuset(self.topo) };
        unsafe { Bitmap::from_raw(bmptr as *mut _, false) }
    }

    /// TODO: UNTESTED
    ///
    /// Get complete node set.
    ///
    /// Returns the complete node set of memory of the system.
    ///
    /// # Notes
    ///
    /// - The returned nodeset is not newly allocated and should thus not be changed or freed;
    /// [`NodeSet::clone`] must be used to obtain a local copy.
    /// - This is equivalent to retrieving the root object complete nodeset.
    ///
    /// [`NodeSet::clone`]: crate::bitmap::Bitmap::clone
    pub fn complete_nodeset(&self) -> Result<NodeSet, Error> {
        let bmptr = unsafe { hwloc2_sys::hwloc_topology_get_complete_nodeset(self.topo) };
        unsafe { Bitmap::from_raw(bmptr as *mut _, false) }
    }

    /// TODO: UNTESTED
    ///
    /// Get topology node set.
    ///
    /// Returns the node set of memory of the system for which hwloc provides topology information.
    /// This is equivalent to the nodeset of the system object.
    ///
    /// # Notes
    ///
    /// - The returned nodeset is not newly allocated and should thus not be changed or freed;
    /// [`NodeSet::clone`] must be used to obtain a local copy.
    /// - This is equivalent to retrieving the root object nodeset.
    ///
    /// [`NodeSet::clone`]: crate::bitmap::Bitmap::clone
    pub fn topology_nodeset(&self) -> Result<NodeSet, Error> {
        let bmptr = unsafe { hwloc2_sys::hwloc_topology_get_topology_nodeset(self.topo) };
        unsafe { Bitmap::from_raw(bmptr as *mut _, false) }
    }

    /// TODO: UNTESTED
    ///
    /// Get allowed node set.
    ///
    /// Returns the node set of allowed memory of the system.
    ///
    /// # Notes
    ///
    /// - If the topology flag [`Flags::INCLUDE_DISALLOWED`] was not set, this is identical to
    /// [`Topology::topology_nodeset`], which means all NUMA nodes are allowed.
    /// - If [`Flags::INCLUDE_DISALLOWED`] was set, applying `hwloc_bitmap_intersects()` on the
    // FIXME doclink:                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// result of this function and on an object nodeset checks whether there are allowed NUMA
    /// nodes inside that object. Applying `hwloc_bitmap_and()` returns the list of these allowed
    // FIXME doclink:                      ^^^^^^^^^^^^^^^^^^^^
    /// NUMA nodes.
    /// - The returned nodeset is not newly allocated and should thus not be changed or freed,
    /// [`NodeSet::clone`] must be used to obtain a local copy.
    ///
    /// [`Flags::INCLUDE_DISALLOWED`]: crate::topology::flags::Flags::INCLUDE_DISALLOWED
    /// [`NodeSet::clone`]: crate::bitmap::Bitmap::clone
    pub fn allowed_nodeset(&self) -> Result<NodeSet, Error> {
        let bmptr = unsafe { hwloc2_sys::hwloc_topology_get_allowed_nodeset(self.topo) };
        unsafe { Bitmap::from_raw(bmptr as *mut _, false) }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  Finding Objects inside a CPU set
    /////
    /////  https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00171.php
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    // TODO

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  Finding Objects covering at least CPU set
    /////
    /////  https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00172.php
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    /// TODO: UNTESTED
    ///
    /// Get the child covering at least CPU set `cpuset`.
    ///
    /// Returns `None` if no child matches or if set is empty.
    ///
    /// # Note
    ///
    /// This function cannot work if parent does not have a CPU set (I/O or Misc objects).
    // Implementation port from C (file `include/hwloc/helper.h`).
    pub fn child_covering_cpuset<'topo, 'parent, 'child>(
        &'topo self,
        cpuset: CpuSet,
        parent: Object<'parent>,
    ) -> Option<Object<'child>>
    where
        'topo: 'parent,
        'parent: 'child,
    {
        if cpuset.is_zero() {
            return None;
        }

        let mut o = parent.first_child();
        while let Some(child) = o {
            if let Some(child_cpuset) = child.cpuset() {
                if cpuset.is_included(&child_cpuset) {
                    return Some(child);
                }
            }
            o = child.next_sibling();
        }
        None
    }

    /// TODO: UNTESTED
    ///
    /// Get the lowest object covering at least CPU set `cpuset`.
    ///
    /// Returns `None` if no object matches or if set is empty.
    // Implementation port from C (file `include/hwloc/helper.h`).
    pub fn object_covering_cpuset<'o, 't: 'o>(&'t self, cpuset: CpuSet) -> Option<Object<'o>> {
        let mut curr = self.root_object()?;
        if cpuset.is_zero()
            || !cpuset.is_included(
                &curr
                    .cpuset()
                    .expect("failed to retrieve current object's cpuset"),
            )
        {
            return None;
        }
        loop {
            if let Some(child) = self.child_covering_cpuset(cpuset.clone(), curr) {
                curr = child;
            } else {
                return Some(curr);
            }
        }
    }

    /// TODO: UNTESTED
    ///
    /// Iterate through same-depth objects covering at least CPU set `cpuset`.
    ///
    /// If object `prev` is `None`, return the first object at depth `depth` covering at least part
    /// of CPU set `cpuset`. The next invocation should pass the previous return value in `prev` so
    /// as to obtain the next object covering at least another part of set.
    ///
    /// # Note
    ///
    /// This function cannot work if objects at the given depth do not have CPU sets (I/O or Misc
    /// objects).
    // Implementation port from C (file `include/hwloc/helper.h`).
    pub fn next_object_covering_cpuset_by_depth<'topo, 'prev, 'next>(
        &'topo self,
        cpuset: CpuSet,
        depth: i32,
        prev: Option<Object<'prev>>,
    ) -> Option<Object<'next>>
    where
        'topo: 'prev,
        'prev: 'next,
    {
        let mut o = self.next_object_by_depth(depth, prev);
        while let Some(next) = o {
            if !cpuset.intersects(next.cpuset().expect("failed to retrieve next's cpuset")) {
                o.replace(next);
            } else {
                return Some(next);
            }
        }
        None
    }

    /// TODO: UNTESTED
    ///
    /// Iterate through same-type objects covering at least CPU set `cpuset`.
    ///
    /// If object `prev` is `None`, return the first object of type `obj_type` covering at least
    /// part of CPU set `cpuset`. The next invocation should pass the previous return value in
    /// `prev` so as to obtain the next object of type `obj_type` covering at least another part of
    /// set.
    ///
    /// If there are no or multiple depths for type `obj_type`, `None` is returned. The caller may
    /// fallback to [`Topology::next_object_covering_cpuset_by_depth`] for each depth.
    ///
    /// # Note
    ///
    /// This function cannot work if objects of the given type do not have CPU sets (I/O or Misc
    /// objects).
    // Implementation port from C (file `include/hwloc/helper.h`).
    pub fn next_object_covering_cpuset_by_type<'topo, 'prev, 'next>(
        &'topo self,
        cpuset: CpuSet,
        obj_type: ObjectType,
        prev: Option<Object<'prev>>,
    ) -> Option<Object<'next>>
    where
        'topo: 'prev,
        'prev: 'next,
    {
        match self.type_depth(obj_type) {
            d if d == TypeDepth::Unknown as i32 || d == TypeDepth::Multiple as i32 => None,
            depth => self.next_object_covering_cpuset_by_depth(cpuset, depth, prev),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  Converting between CPU sets and node sets
    /////
    /////  https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00179.php
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    /// TODO: UNTESTED
    ///
    /// Convert a CPU set into a NUMA node set.
    ///
    /// For each PU included in the input `cpuset`, set the corresponding local NUMA node(s) in the
    /// output node set.
    ///
    /// If some NUMA nodes have no CPUs at all, this function never sets their indexes in the
    /// output node set, even if a full CPU set is given in input.
    ///
    /// Hence the entire topology CPU set is converted into the set of all nodes that have some
    /// local CPUs.
    // Implementation port from C (file `include/hwloc/helper.h`).
    pub fn cpuset_to_nodeset(&self, cpuset: CpuSet) -> Result<NodeSet, Error> {
        let depth = self.type_depth(ObjectType::NumaNode);
        assert_ne!(depth, TypeDepth::Unknown as i32);
        let mut ret = NodeSet::try_new_empty()?;
        let mut o = None;
        while let Some(obj) = self.next_object_covering_cpuset_by_depth(cpuset.clone(), depth, o) {
            ret.set(obj.os_index())?;
            o.replace(obj);
        }
        Ok(ret)
    }

    /// Convert a NUMA node set into a CPU set.
    ///
    /// For each NUMA node included in the input `nodeset`, set the corresponding local PUs in the
    /// output CPU set.
    ///
    /// If some CPUs have no local NUMA nodes, this function never sets their indexes in the output
    /// CPU set, even if a full node set is given in input.
    ///
    /// Hence the entire topology node set is converted into the set of all CPUs that have some
    /// local NUMA nodes.
    // Implementation port from C (file `include/hwloc/helper.h`).
    pub fn cpuset_from_nodeset(&self, nodeset: NodeSet) -> Result<CpuSet, Error> {
        let depth = self.type_depth(ObjectType::NumaNode);
        assert_ne!(depth, TypeDepth::Unknown as i32);
        let mut ret = CpuSet::try_new_empty()?;
        let mut o = None;
        while let Some(obj) = self.next_object_by_depth(depth, o) {
            if nodeset.is_set(obj.os_index()) {
                ret |= &obj.cpuset().expect(
                    "no need to check obj->cpuset because objects in levels always have a cpuset",
                );
            }
            o.replace(obj);
        }
        Ok(ret)
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /////
    /////  Finding I/O Objects
    /////
    /////  https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00180.php
    /////
    ///////////////////////////////////////////////////////////////////////////////////////////////

    /// Get the next bridge in the system.
    ///
    /// Returns the first bridge if `prev` is `None`.
    pub fn next_bridge<'topo: 'prev, 'prev: 'next, 'next>(
        &'topo self,
        prev: Option<Object<'prev>>,
    ) -> Option<Object<'next>> {
        self.next_object_by_type(ObjectType::Bridge, prev)
    }

    // TODO(ckatsak): UNTESTED
    pub fn bridge_covers_pcibus(obj: Object<'_>, domain: u16, bus: u8) -> bool {
        match obj.attributes() {
            Some(Attributes::Bridge(attrs)) => {
                matches!(attrs.downstream_type(), BridgeType::Pci)
                    && attrs.downstream_domain() == domain
                    && attrs.downstream_secondary_bus() <= bus
                    && attrs.downstream_subordinate_bus() >= bus
            }
            _ => false,
        }
    }

    // FIXME(ckatsak): BUG; see tests.
    /// Get the next OS device in the system.
    ///
    /// Returns the first OS device if `prev` is `None`.
    #[deprecated = "BUG: may trigger a double free() issue"]
    pub fn next_osdev<'topo: 'prev, 'prev: 'next, 'next>(
        &'topo self,
        prev: Option<Object<'prev>>,
    ) -> Option<Object<'next>> {
        self.next_object_by_type(ObjectType::OsDevice, prev)
    }

    /// Get the next PCI device in the system.
    ///
    /// Returns the first PCI device if `prev` is `None`.
    pub fn next_pcidev<'topo: 'prev, 'prev: 'next, 'next>(
        &'topo self,
        prev: Option<Object<'prev>>,
    ) -> Option<Object<'next>> {
        self.next_object_by_type(ObjectType::PciDevice, prev)
    }

    // TODO(ckatsak): UNTESTED
    /// Find the PCI device object matching the PCI bus id given domain, bus device and function
    /// PCI bus id.
    pub fn pcidev_by_busid<'topo: 'next, 'next>(
        &'topo self,
        domain: u16,
        bus: u8,
        dev: u16,
        func: u8,
    ) -> Option<Object<'next>> {
        let mut o = None;
        while let Some(obj) = self.next_object_by_type(ObjectType::PciDevice, o) {
            if let Some(Attributes::PciDev(attrs)) = obj.attributes() {
                if attrs.domain() == domain
                    && attrs.bus() == bus
                    && attrs.device_id() == dev
                    && attrs.func() == func
                {
                    return Some(obj);
                }
            }
            o.replace(obj);
        }
        None
    }

    // TODO(ckatsak): See include/hwloc/helper.h:1171
    //pub fn pcidev_by_busidstring<'topo: 'next, 'next>(
    //    &'topo self,
    //    busid: &'_ str,
    //) -> Result<Option<Object<'next>>, Error> {
    //    // Parse `domain`, `bus`, `dev` and `func` from the `busid` string as `xxxx:yy:zz.t` or
    //    // `yy:zz.t`
    //    let busid = busid.trim();
    //
    //    todo!()
    //}

    // TODO(ckatsak): UNTESTED
    /// Get the first non-I/O ancestor object.
    ///
    /// Given the I/O object `ioobj`, find the smallest non-I/O ancestor object. This object
    /// (normal or memory) may then be used for binding because it has non-NULL CPU and node sets
    /// and because its locality is the same as `ioobj`.
    ///
    /// # Note
    ///
    /// The resulting object is usually a normal object but it could also be a memory object (e.g.
    /// NUMA node) in future platforms if I/O objects ever get attached to memory instead of CPUs.
    pub fn non_io_ancestor_object(ioobj: Object<'_>) -> Option<Object<'_>> {
        let mut obj = Some(ioobj);
        while let Some(o) = obj {
            if o.cpuset().is_some() {
                return Some(o);
            }
            match o.parent() {
                Some(parent) => obj.replace(parent),
                None => break,
            };
        }
        None
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////

    /// Verify that the topology is compatible with the current hwloc library.
    ///
    /// This is useful when using the same topology structure (in memory) in different libraries
    /// that may use different hwloc installations (for instance if one library embeds a specific
    /// version of hwloc, while another library uses a default system-wide hwloc installation).
    ///
    /// If all libraries/programs use the same hwloc installation, this function always returns
    /// success.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologyAbiCheck`] if hwloc reports that the check failed.
    ///
    /// # Note
    ///
    /// If sharing between processes with `hwloc_shmem_topology_write()`, the relevant check is
    /// already performed inside `hwloc_shmem_topology_adopt()`. FIXME doclink
    ///
    /// [`Error::TopologyAbiCheck`]: crate::error::Error::TopologyAbiCheck
    pub fn abi_check(&self) -> Result<(), Error> {
        // SAFETY: `self.topo` is a valid topology object, created via a `TopologyBuilder`.
        match unsafe { hwloc2_sys::hwloc_topology_abi_check(self.topo) } {
            -1 => Err(Error::TopologyAbiCheck),
            _ => Ok(()),
        }
    }

    /// Does the topology context come from this system?
    pub fn is_this_system(&self) -> bool {
        // SAFETY: `self.topo` is a valid topology object, created via a `TopologyBuilder`.
        0 != unsafe { hwloc2_sys::hwloc_topology_is_thissystem(self.topo) }
    }

    // FIXME(ckatsak): This call aborts on failure. Should it be exposed?
    #[allow(dead_code)]
    fn check(&self) {
        unsafe { hwloc2_sys::hwloc_topology_check(self.topo) }
    }
}

impl Drop for Topology {
    fn drop(&mut self) {
        unsafe { hwloc2_sys::hwloc_topology_destroy(self.topo) }
    }
}

#[derive(Debug, Clone)]
pub struct TopologyBuilder {
    topo: *mut hwloc2_sys::hwloc_topology,

    // Used in `impl Drop` to make sure the new topology's context will not be freed after it has
    // been moved to the new `Topology` object.
    built: bool,
}

impl TopologyBuilder {
    /// Set OR'ed flags to non-yet-loaded topology.
    ///
    /// Set a OR'ed set of [`flags::Flags`] onto a topology that was not yet loaded.
    ///
    /// If this function is called multiple times, the last invocation will erase and replace the
    /// set of flags that was previously set.
    ///
    /// By default, no flags are set (`0`).
    ///
    /// The flags set in a topology may be retrieved with [`Topology::flags`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologyFlags`] if hwloc fails to set the given flags.
    ///
    /// [`Error::TopologyFlags`]: crate::error::Error::TopologyFlags
    pub fn flags(self, flags: flags::Flags) -> Result<Self, Error> {
        // SAFETY: `self.topo` is a valid topology object created via a `TopologyBuilder`, and
        // `flags` is type checked.
        match unsafe { hwloc2_sys::hwloc_topology_set_flags(self.topo, flags.bits()) } {
            -1 => Err(Error::TopologyFlags(flags)),
            _ => Ok(self),
        }
    }

    /// Set the filtering for the given object type.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologySetFilter`] if hwloc fails to set the given filter.
    ///
    /// [`Error::TopologySetFilter`]: crate::error::Error::TopologySetFilter
    pub fn type_filter(self, obj_type: ObjectType, filter: filters::Filter) -> Result<Self, Error> {
        // SAFETY: `self.topo` is a valid topology object created via a `TopologyBuilder`, and
        // `filter` & `obj_type` are type checked.
        match unsafe {
            hwloc2_sys::hwloc_topology_set_type_filter(self.topo, obj_type as u32, filter as u32)
        } {
            -1 => Err(Error::TopologySetFilter(obj_type, filter)),
            _ => Ok(self),
        }
    }

    /// Set the filtering for all object types.
    ///
    /// If some types do not support this filtering, they are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologySetAllTypesFilter`] if hwloc fails to set the given filter.
    ///
    /// [`Error::TopologySetAllTypesFilter`]: crate::error::Error::TopologySetAllTypesFilter
    pub fn all_types_filter(self, filter: filters::Filter) -> Result<Self, Error> {
        // SAFETY: `self.topo` is a valid topology object created via a `TopologyBuilder`, and
        // `filter` is type checked.
        match unsafe { hwloc2_sys::hwloc_topology_set_all_types_filter(self.topo, filter as u32) } {
            -1 => Err(Error::TopologySetAllTypesFilter(filter)),
            _ => Ok(self),
        }
    }

    /// Set the filtering for all CPU cache object types.
    ///
    /// Memory-side caches are not involved since they are not CPU caches.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologySetCacheTypesFilter`] if hwloc fails to set the given filter.
    ///
    /// [`Error::TopologySetCacheTypesFilter`]: crate::error::Error::TopologySetCacheTypesFilter
    pub fn cache_types_filter(self, filter: filters::Filter) -> Result<Self, Error> {
        // SAFETY: `self.topo` is a valid topology object created via a `TopologyBuilder`, and
        // `filter` is type checked.
        match unsafe { hwloc2_sys::hwloc_topology_set_cache_types_filter(self.topo, filter as u32) }
        {
            -1 => Err(Error::TopologySetCacheTypesFilter(filter)),
            _ => Ok(self),
        }
    }

    /// Set the filtering for all CPU instruction cache object types.
    ///
    /// Memory-side caches are not involved since they are not CPU caches.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologySetICacheTypesFilter`] if hwloc fails to set the given filter.
    ///
    /// [`Error::TopologySetICacheTypesFilter`]: crate::error::Error::TopologySetICacheTypesFilter
    pub fn icache_types_filter(self, filter: filters::Filter) -> Result<Self, Error> {
        // SAFETY: `self.topo` is a valid topology object created via a `TopologyBuilder`, and
        // `filter` is type checked.
        match unsafe {
            hwloc2_sys::hwloc_topology_set_icache_types_filter(self.topo, filter as u32)
        } {
            -1 => Err(Error::TopologySetICacheTypesFilter(filter)),
            _ => Ok(self),
        }
    }

    /// Set the filtering for all I/O object types.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologySetIOTypesFilter`] if hwloc fails to set the given filter.
    ///
    /// [`Error::TopologySetIOTypesFilter`]: crate::error::Error::TopologySetIOTypesFilter
    pub fn io_types_filter(self, filter: filters::Filter) -> Result<Self, Error> {
        // SAFETY: `self.topo` is a valid topology object created via a `TopologyBuilder`, and
        // `filter` is type checked.
        match unsafe { hwloc2_sys::hwloc_topology_set_io_types_filter(self.topo, filter as u32) } {
            -1 => Err(Error::TopologySetIOTypesFilter(filter)),
            _ => Ok(self),
        }
    }

    /// Consume this [`TopologyBuilder`] to create the new [`Topology`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::TopologyLoad`] if hwloc fails to complete the initialization of the new
    /// topology.
    ///
    /// [`Error::TopologyLoad`]: crate::error::Error::TopologyLoad
    pub fn build(mut self) -> Result<Topology, Error> {
        let support = Support::try_new(self.topo)?;

        // Build the actual topology object.
        // SAFETY: `topo` is a freshly allocated pointer, of the correct type, and a new topology
        // context must have been allocated successfully right above.
        if -1 == unsafe { hwloc2_sys::hwloc_topology_load(self.topo) } {
            return Err(Error::TopologyLoad);
        }

        self.built = true;
        Ok(Topology {
            topo: self.topo,
            support,
        })
    }
}

impl Drop for TopologyBuilder {
    fn drop(&mut self) {
        // Deallocate the topology context, unless its ownership has changed to a new `Topology`
        // object.
        if !self.built {
            // SAFETY: `self.topo` is a valid pointer of the correct type, which has remained
            // private since its creation. The underlying memory has not been freed before, since
            // only `TopologyBuilder`'s and `Topology`'s destructors may free it, and it has not
            // been moved to an instance of the latter (or `self.build` would be `true`).
            unsafe { hwloc2_sys::hwloc_topology_destroy(self.topo) };
        }
    }
}

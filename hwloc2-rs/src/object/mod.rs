pub mod attributes;

use std::{
    ffi::{CStr, CString},
    fmt,
    marker::PhantomData,
    ptr::addr_of,
};

use num_traits::FromPrimitive;

use crate::{
    bitmap::{Bitmap, CpuSet, NodeSet},
    ptr_mut_to_const, ObjectType,
};
use attributes::{BridgeAttributes, CacheAttributes, NumaNodeAttributes, PciDevAttributes};

#[derive(Clone, Copy)]
pub struct Object<'topo> {
    ptr: *const hwloc2_sys::hwloc_obj,
    _marker: PhantomData<&'topo hwloc2_sys::hwloc_obj>,
}

impl<'topo> Object<'topo> {
    /// The value returned by [`Object::os_index`] when it is unknown or irrelevant for the object.
    ///
    /// Originally defined in `include/hwloc.h` as:
    ///
    /// `# define HWLOC_UNKNOWN_INDEX (unsigned)-1`
    pub const UNKNOWN_INDEX: u32 = u32::MAX;

    /// Create a new `Object` from the provided pointer.
    ///
    /// # SAFETY
    ///
    /// This method does not check the validity of the given pointer, `ptr: *const hwloc_obj`; it
    /// is therefore the responsibility of the caller to make sure this is OK.
    pub(crate) unsafe fn new(ptr: *const hwloc2_sys::hwloc_obj) -> Object<'topo> {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Type of object.
    pub fn object_type(&self) -> ObjectType {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        ObjectType::from_u32(o.type_).expect("failed to cast u32 to ObjectType")
    }

    //pub fn subtype(&self) -> Option<&CStr> {
    //    // SAFETY: `self.0` is valid since it was created legitimately and kept private ever since
    //    let o = unsafe { *self.0 };
    //    if o.subtype.is_null() {
    //        return None;
    //    }
    //    // SAFETY: Since `o.subtype` != NULL, it should be a valid C string according to hwloc
    //    Some(unsafe { CStr::from_ptr(o.subtype as *const _) })
    //}
    /// Subtype string to better describe the type field.
    pub fn subtype(&self) -> Option<String> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.subtype.is_null() {
            return None;
        }
        // SAFETY: Since `o.subtype` != NULL, it should be a valid C string according to hwloc
        let cstring = unsafe { CString::from_raw(o.subtype) };
        cstring.to_str().ok().map(|s| s.to_owned())
    }

    /// OS-provided physical index number. It is not guaranteed unique across the entire machine,
    /// except for PUs and NUMA nodes. Set to [`Object::UNKNOWN_INDEX`] if unknown or irrelevant
    /// for this object.
    pub fn os_index(&self) -> u32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.os_index
    }

    /// Object-specific name if any. Mostly used for identifying OS devices and Misc objects where
    /// a name string is more useful than numerical indexes.
    pub fn name(&self) -> Option<String> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.name.is_null() {
            return None;
        }
        // SAFETY: Since `o.name` != NULL, it should be a valid C string according to hwloc
        let cstring = unsafe { CString::from_raw(o.name) };
        cstring.to_str().ok().map(|s| s.to_owned())
    }

    /// Total memory (in bytes) in NUMA nodes below this object.
    pub fn total_memory(&self) -> u64 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.total_memory
    }

    /// Object type-specific Attributes.
    pub fn attributes(&self) -> Option<Attributes> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        let union = ptr_mut_to_const(o.attr);
        if union.is_null() {
            return None;
        }

        use ObjectType::*;
        // Matching based on the switch in `hwloc_obj_attr_snprintf()`, found in file
        // `hwloc/traversal.c:566` (for hwloc-2.7.1).
        match self.object_type() {
            NumaNode => {
                // SAFETY: `o.attr` (i.e., `union`) has been checked to be non NULL
                let attrs = unsafe { NumaNodeAttributes::new(union) };
                Some(Attributes::NumaNode(attrs))
            }
            L1Cache | L2Cache | L3Cache | L4Cache | L5Cache | L1ICache | L2ICache | L3ICache
            | MemCache => {
                let attrs = unsafe { CacheAttributes::new(union) };
                Some(Attributes::Cache(attrs))
            }
            Bridge => {
                let attrs = unsafe { BridgeAttributes::new(union) };
                Some(Attributes::Bridge(attrs))
            }
            PciDevice => {
                // FIXME(ckatsak): BUG: something is probably accessed incorrectly, since
                // domain/bus/dev/func attributes of PCI devices appear to be wrong (when compared
                // to lstopo and lspci).
                let attrs = unsafe { (*union).pcidev };
                let attrs_ptr = addr_of!(attrs);
                let attrs = unsafe { PciDevAttributes::new(attrs_ptr) };
                Some(Attributes::PciDev(attrs))
            }
            _ => None,
        }
    }

    /// Object type-specific Attributes.
    pub fn attr(&self) -> *mut hwloc2_sys::hwloc_obj_attr_u {
        unsafe { *self.ptr }.attr
    }

    /// Vertical index in the hierarchy.
    ///
    /// For normal objects, this is the depth of the horizontal level that contains this object and
    /// its cousins of the same type. If the topology is symmetric, this is equal to the parent
    /// depth plus one, and also equal to the number of parent/child links from the root object to
    /// here.
    ///
    /// For special objects (NUMA nodes, I/O and Misc) that are not in the main tree, this is a
    /// special negative value that corresponds to their dedicated level, see
    /// [`Topology::type_depth`] and [`TypeDepth`]. Those special values can be passed to hwloc
    /// functions such [`Topology::nbobjs_by_depth`] as usual.
    ///
    /// [`Topology::type_depth`]: crate::topology::Topology::type_depth
    /// [`TypeDepth`]: crate::types::TypeDepth
    /// [`Topology::nbobjs_by_depth`]: crate::topology::Topology::nbobjs_by_depth
    pub fn depth(&self) -> i32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.depth
    }

    /// Horizontal index in the whole list of similar objects, hence guaranteed unique across the
    /// entire machine. Could be a "cousin_rank" since it's the rank within the "cousin" list
    /// below.
    ///
    /// Note that this index may change when restricting the topology or when inserting a group.
    pub fn logical_index(&self) -> u32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.logical_index
    }

    /// Next object of same type and depth.
    pub fn next_cousin(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.next_cousin.is_null() {
            return None;
        }
        Some(Self {
            ptr: ptr_mut_to_const(o.next_cousin),
            _marker: PhantomData,
        })
    }

    /// Previous object of same type and depth.
    pub fn prev_cousin(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        (!unsafe { *self.ptr }.prev_cousin.is_null()).then(|| Self {
            ptr: ptr_mut_to_const(unsafe { *self.ptr }.prev_cousin),
            _marker: PhantomData,
        })
    }

    /// Parent, `None` if root (i.e., Machine object).
    pub fn parent(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        (!o.parent.is_null()).then(|| Self {
            ptr: ptr_mut_to_const(o.parent),
            _marker: PhantomData,
        })
    }

    /// Index in parent's children array. Or the index in parent's Memory, I/O or Misc children
    /// list.
    pub fn sibling_rank(&self) -> u32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.sibling_rank
    }

    /// Next object below the same parent (inside the same list of children).
    pub fn next_sibling(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.next_sibling.is_null() {
            return None;
        };
        Some(Self {
            ptr: ptr_mut_to_const(o.next_sibling),
            _marker: PhantomData,
        })
    }

    /// Previous object below the same parent (inside the same list of children).
    pub fn prev_sibling(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.prev_sibling.is_null() {
            return None;
        };
        Some(Self {
            ptr: ptr_mut_to_const(o.prev_sibling),
            _marker: PhantomData,
        })
    }

    /// Set if the subtree of normal objects below this object is symmetric, which means all normal
    /// children and their children have identical subtrees.
    ///
    /// Memory, I/O and Misc children are ignored.
    ///
    /// If set in the topology root object, lstopo may export the topology as a synthetic string.
    pub fn symmetric_subtree(&self) -> bool {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        0 != unsafe { *self.ptr }.symmetric_subtree
    }

    /// TODO: UNTESTED
    ///
    /// CPUs covered by this object.
    ///
    /// This is the set of CPUs for which there are PU objects in the topology under this object,
    /// i.e. which are known to be physically contained in this object and known how (the children
    /// path between this object and the PU objects).
    ///
    /// If the [`Flags::INCLUDE_DISALLOWED`] configuration flag is set, some of these CPUs may be
    /// online but not allowed for binding, see `hwloc_topology_get_allowed_cpuset()`.
    ///
    /// # Notes
    ///
    /// - All objects have non-NULL CPU and node sets except Misc and I/O objects.
    /// - Its value must not be changed, [`Bitmap::clone`] must be used instead.
    ///
    /// [`Flags::INCLUDE_DISALLOWED`]: crate::topology::flags::Flags::INCLUDE_DISALLOWED
    /// [`Bitmap::clone`]: crate::bitmap::Bitmap::clone
    pub fn cpuset(&self) -> Option<CpuSet> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { Bitmap::from_raw((*self.ptr).cpuset, false) }.ok()
    }

    /// TODO: UNTESTED
    ///
    /// The complete CPU set of processors of this object.
    ///
    /// This may include not only the same as the [`Object::cpuset`] field, but also some CPUs for
    /// which topology information is unknown or incomplete, some offlines CPUs, and the CPUs that
    /// are ignored when the [`Flags::INCLUDE_DISALLOWED`] flag is not set. Thus no corresponding
    /// PU object may be found in the topology, because the precise position is undefined. It is
    /// however known that it would be somewhere under this object.
    ///
    /// # Note
    ///
    /// Its value must not be changed, [`Bitmap::clone`] must be used instead.
    ///
    /// [`Flags::INCLUDE_DISALLOWED`]: crate::topology::flags::Flags::INCLUDE_DISALLOWED
    /// [`Bitmap::clone`]: crate::bitmap::Bitmap::clone
    pub fn complete_cpuset(&self) -> Option<CpuSet> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { Bitmap::from_raw((*self.ptr).complete_cpuset, false) }.ok()
    }

    /// TODO: UNTESTED
    ///
    /// NUMA nodes covered by this object or containing this object.
    ///
    /// This is the set of NUMA nodes for which there are NUMA node objects in the topology under
    /// or above this object, i.e. which are known to be physically contained in this object or
    /// containing it and known how (the children path between this object and the NUMA node
    /// objects).
    ///
    /// In the end, these nodes are those that are close to the current object. Function
    /// `hwloc_get_local_numanode_objs()` may be used to list those NUMA nodes more precisely.
    // FIXME: doclink
    ///
    /// If the [`Flags::INCLUDE_DISALLOWED`] configuration flag is set, some of these nodes may be
    /// online but not allowed for allocation, see `hwloc_topology_get_allowed_nodeset()`.
    // FIXME: doclink
    ///
    /// If there are no NUMA nodes in the machine, all the memory is close to this object, so only
    /// the first bit may be set in nodeset.
    ///
    /// # Note
    ///
    /// - All objects have non-NULL CPU and node sets except Misc and I/O objects.
    /// - Its value must not be changed, [`Bitmap::clone`] must be used instead.
    ///
    /// [`Flags::INCLUDE_DISALLOWED`]: crate::topology::flags::Flags::INCLUDE_DISALLOWED
    /// [`Bitmap::clone`]: crate::bitmap::Bitmap::clone
    pub fn nodeset(&self) -> Option<NodeSet> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { Bitmap::from_raw((*self.ptr).nodeset, false) }.ok()
    }

    /// TODO: UNTESTED
    ///
    /// The complete NUMA node set of this object.
    ///
    /// This may include not only the same as the [`Object::nodeset`] field, but also some NUMA
    /// nodes for which topology information is unknown or incomplete, some offlines nodes, and the
    /// nodes that are ignored when the [`Flags::INCLUDE_DISALLOWED`] flag is not set. Thus no
    /// corresponding NUMA node object may be found in the topology, because the precise position
    /// is undefined. It is however known that it would be somewhere under this object.
    ///
    /// If there are no NUMA nodes in the machine, all the memory is close to this object, so only
    /// the first bit is set in complete_nodeset.
    ///
    /// # Note
    ///
    /// - Its value must not be changed, [`Bitmap::clone`] must be used instead.
    ///
    /// [`Flags::INCLUDE_DISALLOWED`]: crate::topology::flags::Flags::INCLUDE_DISALLOWED
    /// [`Bitmap::clone`]: crate::bitmap::Bitmap::clone
    pub fn complete_nodeset(&self) -> Option<NodeSet> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { Bitmap::from_raw((*self.ptr).complete_nodeset, false) }.ok()
    }

    /// Array of stringified info type=name.
    ///
    /// # Note
    ///
    /// This is merely an accessor method for the underlying pointer; no "convenient" API offered,
    /// for now.
    pub fn infos(&self) -> *mut hwloc2_sys::hwloc_info_s {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.infos
    }

    /// Size of [`Object::infos`] array (in C).
    pub fn infos_count(&self) -> u32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.infos_count
    }

    /// Global persistent index. Generated by `hwloc`, unique across the topology (contrary to
    /// [`Object::os_index`]) and persistent across topology changes (contrary to
    /// [`Object::logical_index`]). Mostly used internally, but could also be used by application
    /// to identify objects.
    pub fn gp_index(&self) -> u64 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.gp_index
    }

    //
    // List and array of normal children below this object (except Memory, I/O and Misc children).
    //

    /// Number of normal children. Memory, Misc and I/O children are not listed here but rather in
    /// their dedicated children list.
    pub fn arity(&self) -> u32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.arity
    }

    /// Normal children, `children[0 .. arity-1]`.
    ///
    /// # Panics
    ///
    /// If the underlying `hwloc2_sys::hwloc_obj`'s `children` pointer is `NULL`, or if one of the
    /// pointers in this `children` array is `NULL` while it should not.
    pub fn children(&self) -> Vec<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        // XXX(ckatsak): An `Object` with `self.arity() == 0` might still call this function. For
        // now, an empty `Vec<Object<'topo>>` is returned, but maybe it should be changed to
        // `None`, thus modifying the return type to `Option<Vec<Object<'topo>>>`:
        //if o.children.is_null() {
        //    return None;
        //}
        (0..self.arity())
            .map(|i| {
                // SAFETY: Taking into account the safety notice above, having asserted that the
                // children pointer is not NULL, and assuming hwloc's memory is not corrupted (due
                // to some internal bug, or my fault?), it should be safe to dereference the
                // children pointer to access the `arity`-sized array. We then assert the retrieved
                // pointer is non-NULL too, before creating each new `Object`.
                let ptr = ptr_mut_to_const(unsafe { *o.children.offset(i as isize) });
                assert!(!ptr.is_null());
                Self {
                    ptr,
                    _marker: PhantomData,
                }
            })
            .collect()
    }

    /// First normal child.
    pub fn first_child(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.first_child.is_null() {
            return None;
        };
        Some(Self {
            ptr: ptr_mut_to_const(o.first_child),
            _marker: PhantomData,
        })
    }

    /// Last normal child.
    pub fn last_child(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.last_child.is_null() {
            return None;
        };
        Some(Self {
            ptr: ptr_mut_to_const(o.last_child),
            _marker: PhantomData,
        })
    }

    //
    // List of Memory children below this object.
    //

    /// Number of Memory children. These children are listed in [`Object::memory_first_child`].
    pub fn memory_arity(&self) -> u32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.memory_arity
    }

    /// First Memory child. NUMA nodes and Memory-side caches are listed here
    /// ([`Object::memory_arity`] and [`Object::memory_first_child`]) instead of in the normal
    /// children list. See also [`ObjectType::is_memory`].
    ///
    /// A memory hierarchy starts from a normal CPU-side object (e.g. Package) and ends with NUMA
    /// nodes as leaves. There might exist some memory-side caches between them in the middle of
    /// the memory subtree.
    ///
    /// [`ObjectType::is_memory`]: crate::types::ObjectType::is_memory
    pub fn memory_first_child(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.memory_first_child.is_null() {
            return None;
        };
        Some(Self {
            ptr: ptr_mut_to_const(o.memory_first_child),
            _marker: PhantomData,
        })
    }

    //
    // List of I/O children below this object.
    //

    /// Number of I/O children. These children are listed in io_first_child.
    pub fn io_arity(&self) -> u32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.io_arity
    }

    /// First I/O child. Bridges, PCI and OS devices are listed here ([`Object::io_arity`] and
    /// [`Object::io_first_child`]) instead of in the normal children list. See also
    /// [`ObjectType::is_io`].
    ///
    /// [`ObjectType::is_io`]: crate::types::ObjectType::is_io
    pub fn io_first_child(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.io_first_child.is_null() {
            return None;
        };
        Some(Self {
            ptr: ptr_mut_to_const(o.io_first_child),
            _marker: PhantomData,
        })
    }

    //
    // List of Misc children below this object.
    //

    /// Number of Misc children. These children are listed in [`Object::misc_first_child`].
    pub fn misc_arity(&self) -> u32 {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        unsafe { *self.ptr }.misc_arity
    }

    /// First Misc child. Misc objects are listed here ([`Object::misc_arity`] and
    /// [`Object::misc_first_child`]) instead of in the normal children list.
    pub fn misc_first_child(&self) -> Option<Object<'topo>> {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        let o = unsafe { *self.ptr };
        if o.misc_first_child.is_null() {
            return None;
        };
        Some(Self {
            ptr: ptr_mut_to_const(o.misc_first_child),
            _marker: PhantomData,
        })
    }
}

impl<'topo> fmt::Display for Object<'topo> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf_type = [0; 64];
        let mut buf_attr = [0; 2048];

        let sep_ptr = b"  \0".as_ptr() as *const ::std::os::raw::c_char;

        // SAFETY: Both buffers have been just allocated, their lengths are correctly passed, and
        // their returned value is checked for errors; therefore it is up to hwloc to treat them
        // correctly, I guess.
        unsafe {
            if hwloc2_sys::hwloc_obj_type_snprintf(
                buf_type.as_mut_ptr(),
                buf_type.len() as u64,
                self.ptr as *mut _,
                0,
            ) == -1
            {
                return Err(fmt::Error);
            }
            if hwloc2_sys::hwloc_obj_attr_snprintf(
                buf_attr.as_mut_ptr(),
                buf_attr.len() as u64,
                self.ptr as *mut _,
                sep_ptr,
                0,
            ) == -1
            {
                return Err(fmt::Error);
            }
        }

        unsafe {
            write!(
                f,
                "{} ({})",
                CStr::from_ptr(buf_type.as_ptr()).to_str().unwrap(),
                CStr::from_ptr(buf_attr.as_ptr()).to_str().unwrap(),
            )
        }
    }
}

impl<'topo> fmt::Debug for Object<'topo> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: `self.ptr` can be safely dereferenced because it was created either via `new()`
        // or based on another (valid) `Object`'s (valid) pointer, and remained private ever since.
        write!(f, "Object({:p}){{ ", self.ptr)?;
        write!(f, "type: {:?}, ", self.object_type())?;
        write!(f, "subtype: {:?}, ", self.subtype())?;
        write!(f, "os_index: {}, ", self.os_index())?;
        write!(f, "total_memory: {}, ", self.total_memory())?;
        write!(f, "attr: {:?}, ", self.attributes())?; // FIXME(ckatsak): BUG in PCI ?
        write!(f, "depth: {}, ", self.depth())?;
        write!(f, "logical_index: {}, ", self.logical_index())?;
        write!(f, "next_cousin: {:p}, ", unsafe { (*self.ptr).next_cousin })?;
        write!(f, "prev_cousin: {:p}, ", unsafe { (*self.ptr).prev_cousin })?;
        write!(f, "parent: {:p}, ", unsafe { (*self.ptr).parent })?;
        write!(f, "sibling_rank: {}, ", self.sibling_rank())?;
        write!(f, "next_sibling: {:p}, ", unsafe {
            (*self.ptr).next_sibling
        })?;
        write!(f, "prev_sibling: {:p}, ", unsafe {
            (*self.ptr).prev_sibling
        })?;
        write!(f, "symmetric_subtree: {}, ", self.symmetric_subtree())?;
        write!(f, "cpuset: {:?}, ", self.cpuset())?;
        write!(f, "complete_cpuset: {:?}, ", self.complete_cpuset())?;
        write!(f, "nodeset: {:?}, ", self.nodeset())?;
        write!(f, "complete_nodeset: {:?}, ", self.complete_nodeset())?;
        write!(f, "infos: {:p}, ", self.infos())?;
        write!(f, "infos_count: {}, ", self.infos_count())?;
        write!(f, "gp_index: {}, ", self.gp_index())?;

        write!(f, "arity: {}, ", self.arity())?;
        write!(f, "children: vec.len={}, ", self.children().len())?;
        write!(f, "first_child: {:p}, ", unsafe { (*self.ptr).first_child })?;
        write!(f, "last_child: {:p}, ", unsafe { (*self.ptr).last_child })?;

        write!(f, "memory_arity: {}, ", self.memory_arity())?;
        write!(f, "memory_first_child: {:p}, ", unsafe {
            (*self.ptr).memory_first_child
        })?;

        write!(f, "io_arity: {}, ", self.io_arity())?;
        write!(f, "io_first_child: {:p}, ", unsafe {
            (*self.ptr).io_first_child
        })?;

        write!(f, "misc_arity: {}, ", self.misc_arity())?;
        write!(f, "misc_first_child: {:p} ", unsafe {
            (*self.ptr).misc_first_child
        })?;
        write!(f, "}}")
    }
}

/// Object type-specific Attributes.
#[derive(Debug, Clone, Copy)]
pub enum Attributes<'topo> {
    NumaNode(NumaNodeAttributes<'topo>),
    Cache(CacheAttributes<'topo>),
    PciDev(PciDevAttributes<'topo>),
    Bridge(BridgeAttributes<'topo>),
}

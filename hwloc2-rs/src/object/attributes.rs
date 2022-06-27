use std::{fmt, marker::PhantomData, ptr::addr_of};

use num_traits::FromPrimitive;

use crate::types::{BridgeType, CacheType};

/// NUMA node-specific Object Attributes.
#[derive(Clone, Copy)]
pub struct NumaNodeAttributes<'topo> {
    ptr: *const hwloc2_sys::hwloc_obj_attr_u,
    _marker: PhantomData<&'topo hwloc2_sys::hwloc_obj_attr_u>,
}

impl<'topo> NumaNodeAttributes<'topo> {
    /// Create a new NumaNodeAttributes.
    ///
    /// # Safety
    ///
    /// The given pointer `ptr` is assumed to be valid, and is not checked. It is the
    /// responsibility of the caller to make sure it is not NULL.
    pub(super) unsafe fn new(ptr: *const hwloc2_sys::hwloc_obj_attr_u) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Local memory (in bytes).
    pub fn local_memory(&self) -> u64 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.numanode`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_numanode_attr_s` as the former is `repr(C)`.
        unsafe { (*self.ptr).numanode }.local_memory
    }

    /// Size of array `page_types` (in C).
    pub fn page_types_len(&self) -> u32 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.numanode`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_numanode_attr_s` as the former is `repr(C)`.
        unsafe { (*self.ptr).numanode }.page_types_len
    }

    /// Array of local memory page types, `None` if no local memory and `page_types` is `0`.
    ///
    /// The array is sorted by increasing size fields. It contains `page_types_len` slots.
    pub fn page_types(&self) -> Option<Vec<PageType>> {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.numanode`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_numanode_attr_s` as the former is `repr(C)`.
        let arr_base = unsafe { (*self.ptr).numanode }.page_types;
        if arr_base.is_null() {
            return None;
        }
        Some(
            (0..self.page_types_len())
                .filter_map(|i| {
                    // SAFETY: We checked that `arr_base != NULL`, hence safe to dereference.
                    let pt_addr = unsafe { arr_base.offset(i as isize) };
                    if pt_addr.is_null() {
                        None
                    } else {
                        // SAFETY: We checked that `pt_addr != NULL`, hence safe to dereference.
                        let pt = unsafe { *pt_addr };
                        Some(PageType {
                            size: pt.size,
                            count: pt.count,
                        })
                    }
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageType {
    /// Size of pages.
    pub size: u64,
    /// Number of pages of this size.
    pub count: u64,
}

impl<'topo> fmt::Debug for NumaNodeAttributes<'topo> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NumaNodeAttributes{{ ")?;
        write!(f, "local_memory: {}, ", self.local_memory())?;
        write!(f, "page_types_len: {}, ", self.page_types_len())?;
        write!(f, "page_types: {:?} ", self.page_types())?;
        write!(f, "}}")
    }
}

/// Cache-specific Object Attributes.
#[derive(Clone, Copy)]
pub struct CacheAttributes<'topo> {
    ptr: *const hwloc2_sys::hwloc_obj_attr_u,
    _marker: PhantomData<&'topo hwloc2_sys::hwloc_obj_attr_u>,
}

impl<'topo> CacheAttributes<'topo> {
    /// Create a new CacheAttributes.
    ///
    /// # Safety
    ///
    /// The given pointer `ptr` is assumed to be valid, and is not checked. It is the
    /// responsibility of the caller to make sure it is not NULL.
    pub(super) unsafe fn new(ptr: *const hwloc2_sys::hwloc_obj_attr_u) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Size of cache in bytes.
    pub fn size(&self) -> u64 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.cache`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_cache_attr_s` as the former is `repr(C)`.
        unsafe { (*self.ptr).cache }.size
    }

    /// Depth of cache (e.g., L1, L2, ...etc.)
    pub fn depth(&self) -> u32 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.cache`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_cache_attr_s` as the former is `repr(C)`.
        unsafe { (*self.ptr).cache }.depth
    }

    /// Cache-line size in bytes. `0` if unknown.
    pub fn linesize(&self) -> u32 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.cache`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_cache_attr_s` as the former is `repr(C)`.
        unsafe { (*self.ptr).cache }.linesize
    }

    /// Ways of associativity, `-1` if fully associative, `0` if unknown.
    pub fn associativity(&self) -> i32 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.cache`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_cache_attr_s` as the former is `repr(C)`.
        unsafe { (*self.ptr).cache }.associativity
    }

    /// Cache type.
    pub fn cache_type(&self) -> CacheType {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.cache`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_cache_attr_s` as the former is `repr(C)`.
        CacheType::from_u32(unsafe { (*self.ptr).cache }.type_)
            .expect("failed to cast u32 to CacheType")
    }
}

impl<'topo> fmt::Debug for CacheAttributes<'topo> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CacheAttributes{{ ")?;
        write!(f, "size: {}, ", self.size())?;
        write!(f, "depth: {}, ", self.depth())?;
        write!(f, "linesize: {}, ", self.linesize())?;
        write!(f, "associativity: {}, ", self.associativity())?;
        write!(f, "cache_type: {:?} ", self.cache_type())?;
        write!(f, "}}")
    }
}

// FIXME(ckatsak): BUG: something is probably accessed incorrectly, since domain/bus/dev/func
// attributes of PCI devices appear to be wrong (when compared to lstopo and lspci).
/// PCI Device specific Object Attributes.
///
/// # Note
///
/// Current bindings have been created for 16bits PCI domain -- `hwloc`'s default.
#[derive(Clone, Copy)]
pub struct PciDevAttributes<'topo> {
    ptr: *const hwloc2_sys::hwloc_obj_attr_u_hwloc_pcidev_attr_s,
    _marker: PhantomData<&'topo hwloc2_sys::hwloc_obj_attr_u_hwloc_pcidev_attr_s>,
}

impl<'topo> PciDevAttributes<'topo> {
    /// Create a new PciDevAttributes.
    ///
    /// # Safety
    ///
    /// The given pointer `ptr` is assumed to be valid, and is not checked. It is the
    /// responsibility of the caller to make sure it is not NULL.
    pub(super) unsafe fn new(ptr: *const hwloc2_sys::hwloc_obj_attr_u_hwloc_pcidev_attr_s) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn domain(&self) -> u16 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.domain`: TODO
        unsafe { *self.ptr }.domain
    }

    pub fn bus(&self) -> u8 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.bus`: TODO
        unsafe { *self.ptr }.bus
    }

    pub fn func(&self) -> u8 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.func`: TODO
        unsafe { *self.ptr }.func
    }

    pub fn class_id(&self) -> u16 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.class_id`: TODO
        unsafe { *self.ptr }.class_id
    }

    pub fn vendor_id(&self) -> u16 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.vendor_id`: TODO
        unsafe { *self.ptr }.vendor_id
    }

    pub fn device_id(&self) -> u16 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.device_id`: TODO
        unsafe { *self.ptr }.device_id
    }

    pub fn subvendor_id(&self) -> u16 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.subvendor_id`: TODO
        unsafe { *self.ptr }.subvendor_id
    }

    pub fn subdevice_id(&self) -> u16 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.subdevice_id`: TODO
        unsafe { *self.ptr }.subdevice_id
    }

    pub fn revision(&self) -> u8 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.revision`: TODO
        unsafe { *self.ptr }.revision
    }

    pub fn linkspeed(&self) -> f32 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.linkspeed`: TODO
        unsafe { *self.ptr }.linkspeed
    }
}

impl<'topo> fmt::Debug for PciDevAttributes<'topo> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PciDevAttributes{{ ")?;
        write!(f, "domain: 0x{:x}, ", self.domain())?;
        write!(f, "bus: 0x{:x}, ", self.bus())?;
        write!(f, "func: 0x{:x}, ", self.func())?;
        write!(f, "class_id: 0x{:x}, ", self.class_id())?;
        write!(f, "vendor_id: 0x{:x}, ", self.vendor_id())?;
        write!(f, "device_id: 0x{:x}, ", self.device_id())?;
        write!(f, "subvendor_id: 0x{:x}, ", self.subvendor_id())?;
        write!(f, "subdevice_id: 0x{:x}, ", self.subdevice_id())?;
        write!(f, "revision: 0x{:x}, ", self.revision())?;
        write!(f, "linkspeed: {} ", self.linkspeed())?;
        write!(f, "}}")
    }
}

/// Bridge specific Object Attributes.
///
/// # Note
///
/// Current bindings have been created for 16-bit PCI domain -- `hwloc`'s default.
#[derive(Clone, Copy)]
pub struct BridgeAttributes<'topo> {
    ptr: *const hwloc2_sys::hwloc_obj_attr_u,
    _marker: PhantomData<&'topo hwloc2_sys::hwloc_obj_attr_u>,
}

impl<'topo> BridgeAttributes<'topo> {
    /// Create a new BridgeAttributes.
    ///
    /// # Safety
    ///
    /// The given pointer `ptr` is assumed to be valid, and is not checked. It is the
    /// responsibility of the caller to make sure it is not NULL.
    pub(super) unsafe fn new(ptr: *const hwloc2_sys::hwloc_obj_attr_u) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn depth(&self) -> u32 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.bridge`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_bridge_attr_s` as the former is `repr(C)`.
        unsafe { (*self.ptr).bridge }.depth
    }

    /// TODO
    ///
    /// # Panics
    ///
    /// If the `u32` retrieved from `hwloc` cannot be casted to [`BridgeType`].
    ///
    /// [`BridgeType`]: crate::types::BridgeType
    pub fn upstream_type(&self) -> BridgeType {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.bridge`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_bridge_attr_s` as the former is `repr(C)`.
        BridgeType::from_u32(unsafe { (*self.ptr).bridge }.upstream_type)
            .expect("failed to cast u32 to BridgeType")
    }

    pub fn upstream(&self) -> PciDevAttributes {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.bridge`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_bridge_attr_s` as the former is `repr(C)`.
        let upstream = unsafe { (*self.ptr).bridge }.upstream;

        // SAFETY: TODO
        let upstream = unsafe { upstream.pci };

        // SAFETY: TODO
        unsafe { PciDevAttributes::new(addr_of!(upstream)) }
    }

    pub fn downstream_type(&self) -> BridgeType {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.bridge`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_bridge_attr_s` as the former is `repr(C)`.
        BridgeType::from_u32(unsafe { (*self.ptr).bridge }.downstream_type)
            .expect("failed to cast u32 to BridgeType")
    }

    pub fn downstream_domain(&self) -> u16 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.bridge`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_bridge_attr_s` as the former is `repr(C)`.
        let downstream = unsafe { (*self.ptr).bridge }.downstream;

        // SAFETY: TODO
        let downstream_pci = unsafe { downstream.pci };

        downstream_pci.domain
    }

    pub fn downstream_secondary_bus(&self) -> u8 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.bridge`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_bridge_attr_s` as the former is `repr(C)`.
        let downstream = unsafe { (*self.ptr).bridge }.downstream;

        // SAFETY: TODO
        let downstream_pci = unsafe { downstream.pci };

        downstream_pci.secondary_bus
    }

    pub fn downstream_subordinate_bus(&self) -> u8 {
        // SAFETY:
        // - Dereferencing `self.ptr`: it can be safely dereferenced because it was created via
        // `new()` by some `Object`, and remained private (i.e., unmodified) ever since.
        // - Accessing union field `.bridge`: casting `*mut hwloc_obj_attr_u` to
        // `*mut hwloc_obj_attr_u_hwloc_bridge_attr_s` as the former is `repr(C)`.
        let downstream = unsafe { (*self.ptr).bridge }.downstream;

        // SAFETY: TODO
        let downstream_pci = unsafe { downstream.pci };

        downstream_pci.subordinate_bus
    }
}

impl<'topo> fmt::Debug for BridgeAttributes<'topo> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BridgeAttributes{{ ")?;
        write!(f, "depth: {}, ", self.depth())?;
        write!(f, "upstream_type: {:?}, ", self.upstream_type())?;
        write!(f, "upstream: {:?}, ", self.upstream())?;
        write!(f, "downstream_type: {:?}, ", self.downstream_type())?;
        write!(f, "downstream_domain: 0x{:x?}, ", self.downstream_domain())?;
        write!(
            f,
            "downstream_secondary_bus: 0x{:x?}, ",
            self.downstream_secondary_bus()
        )?;
        write!(
            f,
            "downstream_subordinate_bus: 0x{:x?}, ",
            self.downstream_subordinate_bus()
        )?;
        write!(f, "}}")
    }
}

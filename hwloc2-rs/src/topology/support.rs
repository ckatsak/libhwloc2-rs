use std::fmt;

use crate::Error;

/// Set of flags describing actual support for this topology.
///
/// This is retrieved with [`Topology::support`] and will be valid until the topology object is
/// destroyed.
///
/// # Note
///
/// The values are correct only after discovery.
///
/// [`Topology::support`]: crate::topology::Topology::support
pub struct Support {
    discovery: Discovery,
    cpubind: Cpubind,
    membind: Membind,
    misc: Misc,
}

impl fmt::Debug for Support {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Support{{ {:?}, {:?}, {:?}, {:?} }}",
            self.discovery, self.cpubind, self.membind, self.misc
        )
    }
}

impl Support {
    pub(crate) fn try_new(topo: *mut hwloc2_sys::hwloc_topology) -> Result<Self, Error> {
        let support = unsafe { hwloc2_sys::hwloc_topology_get_support(topo) };
        if support.is_null()
            || unsafe { *support }.discovery.is_null()
            || unsafe { *support }.cpubind.is_null()
            || unsafe { *support }.membind.is_null()
            || unsafe { *support }.misc.is_null()
        {
            return Err(Error::TopologySupport);
        }
        Ok(Self {
            discovery: Discovery(unsafe { *support }.discovery),
            cpubind: Cpubind(unsafe { *support }.cpubind),
            membind: Membind(unsafe { *support }.membind),
            misc: Misc(unsafe { *support }.misc),
        })
    }

    /// Flags describing actual discovery support for this topology.
    pub fn discovery(&self) -> &Discovery {
        &self.discovery
    }

    /// Flags describing actual PU binding support for this topology.
    ///
    /// A flag may be set even if the feature isn't supported in all cases (e.g. binding to random
    /// sets of non-contiguous objects).
    pub fn cpubind(&self) -> &Cpubind {
        &self.cpubind
    }

    /// Flags describing actual memory binding support for this topology.
    ///
    /// A flag may be set even if the feature isn't supported in all cases (e.g. binding to random
    /// sets of non-contiguous objects).
    pub fn membind(&self) -> &Membind {
        &self.membind
    }

    /// Flags describing miscellaneous features.
    pub fn misc(&self) -> &Misc {
        &self.misc
    }
}

/// Flags describing actual discovery support for this topology.
pub struct Discovery(*const hwloc2_sys::hwloc_topology_discovery_support);

impl Discovery {
    /// Detecting the number of PU objects is supported.
    pub fn pu(&self) -> bool {
        unsafe { 0 != (*self.0).pu }
    }

    /// Detecting the number of NUMA nodes is supported.
    pub fn numa(&self) -> bool {
        unsafe { 0 != (*self.0).numa }
    }

    /// Detecting the amount of memory in NUMA nodes is supported.
    pub fn numa_memory(&self) -> bool {
        unsafe { 0 != (*self.0).numa_memory }
    }

    /// Detecting and identifying PU objects that are not available to the current process is
    /// supported.
    pub fn disallowed_pu(&self) -> bool {
        unsafe { 0 != (*self.0).disallowed_pu }
    }

    /// Detecting and identifying NUMA nodes that are not available to the current process is
    /// supported.
    pub fn disallowed_numa(&self) -> bool {
        unsafe { 0 != (*self.0).disallowed_numa }
    }

    /// Detecting the efficiency of CPU kinds is supported, see
    /// [Kinds of CPU cores](https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00190.php).
    pub fn cpukind_efficiency(&self) -> bool {
        unsafe { 0 != (*self.0).cpukind_efficiency }
    }
}

impl fmt::Debug for Discovery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_null() {
            return Err(fmt::Error);
        }
        let d = unsafe { *self.0 };
        write!(f, "Discovery{{ ")?;
        write!(f, "pu: {}, ", d.pu)?;
        write!(f, "numa: {}, ", d.numa)?;
        write!(f, "numa_memory: {}, ", d.numa_memory)?;
        write!(f, "disallowed_pu: {}, ", d.disallowed_pu)?;
        write!(f, "disallowed_numa: {}, ", d.disallowed_numa)?;
        write!(f, "cpukind_efficiency: {} ", d.cpukind_efficiency)?;
        write!(f, "}}")
    }
}

/// Flags describing actual PU binding support for this topology.
///
/// A flag may be set even if the feature isn't supported in all cases (e.g. binding to random sets
/// of non-contiguous objects).
pub struct Cpubind(*const hwloc2_sys::hwloc_topology_cpubind_support);

impl Cpubind {
    /// Binding the whole current process is supported.
    pub fn set_thisproc_cpubind(&self) -> bool {
        unsafe { 0 != (*self.0).set_thisproc_cpubind }
    }

    /// Getting the binding of the whole current process is supported.
    pub fn get_thisproc_cpubind(&self) -> bool {
        unsafe { 0 != (*self.0).get_thisproc_cpubind }
    }

    /// Binding a whole given process is supported.
    pub fn set_proc_cpubind(&self) -> bool {
        unsafe { 0 != (*self.0).set_proc_cpubind }
    }

    /// Getting the binding of a whole given process is supported.
    pub fn get_proc_cpubind(&self) -> bool {
        unsafe { 0 != (*self.0).get_proc_cpubind }
    }

    /// Binding the current thread only is supported.
    pub fn set_thisthread_cpubind(&self) -> bool {
        unsafe { 0 != (*self.0).set_thisthread_cpubind }
    }

    /// Getting the binding of the current thread only is supported.
    pub fn get_thisthread_cpubind(&self) -> bool {
        unsafe { 0 != (*self.0).get_thisproc_cpubind }
    }

    /// Binding a given thread only is supported.
    pub fn set_thread_cpubind(&self) -> bool {
        unsafe { 0 != (*self.0).set_thread_cpubind }
    }

    /// Getting the binding of a given thread only is supported.
    pub fn get_thread_cpubind(&self) -> bool {
        unsafe { 0 != (*self.0).get_thread_cpubind }
    }

    /// Getting the last processors where the whole current process ran is supported.
    pub fn get_thisproc_last_cpu_location(&self) -> bool {
        unsafe { 0 != (*self.0).get_thisproc_last_cpu_location }
    }

    /// Getting the last processors where a whole process ran is supported.
    pub fn get_proc_last_cpu_location(&self) -> bool {
        unsafe { 0 != (*self.0).get_proc_last_cpu_location }
    }

    /// Getting the last processors where the current thread ran is supported
    pub fn get_thisthread_last_cpu_location(&self) -> bool {
        unsafe { 0 != (*self.0).get_thisthread_last_cpu_location }
    }
}

impl fmt::Debug for Cpubind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_null() {
            return Err(fmt::Error);
        }
        let c = unsafe { *self.0 };
        write!(f, "Cpubind{{ ")?;
        write!(f, "set_thisproc_cpubind: {}, ", c.set_thisproc_cpubind)?;
        write!(f, "get_thisproc_cpubind: {}, ", c.get_thisproc_cpubind)?;
        write!(f, "set_proc_cpubind: {}, ", c.set_proc_cpubind)?;
        write!(f, "get_proc_cpubind: {}, ", c.get_proc_cpubind)?;
        write!(f, "set_thisthread_cpubind: {}, ", c.set_thisthread_cpubind)?;
        write!(f, "get_thisthread_cpubind: {}, ", c.get_thisthread_cpubind)?;
        write!(f, "set_thread_cpubind: {}, ", c.set_thread_cpubind)?;
        write!(f, "get_thread_cpubind: {}, ", c.get_thread_cpubind)?;
        write!(
            f,
            "get_thisproc_last_cpu_location: {}, ",
            c.get_thisproc_last_cpu_location
        )?;
        write!(
            f,
            "get_proc_last_cpu_location: {}, ",
            c.get_proc_last_cpu_location
        )?;
        write!(
            f,
            "get_thisthread_last_cpu_location: {} ",
            c.get_thisthread_last_cpu_location
        )?;
        write!(f, "}}")
    }
}

/// Flags describing actual memory binding support for this topology.
///
/// A flag may be set even if the feature isn't supported in all cases (e.g. binding to random sets
/// of non-contiguous objects).
pub struct Membind(*const hwloc2_sys::hwloc_topology_membind_support);

impl Membind {
    /// Binding the whole current process is supported.
    pub fn set_thisproc_membind(&self) -> bool {
        unsafe { 0 != (*self.0).set_thisproc_membind }
    }

    /// Getting the binding of the whole current process is supported.
    pub fn get_thisproc_membind(&self) -> bool {
        unsafe { 0 != (*self.0).get_thisproc_membind }
    }

    /// Binding a whole given process is supported.
    pub fn set_proc_membind(&self) -> bool {
        unsafe { 0 != (*self.0).set_proc_membind }
    }

    /// Getting the binding of a whole given process is supported.
    pub fn get_proc_membind(&self) -> bool {
        unsafe { 0 != (*self.0).get_proc_membind }
    }

    /// Binding the current thread only is supported.
    pub fn set_thisthread_membind(&self) -> bool {
        unsafe { 0 != (*self.0).set_thisthread_membind }
    }

    /// Getting the binding of the current thread only is supported.
    pub fn get_thisthread_membind(&self) -> bool {
        unsafe { 0 != (*self.0).get_thisproc_membind }
    }

    /// Binding a given memory area is supported.
    pub fn set_area_membind(&self) -> bool {
        unsafe { 0 != (*self.0).set_area_membind }
    }

    /// Getting the binding of a given memory area is supported.
    pub fn get_area_membind(&self) -> bool {
        unsafe { 0 != (*self.0).get_area_membind }
    }

    /// Allocating a bound memory area is supported.
    pub fn alloc_membind(&self) -> bool {
        unsafe { 0 != (*self.0).alloc_membind }
    }

    /// First-touch policy is supported.
    pub fn firsttouch_membind(&self) -> bool {
        unsafe { 0 != (*self.0).firsttouch_membind }
    }

    /// Bind policy is supported.
    pub fn bind_membind(&self) -> bool {
        unsafe { 0 != (*self.0).bind_membind }
    }

    /// Interleave policy is supported.
    pub fn interleave_membind(&self) -> bool {
        unsafe { 0 != (*self.0).interleave_membind }
    }

    /// Next-touch migration policy is supported.
    pub fn nexttouch_membind(&self) -> bool {
        unsafe { 0 != (*self.0).nexttouch_membind }
    }

    /// Migration flags is supported.
    pub fn migrate_membind(&self) -> bool {
        unsafe { 0 != (*self.0).migrate_membind }
    }

    /// Getting the last NUMA nodes where a memory area was allocated is supported.
    pub fn get_area_memlocation(&self) -> bool {
        unsafe { 0 != (*self.0).get_area_memlocation }
    }
}

impl fmt::Debug for Membind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_null() {
            return Err(fmt::Error);
        }
        let m = unsafe { *self.0 };
        write!(f, "Membind{{ ")?;
        write!(f, "set_thisproc_membind: {}, ", m.set_thisproc_membind)?;
        write!(f, "get_thisproc_membind: {}, ", m.get_thisproc_membind)?;
        write!(f, "set_proc_membind: {}, ", m.set_proc_membind)?;
        write!(f, "get_proc_membind: {}, ", m.get_proc_membind)?;
        write!(f, "set_thisthread_membind: {}, ", m.set_thisthread_membind)?;
        write!(f, "get_thisthread_membind: {}, ", m.get_thisthread_membind)?;
        write!(f, "set_area_membind: {}, ", m.set_area_membind)?;
        write!(f, "get_area_membind: {}, ", m.get_area_membind)?;
        write!(f, "alloc_membind: {}, ", m.alloc_membind)?;
        write!(f, "firsttouch_membind: {}, ", m.firsttouch_membind)?;
        write!(f, "bind_membind: {}, ", m.bind_membind)?;
        write!(f, "interleave_membind: {}, ", m.interleave_membind)?;
        write!(f, "nexttouch_membind: {}, ", m.nexttouch_membind)?;
        write!(f, "migrate_membind: {}, ", m.migrate_membind)?;
        write!(f, "get_area_memlocation: {} ", m.get_area_memlocation)?;
        write!(f, "}}")
    }
}

/// Flags describing miscellaneous features.
pub struct Misc(*const hwloc2_sys::hwloc_topology_misc_support);

impl Misc {
    /// Support was imported when importing another topology, see [`Flags::IMPORT_SUPPORT`].
    ///
    /// [`Flags::IMPORT_SUPPORT`]: crate::topology::flags::Flags::IMPORT_SUPPORT
    pub fn imported_support(&self) -> bool {
        unsafe { 0 != (*self.0).imported_support }
    }
}

impl fmt::Debug for Misc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_null() {
            return Err(fmt::Error);
        }
        let m = unsafe { *self.0 };
        write!(f, "Misc{{ imported_support: {} }}", m.imported_support)
    }
}

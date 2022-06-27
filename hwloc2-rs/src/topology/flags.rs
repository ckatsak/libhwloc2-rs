bitflags::bitflags! {
    /// Flags to be set onto a topology context before load.
    ///
    /// Flags should be given to [`TopologyBuilder::flags`].
    /// They may also be returned by [`Topology::flags`].
    ///
    /// [`TopologyBuilder::flags`]: crate::topology::TopologyBuilder::flags
    /// [`Topology::flags`]: crate::topology::Topology::flags
    #[derive(Default)]
    #[repr(C)]
    pub struct Flags: u64 {
        /// Detect the whole system, ignore reservations, include disallowed objects.
        ///
        /// Gather all online resources, even if some were disabled by the administrator. For
        /// instance, ignore Linux Cgroup/Cpusets and gather all processors and memory nodes.
        /// However offline PUs and NUMA nodes are still ignored.
        ///
        /// When this flag is not set, PUs and NUMA nodes that are disallowed are not added to the
        /// topology. Parent objects (package, core, cache, etc.) are added only if some of their
        /// children are allowed. All existing PUs and NUMA nodes in the topology are allowed.
        /// `hwloc_topology_get_allowed_cpuset()` and `hwloc_topology_get_allowed_nodeset()` are
        /// equal to the root object cpuset and nodeset. FIXME doclink
        ///
        /// When this flag is set, the actual sets of allowed PUs and NUMA nodes are given by
        /// `hwloc_topology_get_allowed_cpuset()` and `hwloc_topology_get_allowed_nodeset()`.
        /// They may be smaller than the root object cpuset and nodeset. FIXME doclink
        ///
        /// If the current topology is exported to XML and reimported later, this flag should be
        /// set again in the reimported topology so that disallowed resources are reimported as
        /// well.
        const INCLUDE_DISALLOWED =
            hwloc2_sys::hwloc_topology_flags_e_HWLOC_TOPOLOGY_FLAG_INCLUDE_DISALLOWED as u64;

        /// Assume that the selected backend provides the topology for the system on which we are running.
        ///
        /// This forces [`Topology::is_this_system`] to return `true`, i.e. makes hwloc assume that
        /// the selected backend provides the topology for the system on which we are running, even
        /// if it is not the OS-specific backend but the XML backend for instance. This means
        /// making the binding functions actually call the OS-specific system calls and really do
        /// binding, while the XML backend would otherwise provide empty hooks just returning
        /// success.
        ///
        /// Setting the environment variable `HWLOC_THISSYSTEM` may also result in the same
        /// behavior.
        ///
        /// This can be used for efficiency reasons to first detect the topology once, save it to
        /// an XML file, and quickly reload it later through the XML backend, but still having
        /// binding functions actually do bind.
        ///
        /// [`Topology::is_this_system`]: crate::topology::Topology::is_this_system
        const IS_THISSYSTEM =
            hwloc2_sys::hwloc_topology_flags_e_HWLOC_TOPOLOGY_FLAG_IS_THISSYSTEM as u64;

        /// Get the set of allowed resources from the local operating system even if the topology
        /// was loaded from XML or synthetic description.
        ///
        /// If the topology was loaded from XML or from a synthetic string, restrict it by applying
        /// the current process restrictions such as Linux Cgroup/Cpuset.
        ///
        /// This is useful when the topology is not loaded directly from the local machine (e.g.
        /// for performance reason) and it comes with all resources, while the running process is
        /// restricted to only parts of the machine.
        ///
        /// This flag is ignored unless [`Flags::IS_THISSYSTEM`] is also set since the loaded
        /// topology must match the underlying machine where restrictions will be gathered from.
        ///
        /// Setting the environment variable `HWLOC_THISSYSTEM_ALLOWED_RESOURCES` would result in
        /// the same behavior.
        const THISSYSTEM_ALLOWED_RESOURCES =
            hwloc2_sys::hwloc_topology_flags_e_HWLOC_TOPOLOGY_FLAG_THISSYSTEM_ALLOWED_RESOURCES
                as u64;

        /// Import support from the imported topology.
        ///
        /// When importing a XML topology from a remote machine, binding is disabled by default
        /// (see [`Flags::IS_THISSYSTEM`]). This disabling is also marked by putting zeroes in the
        /// corresponding supported feature bits reported by [`Topology::support`].
        ///
        /// The flag [`IMPORT_SUPPORT`] actually imports support bits from the remote machine. It
        /// also sets the flag [`imported_support`] in [`support::Misc`]. If the imported XML did
        /// not contain any support information (exporter hwloc is too old), this flag is not set.
        ///
        /// Note that these supported features are only relevant for the hwloc installation that
        /// actually exported the XML topology (it may vary with the operating system, or with how
        /// hwloc was compiled).
        ///
        /// Note that setting this flag however does not enable binding for the locally imported
        /// hwloc topology, it only reports what the remote hwloc and machine support.
        ///
        /// [`Topology::support`]: crate::topology::Topology::support
        /// [`IMPORT_SUPPORT`]: Flags::IMPORT_SUPPORT
        /// [`imported_support`]: crate::topology::support::Misc::imported_support
        /// [`support::Misc`]: crate::topology::support::Misc
        const IMPORT_SUPPORT =
            hwloc2_sys::hwloc_topology_flags_e_HWLOC_TOPOLOGY_FLAG_IMPORT_SUPPORT as u64;

        /// Do not consider resources outside of the process CPU binding.
        ///
        /// If the binding of the process is limited to a subset of cores, ignore the other cores
        /// during discovery.
        ///
        /// The resulting topology is identical to what a call to `hwloc_topology_restrict()` would
        /// generate, but this flag also prevents hwloc from ever touching other resources during
        /// the discovery. FIXME doclink
        ///
        /// This flag especially tells the x86 backend to never temporarily rebind a thread on any
        /// excluded core. This is useful on Windows because such temporary rebinding can change
        /// the process binding. Another use-case is to avoid cores that would not be able to
        /// perform the hwloc discovery anytime soon because they are busy executing some
        /// high-priority real-time tasks.
        ///
        /// If process CPU binding is not supported, the thread CPU binding is considered instead
        /// if supported, or the flag is ignored.
        ///
        /// This flag requires [`Flags::IS_THISSYSTEM`] as well since binding support is required.
        const RESTRICT_TO_CPUBINDING =
            hwloc2_sys::hwloc_topology_flags_e_HWLOC_TOPOLOGY_FLAG_RESTRICT_TO_CPUBINDING as u64;

        /// Do not consider resources outside of the process memory binding.
        ///
        /// If the binding of the process is limited to a subset of NUMA nodes, ignore the other
        /// NUMA nodes during discovery.
        ///
        /// The resulting topology is identical to what a call to `hwloc_topology_restrict()` would
        /// generate, but this flag also prevents hwloc from ever touching other resources during
        /// the discovery. FIXME doclink
        ///
        /// This flag is meant to be used together with [`Flags::RESTRICT_TO_CPUBINDING`] when both
        /// cores and NUMA nodes should be ignored outside of the process binding.
        ///
        /// If process memory binding is not supported, the thread memory binding is considered
        /// instead if supported, or the flag is ignored.
        ///
        /// This flag requires [`Flags::IS_THISSYSTEM`] as well since binding support is required.
        const RESTRICT_TO_MEMBINDING =
            hwloc2_sys::hwloc_topology_flags_e_HWLOC_TOPOLOGY_FLAG_RESTRICT_TO_MEMBINDING as u64;

        /// Do not ever modify the process or thread binding during discovery.
        ///
        /// This flag disables all hwloc discovery steps that require a change of the process or
        /// thread binding. This currently only affects the x86 backend which gets entirely disabled.
        ///
        /// This is useful when `hwloc_topology_load()` ([`TopologyBuilder::build`]) is called
        /// while the application also creates additional threads or modifies the binding.
        ///
        /// This flag is also a strict way to make sure the process binding will not change to due
        /// thread binding changes on Windows (see `HWLOC_TOPOLOGY_FLAG_RESTRICT_TO_CPUBINDING`). FIXME doclink
        ///
        /// [`TopologyBuilder::build`]: crate::topology::TopologyBuilder::build
        const DONT_CHANGE_BINDING =
            hwloc2_sys::hwloc_topology_flags_e_HWLOC_TOPOLOGY_FLAG_DONT_CHANGE_BINDING as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::Flags;

    #[test]
    fn flags() {
        let f = Flags::default();
        assert!(f.is_empty());
    }
}

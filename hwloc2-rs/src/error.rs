use crate::{
    topology::{filters::Filter, flags::Flags},
    ObjectType,
};

/// An error type returned by calls to the API exposed by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failure to initialize a new topology context, reported by hwloc.
    #[error("Failed to allocate a topology context")]
    TopologyInit,

    /// Failure to load the new topology, reported by hwloc.
    #[error("Failed to load the actual topology object")]
    TopologyLoad,

    /// Failure to create a valid [`Support`] object, reported by hwloc.
    ///
    /// [`Support`]: crate::topology::support::Support
    #[error("Failed to create a valid topology::Support object")]
    TopologySupport,

    /// Failure to set the flags, reported by hwloc.
    #[error("Failed to set flags {0:?}")]
    TopologyFlags(Flags),

    /// The ABI check performed by hwloc has failed.
    #[error("Failed to verify topology's compatibility with the current hwloc library")]
    TopologyAbiCheck,

    /// Failure to retrieve the filter for the object type, reported by hwloc.
    #[error("Failed to get filter for object type '{0}'")]
    TopologyGetFilter(ObjectType),

    /// The `i32` value returned by hwloc for the object type does not correspond to a known
    /// [`Filter`].
    ///
    /// [`Filter`]: crate::topology::filters::Filter
    #[error("Unknown filter '{1}' retrieved from hwloc for object type '0'")]
    UnknownFilter(ObjectType, i32),

    /// Failure to set the filter for the object type, reported by hwloc.
    #[error("Failed to set filter '{1}' for object type '{0}'")]
    TopologySetFilter(ObjectType, Filter),

    /// Failure to set the filter for the object types, reported by hwloc.
    #[error("Failed to set filter '{0}' for I/O object types")]
    TopologySetIOTypesFilter(Filter),

    /// Failure to set the filter for the object types, reported by hwloc.
    #[error("Failed to set filter '{0}' for all object types")]
    TopologySetAllTypesFilter(Filter),

    /// Failure to set the filter for the object types, reported by hwloc.
    #[error("Failed to set filter '{0}' for cache object types")]
    TopologySetCacheTypesFilter(Filter),

    /// Failure to set the filter for the object types, reported by hwloc.
    #[error("Failed to set filter '{0}' for instruction cache object types")]
    TopologySetICacheTypesFilter(Filter),

    /// The depth does not exist in the topology.
    #[error("Depth '{0}' does not exist in the topology")]
    TopologyDepthDoesNotExist(i32),

    /// Failure to allocate a new Bitmap, reported by hwloc.
    #[error("Failed to allocate a new Bitmap")]
    BitmapAlloc,

    /// Failed to set the index in the Bitmap.
    #[error("Failed to set index '{0}' in the Bitmap")]
    BitmapSetBit(u32),

    /// Failed to clear the index in the Bitmap.
    #[error("Failed to clear index '{0}' in the Bitmap")]
    BitmapClearBit(u32),

    /// Failed to set the indexes in the Bitmap.
    #[error("Failed to set indexes from '{0}' to '{1}' in the Bitmap")]
    BitmapSetBitRange(u32, i32),

    /// Failed to clear the index in the Bitmap.
    #[error("Failed to clear indexes from '{0}' to '{1}' in the Bitmap")]
    BitmapClearBitRange(u32, i32),

    /// Returned when a pointer to a hwloc bitmap that has been provided from the caller appears to
    /// be `NULL`.
    #[error("The provided bitmap pointer is NULL")]
    BitmapNullPointer,

    /// Returned when [`Bitmap::singlify`] fails.
    ///
    /// [`Bitmap::singlify`]: crate::bitmap::Bitmap::singlify
    #[error("Failed to singlify bitmap")]
    BitmapSinglify,

    /// Failed to empty bitmap and set the specified bit via [`Bitmap::only`], reported by hwloc.
    ///
    /// [`Bitmap::only`]: crate::bitmap::Bitmap::only
    #[error("Failed to empty bitmap and set bit '{0}'")]
    BitmapOnly(u32),

    /// Failed to fill bitmap and clear the specified bit via [`Bitmap::allbut`], reported by
    /// hwloc.
    ///
    /// [`Bitmap::allbut`]: crate::bitmap::Bitmap::allbut
    #[error("Failed to fill bitmap and clear bit '{0}'")]
    BitmapAllBut(u32),

    /// Failure to negate the bitmap, reported by hwloc.
    #[error("Failed to negate the bitmap")]
    BitmapNegation,

    /// Failure to bind the current process or thread on a given CPU, reported by hwloc.
    #[error("Failed to bind the current process or thread on given CPU")]
    CpuBindSet,
}

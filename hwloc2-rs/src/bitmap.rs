//! "Convenient" interface for `hwloc`'s
//! [bitmap API](https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00181.php).
//!
//! This module has not been properly tested yet; it is largely based on
//! [this implementation](https://github.com/daschl/hwloc-rs/tree/b8a35b24168b5950da4dd7405d9359816815bee5),
//! with some minor modifications.

use std::{
    ffi::CStr,
    fmt,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
    ptr::{self, NonNull},
};

use crate::{error::Error, ptr_mut_to_const};

pub struct Bitmap {
    ptr: NonNull<hwloc2_sys::hwloc_bitmap_s>,
    // TODO(ckatsak): daschl's implementation includes a `manage` flag here, probably to deal with
    // `impl Drop` corner cases (similar to `TopologyBuilder`'s `built` flag).
    manage: bool,
}

/// A CPU set is a bitmap whose bits are set according to CPU physical OS indexes.
///
/// It may be consulted and modified with the [`Bitmap`] API.
///
/// Each bit may be converted into a PU object using [`Topology::pu_object_by_os_index`].
///
/// [`Topology::pu_object_by_os_index`]: crate::topology::Topology::pu_object_by_os_index
pub type CpuSet = Bitmap;

/// A node set is a bitmap whose bits are set according to NUMA memory node physical OS indexes.
///
/// It may be consulted and modified with the [`Bitmap`] API.
/// Each bit may be converted into a NUMA node object using
/// [`Topology::numanode_object_by_os_index`].
///
/// When binding memory on a system without any NUMA node, the single main memory bank is
/// considered as NUMA node `#0`.
///
/// See also
/// [Converting between CPU sets and node sets](https://www.open-mpi.org/projects/hwloc/doc/v2.7.1/a00179.php).
///
/// [`Topology::numanode_object_by_os_index`]: crate::topology::Topology::numanode_object_by_os_index
pub type NodeSet = Bitmap;

impl Bitmap {
    /// Allocate a new empty `Bitmap`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BitmapAlloc`] if hwloc failed to allocate the bitmap.
    ///
    /// [`Error::BitmapAlloc`]: crate::error::Error::BitmapAlloc
    pub fn try_new_empty() -> Result<Self, Error> {
        let ptr =
            NonNull::new(unsafe { hwloc2_sys::hwloc_bitmap_alloc() }).ok_or(Error::BitmapAlloc)?;
        Ok(Self { ptr, manage: true })
    }

    /// Allocate a new full `Bitmap`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BitmapAlloc`] if hwloc failed to allocate the bitmap.
    ///
    /// [`Error::BitmapAlloc`]: crate::error::Error::BitmapAlloc
    pub fn try_new_full() -> Result<Self, Error> {
        let ptr = NonNull::new(unsafe { hwloc2_sys::hwloc_bitmap_alloc_full() })
            .ok_or(Error::BitmapAlloc)?;
        Ok(Self { ptr, manage: true })
    }

    /// Add index `id` in this bitmap.
    pub fn set(&mut self, id: u32) -> Result<(), Error> {
        match unsafe { hwloc2_sys::hwloc_bitmap_set(self.ptr.as_ptr(), id) } {
            -1 => Err(Error::BitmapSetBit(id)),
            _ => Ok(()),
        }
    }

    /// Test whether index `id` is part of this bitmap.
    ///
    /// Returns `true` if the bit at index `id` is set in this bitmap, `false` otherwise.
    pub fn is_set(&self, id: u32) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_bitmap_isset(ptr_mut_to_const(self.ptr.as_ptr()), id) }
    }

    /// Add indexes from `begin` to `end` in this bitmap.
    ///
    /// If `end` is `-1`, the range is infinite.
    pub fn set_range(&mut self, begin: u32, end: i32) -> Result<(), Error> {
        match unsafe { hwloc2_sys::hwloc_bitmap_set_range(self.ptr.as_ptr(), begin, end) } {
            -1 => Err(Error::BitmapSetBitRange(begin, end)),
            _ => Ok(()),
        }
    }

    /// Remove index `id` from this bitmap.
    pub fn clear(&mut self, id: u32) -> Result<(), Error> {
        match unsafe { hwloc2_sys::hwloc_bitmap_clr(self.ptr.as_ptr(), id) } {
            -1 => Err(Error::BitmapClearBit(id)),
            _ => Ok(()),
        }
    }

    /// Remove indexes from `begin` to `end` in this bitmap.
    ///
    /// If `end` is `-1`, the range is infinite.
    pub fn clear_range(&mut self, begin: u32, end: i32) -> Result<(), Error> {
        match unsafe { hwloc2_sys::hwloc_bitmap_clr_range(self.ptr.as_ptr(), begin, end) } {
            -1 => Err(Error::BitmapClearBitRange(begin, end)),
            _ => Ok(()),
        }
    }

    /// Empty this bitmap.
    pub fn zero(&mut self) {
        unsafe { hwloc2_sys::hwloc_bitmap_zero(self.ptr.as_ptr()) }
    }

    /// Test whether this bitmap is empty.
    ///
    /// Returns `true` if bitmap is empty, `false` otherwise.
    ///
    /// # Note
    ///
    /// This is the same as [`Bitmap::is_empty`].
    pub fn is_zero(&self) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_bitmap_iszero(ptr_mut_to_const(self.ptr.as_ptr())) }
    }

    /// Test whether this bitmap is empty.
    ///
    /// Returns `true` if bitmap is empty, `false` otherwise.
    ///
    /// # Note
    ///
    /// This is the same as [`Bitmap::is_zero`].
    pub fn is_empty(&self) -> bool {
        self.is_zero()
    }

    /// Test whether this bitmap is completely full.
    ///
    /// Returns `true` if bitmap is full, `false` otherwise.
    ///
    /// # Note
    ///
    /// A full bitmap is always infinitely set.
    pub fn is_full(&self) -> bool {
        1 == unsafe { hwloc2_sys::hwloc_bitmap_isfull(ptr_mut_to_const(self.ptr.as_ptr())) }
    }

    /// Compute the first index (least significant bit) in this bitmap.
    ///
    /// Returns `None` if no index is set in bitmap.
    pub fn first(&self) -> Option<i32> {
        match unsafe { hwloc2_sys::hwloc_bitmap_first(ptr_mut_to_const(self.ptr.as_ptr())) } {
            -1 => None,
            ret => Some(ret),
        }
    }

    /// Compute the first unset index (least significant bit) in this bitmap.
    ///
    /// Returns `None` if no index is unset in bitmap.
    pub fn first_unset(&self) -> Option<i32> {
        match unsafe { hwloc2_sys::hwloc_bitmap_first_unset(ptr_mut_to_const(self.ptr.as_ptr())) } {
            -1 => None,
            ret => Some(ret),
        }
    }

    /// Compute the last index (most significant bit) in this bitmap.
    ///
    /// Returns `None` if no index is set in bitmap, or if bitmap is infinitely set.
    pub fn last(&self) -> Option<i32> {
        match unsafe { hwloc2_sys::hwloc_bitmap_last(ptr_mut_to_const(self.ptr.as_ptr())) } {
            -1 => None,
            ret => Some(ret),
        }
    }

    /// Compute the last unset index (most significant bit) in this bitmap.
    ///
    /// Returns `None` if no index is unset in bitmap, or if bitmap is infinitely set.
    pub fn last_unset(&self) -> Option<i32> {
        match unsafe { hwloc2_sys::hwloc_bitmap_last_unset(ptr_mut_to_const(self.ptr.as_ptr())) } {
            -1 => None,
            ret => Some(ret),
        }
    }

    /// Empty this bitmap and add bit `id`.
    pub fn only(&mut self, id: u32) -> Result<(), Error> {
        match unsafe { hwloc2_sys::hwloc_bitmap_only(self.ptr.as_ptr(), id) } {
            -1 => Err(Error::BitmapOnly(id)),
            _ => Ok(()),
        }
    }

    /// Fill the bitmap and clear the index `id`.
    pub fn allbut(&mut self, id: u32) -> Result<(), Error> {
        match unsafe { hwloc2_sys::hwloc_bitmap_allbut(self.ptr.as_ptr(), id) } {
            -1 => Err(Error::BitmapAllBut(id)),
            _ => Ok(()),
        }
    }

    /// Keep a single index among those set in this bitmap.
    ///
    /// May be useful before binding so that the process does not have a chance of migrating
    /// between multiple processors in the original mask. Instead of running the task on any PU
    /// inside the given CPU set, the operating system scheduler will be forced to run it on a
    /// single of these PUs. It avoids a migration overhead and cache-line ping-pongs between PUs.
    ///
    /// # Note
    ///
    /// - This function is NOT meant to distribute multiple processes within a single CPU set. It
    /// always returns the same single bit when called multiple times on the same input set.
    /// `hwloc_distrib()` may be used for generating CPU sets to distribute multiple tasks below a
    /// single multi-PU object.
    /// - This function cannot be applied to an object set directly. It should be applied to a
    /// copy (which may be obtained with [`Bitmap::clone`]).
    // FIXME: doc not updated & doclinks missing
    ///
    /// # Errors
    ///
    /// Returns [`Error::BitmapSinglify`] in case of failure reported by hwloc.
    ///
    /// [`Error::BitmapSinglify`]: crate::error::Error::BitmapSinglify
    pub fn singlify(&mut self) -> Result<(), Error> {
        match unsafe { hwloc2_sys::hwloc_bitmap_singlify(self.ptr.as_ptr()) } {
            -1 => Err(Error::BitmapSinglify),
            _ => Ok(()),
        }
    }

    /// Negate this bitmap.
    pub fn invert(&mut self) -> Result<(), Error> {
        match unsafe {
            hwloc2_sys::hwloc_bitmap_not(self.ptr.as_ptr(), ptr_mut_to_const(self.ptr.as_ptr()))
        } {
            -1 => Err(Error::BitmapNegation),
            _ => Ok(()),
        }
    }

    /// Setup this bitmap from a `u64` mask.
    ///
    /// # Panics
    ///
    /// In case of allocation failure in hwloc.
    ///
    /// # Note
    ///
    /// See also `impl From<u64> for Bitmap`.
    pub fn from_ulong(&mut self, mask: u64) {
        // Implementation in `hwloc/bitmap.c` appears to always return 0
        let ret = unsafe { hwloc2_sys::hwloc_bitmap_from_ulong(self.ptr.as_ptr(), mask) };
        debug_assert_eq!(0, ret);
    }

    /// Wraps the provided `bitmap` pointer into a `Bitmap` object.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BitmapNullPointer`] if the provided `bitmap` pointer is `NULL`.
    ///
    /// # Safety
    ///
    /// This function is `unsafe` because it is now the responsibility of the caller to make sure
    /// that the provided pointer actually points to a valid hwloc bitmap.
    ///
    /// [`Error::BitmapNullPointer`]: crate::error::Error::BitmapNullPointer
    pub unsafe fn from_raw(
        bitmap: *mut hwloc2_sys::hwloc_bitmap_s,
        manage: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            ptr: NonNull::new(bitmap).ok_or(Error::BitmapNullPointer)?,
            manage,
        })
    }

    /// Returns the contained hwloc bitmap pointer, for interaction with hwloc.
    pub fn as_ptr(&self) -> *mut hwloc2_sys::hwloc_bitmap_s {
        self.ptr.as_ptr()
    }

    /// Duplicate this bitmap by allocating a new bitmap and copying bitmap contents.
    ///
    /// # Panics
    ///
    /// If current hwloc bitmap pointer is `NULL`.
    ///
    /// # Note
    ///
    /// Same as `impl Clone for Bitmap`.
    pub fn dup(&self) -> Self {
        self.clone()
    }

    /// Compute the "weight" of this bitmap (i.e., number of indexes that are in the bitmap).
    ///
    /// Returns:
    /// - the number of indexes that are in the bitmap.
    /// - `-1` if bitmap is infinitely set.
    pub fn weight(&self) -> i32 {
        unsafe { hwloc2_sys::hwloc_bitmap_weight(ptr_mut_to_const(self.ptr.as_ptr())) }
    }

    /// Test whether bitmaps `self` and `other` intersect.
    ///
    /// Returns `true` if bitmaps intersect, `false` otherwise.
    pub fn intersects(&self, other: Bitmap) -> bool {
        1 == unsafe {
            hwloc2_sys::hwloc_bitmap_intersects(
                ptr_mut_to_const(self.ptr.as_ptr()),
                ptr_mut_to_const(other.ptr.as_ptr()),
            )
        }
    }

    /// Test whether this bitmap is part of bitmap `other`.
    ///
    /// Returns `true` if this bitmap is included in `other`, `false` otherwise.
    ///
    /// # Note
    ///
    /// The empty bitmap is considered included in any other bitmap.
    pub fn is_included(&self, other: &Bitmap) -> bool {
        1 == unsafe {
            hwloc2_sys::hwloc_bitmap_isincluded(
                ptr_mut_to_const(self.ptr.as_ptr()),
                ptr_mut_to_const(other.ptr.as_ptr()),
            )
        }
    }
}

impl Clone for Bitmap {
    fn clone(&self) -> Self {
        let new = unsafe { hwloc2_sys::hwloc_bitmap_dup(ptr_mut_to_const(self.ptr.as_ptr())) };
        Self {
            ptr: NonNull::new(new).expect("Bitmap's internal pointer is NULL; this is a BUG"),
            manage: true,
        }
    }
}

impl From<u64> for Bitmap {
    fn from(mask: u64) -> Self {
        let ret = Self::try_new_empty().expect("failed to allocate new empty bitmap");
        // Implementation in `hwloc/bitmap.c` appears to always return 0
        assert_eq!(0, unsafe {
            hwloc2_sys::hwloc_bitmap_from_ulong(ret.ptr.as_ptr(), mask)
        });
        ret
    }
}

impl PartialEq for Bitmap {
    fn eq(&self, other: &Self) -> bool {
        1 == unsafe {
            hwloc2_sys::hwloc_bitmap_isequal(
                ptr_mut_to_const(self.ptr.as_ptr()),
                other.ptr.as_ptr(),
            )
        }
    }
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        if self.manage {
            unsafe { hwloc2_sys::hwloc_bitmap_free(self.ptr.as_ptr()) }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/////
/////  Bitwise Operations
/////
///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'b> Not for &'b Bitmap {
    type Output = Bitmap;

    fn not(self) -> Self::Output {
        let new = unsafe { hwloc2_sys::hwloc_bitmap_alloc() };
        if -1 == unsafe { hwloc2_sys::hwloc_bitmap_not(new, ptr_mut_to_const(self.ptr.as_ptr())) } {
            panic!("hwloc reported failure to negate bitmap");
        }
        match unsafe { Bitmap::from_raw(new, true) } {
            Err(err) => panic!("failed to create Bitmap: {}", err),
            Ok(new) => new,
        }
    }
}

impl<'l, 'r> BitAnd<&'r Bitmap> for &'l Bitmap {
    type Output = Bitmap;

    fn bitand(self, rhs: &'r Bitmap) -> Self::Output {
        let new = unsafe { hwloc2_sys::hwloc_bitmap_alloc() };
        if unsafe {
            hwloc2_sys::hwloc_bitmap_and(
                new,
                ptr_mut_to_const(self.ptr.as_ptr()),
                ptr_mut_to_const(rhs.as_ptr()),
            )
        } == -1
        {
            panic!("hwloc reported failure to bitwise AND bitmaps");
        }
        match unsafe { Bitmap::from_raw(new, true) } {
            Err(err) => panic!("failed to create Bitmap: {}", err),
            Ok(new) => new,
        }
    }
}

impl BitAndAssign<&Bitmap> for Bitmap {
    fn bitand_assign(&mut self, rhs: &Self) {
        if unsafe {
            hwloc2_sys::hwloc_bitmap_and(
                self.as_ptr(),
                ptr_mut_to_const(self.as_ptr()),
                ptr_mut_to_const(rhs.as_ptr()),
            )
        } == -1
        {
            panic!("hwloc reported failure to `&=` bitmaps");
        }
    }
}

impl<'l, 'r> BitOr<&'r Bitmap> for &'l Bitmap {
    type Output = Bitmap;

    fn bitor(self, rhs: &'r Bitmap) -> Self::Output {
        let new = unsafe { hwloc2_sys::hwloc_bitmap_alloc() };
        if unsafe {
            hwloc2_sys::hwloc_bitmap_or(
                new,
                ptr_mut_to_const(self.ptr.as_ptr()),
                ptr_mut_to_const(rhs.as_ptr()),
            )
        } == -1
        {
            panic!("hwloc reported failure to bitwise OR bitmaps");
        }
        match unsafe { Bitmap::from_raw(new, true) } {
            Err(err) => panic!("failed to create Bitmap: {}", err),
            Ok(new) => new,
        }
    }
}

impl BitOrAssign<&Bitmap> for Bitmap {
    fn bitor_assign(&mut self, rhs: &Self) {
        if unsafe {
            hwloc2_sys::hwloc_bitmap_or(
                self.as_ptr(),
                ptr_mut_to_const(self.as_ptr()),
                ptr_mut_to_const(rhs.as_ptr()),
            )
        } == -1
        {
            panic!("hwloc reported failure to `|=` bitmaps");
        }
    }
}

impl<'l, 'r> BitXor<&'r Bitmap> for &'l Bitmap {
    type Output = Bitmap;

    fn bitxor(self, rhs: &'r Bitmap) -> Self::Output {
        let new = unsafe { hwloc2_sys::hwloc_bitmap_alloc() };
        if unsafe {
            hwloc2_sys::hwloc_bitmap_xor(
                new,
                ptr_mut_to_const(self.ptr.as_ptr()),
                ptr_mut_to_const(rhs.as_ptr()),
            )
        } == -1
        {
            panic!("hwloc reported failure to bitwise XOR bitmaps");
        }
        match unsafe { Bitmap::from_raw(new, true) } {
            Err(err) => panic!("failed to create Bitmap: {}", err),
            Ok(new) => new,
        }
    }
}

impl BitXorAssign<&Bitmap> for Bitmap {
    fn bitxor_assign(&mut self, rhs: &Self) {
        if unsafe {
            hwloc2_sys::hwloc_bitmap_xor(
                self.as_ptr(),
                ptr_mut_to_const(self.as_ptr()),
                ptr_mut_to_const(rhs.as_ptr()),
            )
        } == -1
        {
            panic!("hwloc reported failure to `^=` bitmaps");
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/////
/////  Iterator
/////
///////////////////////////////////////////////////////////////////////////////////////////////////

impl IntoIterator for Bitmap {
    type Item = u32;
    type IntoIter = BitmapIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            bitmap: self,
            curr: -1,
        }
    }
}

pub struct BitmapIntoIterator {
    bitmap: Bitmap,
    curr: i32,
}

impl Iterator for BitmapIntoIterator {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = unsafe {
            hwloc2_sys::hwloc_bitmap_next(ptr_mut_to_const(self.bitmap.ptr.as_ptr()), self.curr)
        };
        self.curr = ret;
        if ret < 0 {
            None
        } else {
            Some(ret as _)
        }
    }
}

impl FromIterator<u32> for Bitmap {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        let mut ret = Bitmap::try_new_empty().expect("failed to allocate a new empty bitmap");
        for i in iter {
            ret.set(i).expect("Bitmap::from_iter() failed to set bit");
        }
        ret
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/////
/////  Printing
/////
///////////////////////////////////////////////////////////////////////////////////////////////////

impl fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: Pass a NULL pointer for the C string to be managed by hwloc, and a valid
        // (private since its creation) pointer to the bitmap.
        let mut strp: *mut libc::c_char = ptr::null_mut();
        unsafe {
            hwloc2_sys::hwloc_bitmap_list_asprintf(&mut strp, ptr_mut_to_const(self.ptr.as_ptr()))
        };

        // SAFETY: The memory behind this pointer should have been allocated and managed by hwloc.
        let cstr = unsafe { CStr::from_ptr(ptr_mut_to_const(strp)) }
            .to_str()
            .expect("failed to convert CStr to str");
        let ret = write!(f, "{}", cstr);

        // SAFETY: The memory behind this pointer should have been allocated and managed by hwloc.
        // If it was invalid, I guess we would have already died, and it has certainly not been
        // freed before, as it was allocated just above.
        unsafe { libc::free(strp as _) };
        ret
    }
}

impl fmt::Display for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::Bitmap;

    const TEST_RANGE: u32 = 128;

    fn set(b: &mut Bitmap, range: u32, step: u32) {
        for i in 0..range {
            if i % step == 0 {
                b.set(i).expect("failed to Bitmap::set()");
            }
        }
    }

    #[test]
    fn bitmap_print() {
        let mut b1 = Bitmap::try_new_full().unwrap();
        eprintln!(" b1 = {}\n!b1 = {}", b1, !&b1);

        set(&mut b1, TEST_RANGE, 3);
        eprintln!(" b1 = {:?}\n!b1 = {:?}", b1, !&b1);
    }

    #[test]
    fn bitmap_misc() {
        let mut b1 = Bitmap::try_new_full().unwrap();
        assert!(b1.is_full());
        assert_eq!(None, b1.first_unset());
        assert_eq!(-1, b1.weight());
        b1.clear_range(3, 8).expect("failed to Bitmap::clear_range");
        b1.clear_range(43, 48).expect("failed to B::clear_range()");

        b1.zero();
        assert!(b1.is_empty());
        assert_eq!(b1.is_zero(), b1.is_empty());
        assert_eq!(None, b1.first());

        let mut b2 = b1.clone();
        b2.only(42).expect("failed to Bitmap::only()");
        b1.set(42).expect("failed to Bitmap::set()");
        assert_eq!(b1, b2);
        assert_eq!(1, b2.weight());
        assert_eq!(b1.weight(), b2.weight());
        assert!(b2.is_included(&b1));

        assert_eq!(42, b1.first().expect("failed to Bitmap::first()"));
        assert_eq!(0, b1.first_unset().expect("failed to Bitmap::first_unset"));
        assert_eq!(42, b1.last().expect("failed to B::last()"));
        assert_eq!(None, b1.last_unset());

        b1 = b2.dup();
        assert_eq!(b1, b2);

        b1.set_range(40, 45).expect("failed to Bitmap::set_range()");
        assert!(b1.is_set(42));

        b1.clear(42).expect("failed to Bitmap::clear()");
        assert!(!b1.is_set(42));

        b1.allbut(24).expect("failed to Bitmap::allbut()");
        assert_eq!(0, b1.first().expect("failed to Bitmap::first()"));
        assert_eq!(24, b1.first_unset().expect("failed to Bitmap::first_unset"));
        assert_eq!(None, b1.last());
        assert_eq!(24, b1.last_unset().expect("failed to Bitmap::last_unset()"));
    }

    #[test]
    fn bitmap_not() {
        let mut b1 = Bitmap::try_new_empty().unwrap();
        set(&mut b1, TEST_RANGE, 3);
        let b2 = !&b1;
        for i in 0..TEST_RANGE {
            assert!(
                (b1.is_set(i) && !b2.is_set(i)) || (!b1.is_set(i) && b2.is_set(i)),
                "Bitmap's bitwise negation failed"
            );
        }

        b1.invert().expect("failed to Bitmap::invert()");
        assert_eq!(b1, b2);
    }

    #[test]
    fn bitmap_and() {
        let mut b1 = Bitmap::try_new_empty().unwrap();
        set(&mut b1, TEST_RANGE, 3);

        let mut b2 = !&b1;
        let b3 = &b1 & &b2;
        b2 &= &b1;
        assert_eq!(b2, b3, "bitwise-AND and &= do not produce the same bitmap!");
    }

    #[test]
    fn bitmap_or() {
        let mut b1 = Bitmap::try_new_empty().unwrap();
        set(&mut b1, TEST_RANGE, 3);

        let mut b2 = !&b1;
        let b3 = &b1 | &b2;
        b2 |= &b1;
        assert_eq!(b2, b3, "bitwise-OR and |= do not produce the same bitmap!");
    }

    #[test]
    fn bitmap_xor() {
        let mut b1 = Bitmap::try_new_empty().unwrap();
        set(&mut b1, TEST_RANGE, 3);

        let mut b2 = !&b1;
        let b3 = &b1 ^ &b2;
        b2 ^= &b1;
        assert_eq!(b2, b3, "bitwise-OR and |= do not produce the same bitmap!");
    }
}

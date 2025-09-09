use core::ptr::{self, NonNull};

use crate::{
    array::{helper::*, *},
    ops, uint,
};

/// Unsizes a raw pointer to an [`Array`] into a raw pointer to a slice.
///
/// # Panics
/// If `A::Length > usize::MAX`
///
/// # Safety
/// This operation is strictly the same as [`ptr::slice_from_raw_parts`] with `ptr.cast()` as
/// the first argument and [`ArrApi::<A>::length()`] as the second.
#[track_caller]
pub const fn unsize_raw<A: Array>(ptr: *const A) -> *const [A::Item] {
    ptr::slice_from_raw_parts(ptr.cast(), arr_len::<A>())
}

/// Unsizes a raw pointer to an [`Array`] into a raw pointer to a slice.
///
/// # Panics
/// If `A::Length > usize::MAX`
///
/// # Safety
/// This operation is strictly the same as [`ptr::slice_from_raw_parts_mut`] with `ptr.cast()` as
/// the first argument and [`ArrApi::<A>::length()`] as the second.
#[track_caller]
pub const fn unsize_raw_mut<A: Array>(ptr: *mut A) -> *mut [A::Item] {
    ptr::slice_from_raw_parts_mut(ptr.cast(), arr_len::<A>())
}

/// Unsizes a raw pointer to an [`Array`] into a raw pointer to a slice.
///
/// # Panics
/// If `A::Length > usize::MAX`
///
/// # Safety
/// This operation is strictly the same as [`NonNull::slice_from_raw_parts`] with `ptr.cast()` as
/// the first argument and [`ArrApi::<A>::length()`] as the second.
#[track_caller]
pub const fn unsize_nonnull<A: Array>(ptr: NonNull<A>) -> NonNull<[A::Item]> {
    NonNull::slice_from_raw_parts(ptr.cast(), arr_len::<A>())
}
/// Unsizes a reference to an [`Array`] into a reference to a slice.
///
/// This is the same as [`ArrApi::as_slice`].
///
/// # Panics
/// If `A::Length > usize::MAX`
#[track_caller]
pub const fn unsize_ref<A: Array>(arr: &A) -> &[A::Item] {
    // SAFETY: `Array` to slice cast
    unsafe { &*unsize_raw(arr) }
}

/// Unsizes a reference to an [`Array`] into a reference to a slice.
///
/// This is the same as [`ArrApi::as_mut_slice`].
///
/// # Panics
/// If `A::Length > usize::MAX`
#[track_caller]
pub const fn unsize_mut<A: Array>(arr: &mut A) -> &mut [A::Item] {
    // SAFETY: `Array` to slice cast
    unsafe { &mut *unsize_raw_mut(arr) }
}

/// Unsizes a boxed [`Array`] into a boxed slice.
///
/// # Panics
/// If `A::Length > usize::MAX`
#[cfg(feature = "alloc")]
pub fn unsize_box<A: Array>(arr: alloc::boxed::Box<A>) -> alloc::boxed::Box<[A::Item]> {
    use alloc::boxed::Box;
    // SAFETY: `Array` to slice cast
    unsafe { Box::from_raw(unsize_raw_mut(Box::into_raw(arr))) }
}

/// Unsizes a ref counted [`Array`] into a ref counted slice.
///
/// # Panics
/// If `A::Length > usize::MAX`
#[cfg(feature = "alloc")]
pub fn unsize_rc<A: Array>(arr: alloc::rc::Rc<A>) -> alloc::rc::Rc<[A::Item]> {
    use alloc::rc::Rc;
    // SAFETY: `Array` to slice cast
    unsafe { Rc::from_raw(unsize_raw(Rc::into_raw(arr))) }
}

/// Unsizes a ref counted [`Array`] into a ref counted slice.
///
/// # Panics
/// If `A::Length > usize::MAX`
#[cfg(feature = "alloc")]
pub fn unsize_arc<A: Array>(arr: alloc::sync::Arc<A>) -> alloc::sync::Arc<[A::Item]> {
    use alloc::sync::Arc;
    // SAFETY: `Array` to slice cast
    unsafe { Arc::from_raw(unsize_raw(Arc::into_raw(arr))) }
}

/// Casts a [`NonNull`] slice to a `NonNull` [`Array`].
///
/// If [`NonNull::len`] is not equal to [`A::Length`](Array::Length), then [`None`] is returned; this includes
/// the case where `A::Length` exceeds [`usize::MAX`].
///
/// # Safety
/// If a pointer is returned, then it will be the result of calling [`NonNull::cast`] on the slice.
/// As a consequence, due to the layout and interchangability guarantees made by [`Array`], the
/// returned pointer will be valid for any access that the input was valid for.
#[inline(always)]
pub const fn from_nonnull_slice<A: Array>(slice: NonNull<[A::Item]>) -> Option<NonNull<A>> {
    match uint::to_usize::<A::Length>() {
        Some(arr_len) if arr_len == slice.len() => Some(slice.cast()),
        _ => None,
    }
}

/// Turns a slice into an [`Array`] if the length is the same.
///
/// Otherwise returns [`None`].
pub const fn from_ref_slice<A: Array>(slice: &[A::Item]) -> Option<&A> {
    crate::utils::opt_map!(
        // SAFETY: `&[A::Item]` to `&A` cast with same item and length.
        // This is the same as the `&[T; N] as TryFrom<&[T]>` impl
        |ptr| unsafe { ptr.as_ref() },
        from_nonnull_slice(core::ptr::NonNull::from_ref(slice)),
    )
}

/// Turns a slice into an [`Array`] if the length is the same.
///
/// Otherwise returns [`None`].
pub const fn from_mut_slice<A: Array>(slice: &mut [A::Item]) -> Option<&mut A> {
    crate::utils::opt_map!(
        // SAFETY: `&mut [A::Item]` to `&mut A` cast with same item and length.
        // This is the same as the `&mut [T; N] as TryFrom<&mut [T]>` impl
        |mut ptr| unsafe { ptr.as_mut() },
        from_nonnull_slice(core::ptr::NonNull::from_mut(slice)),
    )
}

macro_rules! unsafe_smartptr_from_slice {
    ($name:ident, $mod:ident, $ptr:ident) => {
        /// Turns a slice into an [`Array`] if the length is the same.
        ///
        /// Otherwise returns the input back in an [`Err`] variant.
        #[cfg(feature = "alloc")]
        pub fn $name<A: crate::array::Array>(
            slice: alloc::$mod::$ptr<[A::Item]>,
        ) -> Result<alloc::$mod::$ptr<A>, alloc::$mod::$ptr<[A::Item]>> {
            use alloc::$mod::$ptr;

            match uint::to_usize::<A::Length>() {
                Some(arr_len) if arr_len == slice.len() => {
                    // SAFETY:
                    // - Pointer cast with same item and length.
                    // - Ownership is transferred through into_raw followed by from_raw.
                    // - This is the same as the `$ptr<[T; N]> as TryFrom<$ptr<[T]>>` impl
                    Ok(unsafe { $ptr::from_raw($ptr::into_raw(slice).cast()) })
                }
                _ => Err(slice),
            }
        }
    };
}
unsafe_smartptr_from_slice!(from_boxed_slice, boxed, Box);
unsafe_smartptr_from_slice!(from_rc_slice, rc, Rc);
unsafe_smartptr_from_slice!(from_arc_slice, sync, Arc);

macro_rules! retype_fns {
    ([$($const:ident)?], $unchecked:ident, $infallible:ident, $try:ident, $src:ty, $dst:ty, |$srcpat:pat_param| $conv_unchecked:expr,) => {
        /// Converts between equivalent array types.
        ///
        /// # Safety
        /// `Src::Length == Dst::Length`
        pub $($const)? unsafe fn $unchecked<Src, Dst>($srcpat: $src) -> $dst
        where
            Src: crate::array::Array,
            Dst: crate::array::Array<Item = Src::Item>,
        {
            $conv_unchecked
        }

        /// Converts between equivalent array types.
        pub $($const)? fn $infallible<Src, Dst>(src: $src) -> $dst
        where
            Src: crate::array::Array,
            Dst: crate::array::Array<Item = Src::Item, Length = Src::Length>,
        {
            // SAFETY: By definition
            unsafe { $unchecked(src) }
        }

        /// Converts between equivalent array types.
        ///
        /// Returns the original array if the lengths do not match.
        pub $($const)? fn $try<Src, Dst>(
            src: $src,
        ) -> crate::tern::TernRes<crate::ops::Eq<Src::Length, Dst::Length>, $dst, $src>
        where
            Src: crate::array::Array,
            Dst: crate::array::Array<Item = Src::Item>,
        {
            match uint::to_bool::<ops::Eq<Src::Length, Dst::Length>>() {
                // SAFETY: Src::Length == Dst::Length
                true => crate::tern::TernRes::make_ok(unsafe { $unchecked(src) }),
                false => crate::tern::TernRes::make_err(src),
            }
        }
    };
}
retype_fns![
    [const],
    retype_unchecked,
    retype,
    try_retype,
    Src,
    Dst,
    // SAFETY: Src::Length == Dst::Length, `Array` invariant
    |src| unsafe { crate::utils::exact_transmute(src) },
];
retype_fns![
    [const],
    retype_ref_unchecked,
    retype_ref,
    try_retype_ref,
    &Src,
    &Dst,
    // SAFETY: Src::Length == Dst::Length, `Array` invariant
    |src| unsafe { &*core::ptr::from_ref(src).cast() },
];
retype_fns![
    [const],
    retype_mut_unchecked,
    retype_mut,
    try_retype_mut,
    &mut Src,
    &mut Dst,
    // SAFETY: Src::Length == Dst::Length, `Array` invariant
    |src| unsafe { &mut *core::ptr::from_mut(src).cast() },
];
#[cfg(feature = "alloc")]
retype_fns![
    [],
    retype_box_unchecked,
    retype_box,
    try_retype_box,
    alloc::boxed::Box<Src>,
    alloc::boxed::Box<Dst>,
    // SAFETY: Src::Length == Dst::Length, `Array` invariant
    |src| unsafe { alloc::boxed::Box::from_raw(alloc::boxed::Box::into_raw(src).cast()) },
];
#[cfg(feature = "alloc")]
retype_fns![
    [],
    retype_rc_unchecked,
    retype_rc,
    try_retype_rc,
    alloc::rc::Rc<Src>,
    alloc::rc::Rc<Dst>,
    // SAFETY: Src::Length == Dst::Length, `Array` invariant
    |src| unsafe { alloc::rc::Rc::from_raw(alloc::rc::Rc::into_raw(src).cast()) },
];
#[cfg(feature = "alloc")]
retype_fns![
    [],
    retype_arc_unchecked,
    retype_arc,
    try_retype_arc,
    alloc::sync::Arc<Src>,
    alloc::sync::Arc<Dst>,
    // SAFETY: Src::Length == Dst::Length, `Array` invariant
    |src| unsafe { alloc::sync::Arc::from_raw(alloc::sync::Arc::into_raw(src).cast()) },
];

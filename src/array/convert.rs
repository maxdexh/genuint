use core::ptr::{self, NonNull};

use crate::{
    array::{helper::*, *},
    uint,
};

// FIXME: Missing docs

pub(crate) const fn retype_arr<Src, Dst>(src: Src) -> Dst
where
    Src: Array,
    Dst: Array<Item = Src::Item, Length = Src::Length>,
{
    // SAFETY: By definition
    unsafe { arr_convert_unchecked(src) }
}

pub const fn unsize_ref<A: Array>(arr: &A) -> &[A::Item] {
    // SAFETY: `Array` to slice cast
    unsafe { &*unsize_raw(arr) }
}
pub const fn unsize_mut<A: Array>(arr: &mut A) -> &mut [A::Item] {
    // SAFETY: `Array` to slice cast
    unsafe { &mut *unsize_raw_mut(arr) }
}
pub const fn unsize_raw<A: Array>(ptr: *const A) -> *const [A::Item] {
    ptr::slice_from_raw_parts(ptr.cast(), arr_len::<A>())
}
pub const fn unsize_raw_mut<A: Array>(ptr: *mut A) -> *mut [A::Item] {
    ptr::slice_from_raw_parts_mut(ptr.cast(), arr_len::<A>())
}
pub const fn unsize_nonnull<A: Array>(ptr: NonNull<A>) -> NonNull<[A::Item]> {
    NonNull::slice_from_raw_parts(ptr.cast(), arr_len::<A>())
}
#[cfg(feature = "alloc")]
pub fn unsize_box<A: Array>(arr: alloc::boxed::Box<A>) -> alloc::boxed::Box<[A::Item]> {
    use alloc::boxed::Box;
    // SAFETY: `Array` to slice cast
    unsafe { Box::from_raw(unsize_raw_mut(Box::into_raw(arr))) }
}
#[cfg(feature = "alloc")]
pub fn unsize_rc<A: Array>(arr: alloc::rc::Rc<A>) -> alloc::rc::Rc<[A::Item]> {
    use alloc::rc::Rc;
    // SAFETY: `Array` to slice cast
    unsafe { Rc::from_raw(unsize_raw(Rc::into_raw(arr))) }
}
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
pub const fn from_slice<A: Array>(slice: &[A::Item]) -> Option<&A> {
    crate::utils::opt_map!(
        // SAFETY: `&[A::Item]` to `&A` cast with same item and length.
        // This is the same as the `&[T; N] as TryFrom<&[T]>` impl
        |ptr| unsafe { ptr.as_ref() },
        from_nonnull_slice(core::ptr::NonNull::from_ref(slice)),
    )
}
pub const fn from_mut_slice<A: Array>(slice: &mut [A::Item]) -> Option<&mut A> {
    crate::utils::opt_map!(
        // SAFETY: `&mut [A::Item]` to `&mut A` cast with same item and length.
        // This is the same as the `&mut [T; N] as TryFrom<&mut [T]>` impl
        |mut ptr| unsafe { ptr.as_mut() },
        from_nonnull_slice(core::ptr::NonNull::from_mut(slice)),
    )
}
#[cfg(feature = "alloc")]
pub fn from_boxed_slice<A: Array>(
    slice: alloc::boxed::Box<[A::Item]>,
) -> Option<alloc::boxed::Box<A>> {
    use alloc::boxed::Box;

    // SAFETY: `Box` is never null.
    let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(slice)) };
    // SAFETY:
    // - `Box<[A::Item]>` to `Box<A>` cast with same item and length.
    // - Ownership is transferred through into_raw followed by from_raw.
    // - This is the same as the `Box<[T; N]> as TryFrom<Box<[T]>>` impl
    unsafe { Some(Box::from_raw(from_nonnull_slice(ptr)?.as_ptr())) }
}
#[cfg(feature = "alloc")]
pub fn from_rc_slice<A: Array>(slice: alloc::rc::Rc<[A::Item]>) -> Option<alloc::rc::Rc<A>> {
    use alloc::rc::Rc;

    // SAFETY: `Rc` is never null.
    let ptr = unsafe { NonNull::new_unchecked(Rc::into_raw(slice).cast_mut()) };
    // SAFETY:
    // - `Rc<[A::Item]>` to `Rc<A>` cast with same item and size.
    // - The ref count is preserved through into_raw followed by from_raw.
    // - The drop glue of `A` is required to do the same thing as that of `[A::Item]`, so it does
    //   not matter which `Rc` drops the slice in the end
    // - This is the same as the `Rc<[T; N]> as TryFrom<Rc<[T]>>` impl
    unsafe { Some(Rc::from_raw(from_nonnull_slice(ptr)?.as_ptr())) }
}
#[cfg(feature = "alloc")]
pub fn from_arc_slice<A: Array>(slice: alloc::sync::Arc<[A::Item]>) -> Option<alloc::sync::Arc<A>> {
    use alloc::sync::Arc;

    // SAFETY: `Arc` is never null.
    let ptr = unsafe { NonNull::new_unchecked(Arc::into_raw(slice).cast_mut()) };
    // SAFETY: See `from_rc_slice`
    unsafe { Some(Arc::from_raw(from_nonnull_slice(ptr)?.as_ptr())) }
}

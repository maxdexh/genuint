use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{Uint, array::*, const_fmt, uint};

#[track_caller]
pub(crate) const fn arr_len<A: Array>() -> usize {
    const fn doit<N: Uint>() -> usize {
        let precalc = const {
            if let Some(n) = uint::to_usize::<N>() {
                Ok(n)
            } else {
                Err(const_fmt::fmt![
                    "Array length ",
                    PhantomData::<N>,
                    " exceeds the maximum value for a usize",
                ])
            }
        };
        match precalc {
            Ok(n) => n,
            Err(fmt) => fmt.panic(),
        }
    }
    doit::<A::Length>()
}

pub(crate) const fn uint_is<N: crate::ToUint>(n: usize) -> bool {
    if let Some(value) = uint::to_usize::<N>() {
        n == value
    } else {
        false
    }
}
pub(crate) const fn len_is<A: Array>(len: usize) -> bool {
    uint_is::<A::Length>(len)
}

pub(crate) const fn init_fill<T: Copy>(mut buf: &mut [MaybeUninit<T>], item: T) {
    while let [first, rest @ ..] = buf {
        *first = MaybeUninit::new(item);
        buf = rest;
    }
}

/// # Panics
/// If `A::Length > usize::MAX`
///
/// # Safety
/// This operation is strictly the same as [`core::ptr::slice_from_raw_parts_mut`] with `ptr.cast()` as
/// the first argument and [`ArrApi::<A>::length()`] as the second.
///
/// Due to the guarantees made by [`Array`], this should generally mean that the returned pointer
/// is valid for the same operations as `ptr`. In particular, if `ptr` is valid for some operation
/// on `A::Length`  values of `A::Item` with array layout, then the returned pointer is valid for
/// that operation on the corresponding slice.
#[track_caller]
pub(crate) const fn unsize_raw_mut<A: Array>(ptr: *mut A) -> *mut [A::Item] {
    core::ptr::slice_from_raw_parts_mut(ptr.cast(), arr_len::<A>())
}

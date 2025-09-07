//! Items related to implementation details of arrays.

use core::ptr;

use crate::Uint;
use crate::array::{helper::*, *};

pub struct IntoIter<T, N: Uint> {
    pub(crate) deq: ArrDeq<T, N>,
}

#[derive(Debug, Clone, Copy)]
pub struct TryFromSliceError(pub(crate) ());

pub const fn unsize_raw<A: Array>(ptr: *const A) -> *const [A::Item] {
    ptr::slice_from_raw_parts(ptr.cast(), arr_len::<A>())
}
pub const fn unsize_raw_mut<A: Array>(ptr: *mut A) -> *mut [A::Item] {
    ptr::slice_from_raw_parts_mut(ptr.cast(), arr_len::<A>())
}
pub const fn unsize_nonnull<A: Array>(ptr: core::ptr::NonNull<A>) -> core::ptr::NonNull<[A::Item]> {
    core::ptr::NonNull::slice_from_raw_parts(ptr.cast(), arr_len::<A>())
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Concat<A, B>(pub A, pub B);

// SAFETY: `repr(C)` results in the arrays being placed next to each other in memory
// in accordance with array layout
unsafe impl<T, A: Array<Item = T>, B: Array<Item = T>> Array for Concat<A, B> {
    type Item = T;
    type Length = crate::ops::Add<A::Length, B::Length>;
}
impl<T, A: Array<Item = T>, B: Array<Item = T>> ArraySealed for Concat<A, B> {}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Flatten<A>(pub A);

// SAFETY: `[[T; M]; N]` is equivalent to `[T; M * N]`
unsafe impl<A: Array<Item = B>, B: Array> Array for Flatten<A> {
    type Item = B::Item;
    type Length = crate::ops::Mul<A::Length, B::Length>;
}
impl<A: Array<Item = B>, B: Array> ArraySealed for Flatten<A> {}

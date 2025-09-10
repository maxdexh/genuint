//! Items related to implementation details of arrays.

use core::mem::ManuallyDrop;

use crate::{Uint, array::*};

pub struct IntoIter<T, N: Uint> {
    pub(crate) deq: ArrDeq<T, N>,
}

#[derive(Debug, Clone, Copy)]
pub struct TryFromSliceError(pub(crate) ());

/// Adapter that turns two arrays with the same item type into one long array.
///
/// This is just a `repr(C)` pair.
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

impl<A, B> Concat<A, B> {
    /// Returns the fields of this struct as a pair of arrays wrapped in [`ManuallyDrop`].
    ///
    /// May make it easier to destructure the result in `const` contexts.
    #[must_use = "The pair returned by this function is wrapped in ManuallyDrop and may need cleanup"]
    pub const fn into_man_drop(self) -> (ManuallyDrop<A>, ManuallyDrop<B>) {
        // SAFETY: `self` is passed by value and can be destructed by read
        unsafe {
            crate::utils::destruct_read!(Concat, (lhs, rhs), self);
            (ManuallyDrop::new(lhs), ManuallyDrop::new(rhs))
        }
    }
}

/// Adapter that turns an array of arrays into one long array of items.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Flatten<A>(pub A);

impl<A> Flatten<A> {
    /// Returns the field of this struct.
    pub const fn into_inner(self) -> A {
        // SAFETY: `self` is passed by value and can be destructed by read
        unsafe {
            crate::utils::destruct_read!(Flatten, (inner), self);
            inner
        }
    }
}
// SAFETY: repr(transparent), `[[T; M]; N]` is equivalent to `[T; M * N]`
unsafe impl<A: Array<Item = B>, B: Array> Array for Flatten<A> {
    type Item = B::Item;
    type Length = crate::ops::Mul<A::Length, B::Length>;
}
impl<A: Array<Item = B>, B: Array> ArraySealed for Flatten<A> {}

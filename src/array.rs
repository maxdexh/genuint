use core::marker::PhantomData;

/// # Safety
/// 1. `Self` must not have *any* safety invariants over arrays. It must be safe
///    to implement any auto traits for (arrays of) it if the item type implements them, regardless
///    of the safety invariants of the auto trait (even `unsafe` auto traits) and this is not limited
///    to auto traits from the standard library.
///     - This implies that it must not have a non-trivial [`Drop`] implementation and that it
///       should inherit the drop glue of its items.
/// 2. `Self` has the same layout and ownership semantics as `[Item; to_usize::<Length>().unwrap()]`.
///    Even if `Length` exceeds the maximum `usize`, it still must behave *as if* it had
///    ownership over exactly `Length` items.
///     - Note that the layout requirements make it impossible to construct an array of size
///       greater than [`isize::MAX`] unless `Item` (and therefore `Self`) is a ZST.
///     - Note that even for `ZST`s, the layout requirement still includes the alignment of an
///       array, which is always the same as that of the item.
/// 3. `Self` must be a `repr(transparent)` or `repr(C)` struct consisting only `Item`, arrays
///    of `Item`, arrays of arrays of `Item`, etc.
///     - Note that due to the layout requirement, it is almost never valid for `Self` to be
///       fieldless, since this will generally produce the wrong alignment.
///     - [`ManuallyDrop`] is not considered valid for this purpose.
///       Instead, an array wrapped in [`ManuallyDrop`], is considered an array of [`ManuallyDrop`].
///     - An array wrapped in [`MaybeUninit`] is considered an array of [`MaybeUninit`]
///     - This may be extended to other wrappers in the future where it makes sense.
///
///
/// [`ManuallyDrop`]: core::mem::ManuallyDrop
/// [`MaybeUninit`]: core::mem::MaybeUninit
pub unsafe trait Array {
    type Item;
    type Length: crate::Uint;
}

pub mod extra;

// SAFETY: Allowed by definition
unsafe impl<T, N: crate::Uint, const L: usize> Array for [T; L]
where
    crate::consts::ConstUsize<L>: crate::ToUint<ToUint = N>,
{
    type Item = T;
    type Length = N;
}
// SAFETY: Allowed by definition
unsafe impl<A: Array> Array for core::mem::ManuallyDrop<A> {
    type Item = core::mem::ManuallyDrop<A::Item>;
    type Length = A::Length;
}
// SAFETY: Allowed by definition
unsafe impl<A: Array> Array for core::mem::MaybeUninit<A> {
    type Item = core::mem::MaybeUninit<A::Item>;
    type Length = A::Length;
}
// SAFETY: repr(transparent)
unsafe impl<A: Array> Array for ArrApi<A> {
    type Item = A::Item;
    type Length = A::Length;
}

pub use crate::internals::arr_reexports::*;

mod core_impl;
mod extra_impl;

/// A wrapper for an array implementor that provides all of the API relating to arrays.
///
/// Through its second generic parameter (which is always the item type of the array),
/// it also provides better lifetime inferrence than using the inner type directly.
/// For example, `ArrApi<A>: 'a` implies `T: 'a` for the compiler, which is useful
/// when returning references to items. Note that `A: 'a` does not always imply this,
/// particularly when `A` is generic.
///
/// Once const traits become stabilized, the inherent methods may also be duplicated
/// as default methods in [`Array`].
#[repr(transparent)]
pub struct ArrApi<A: Array<Item = T>, T = <A as Array>::Item>(pub(crate) A, PhantomData<T>);

mod arr_vec;

/// A wrapper for a [`MaybeUninit`](core::mem::MaybeUninit) array that acts as a [`Vec`]
/// (with limited capacity), as well as a drop guard for the initialized items.
#[cfg_attr(not(doc), repr(transparent))]
pub struct ArrVec<A: Array<Item = T>, T = <A as Array>::Item>(
    arr_vec::ArrVecDrop<A>,
    PhantomData<T>,
);

mod arr_deq;

/// A wrapper for a [`MaybeUninit`](core::mem::MaybeUninit) array that acts as a
/// [`VecDeque`](std::collections::VecDeque) (with limited capacity), as well as
/// a drop guard for the initialized items.
#[repr(transparent)]
pub struct ArrDeq<A: Array<Item = T>, T = <A as Array>::Item>(
    arr_deq::ArrDeqDrop<A>,
    PhantomData<T>,
);

/// Helper macro that drops an [`ArrApi`], [`ArrVec`] or [`ArrDeq`], including in
/// const contexts, by dropping each of its items.
///
/// Currently, dropping in const contexts is only possible if the item type does
/// not have any drop glue or implementation. This macro is preferrable over
/// [`core::mem::forget`] in that it will give a compile error if the item type
/// cannot be dropped in the current context.
///
/// Once `const Destruct` bounds become stabilized, this macro can be rewritten
/// to drop the items in place.
#[macro_export]
#[doc(hidden)]
macro_rules! __drop_items {
    [ $arr:expr ] => {{
        let mut __guard = $crate::__mac::ArrDrop($arr).enter();
        while __guard.has_next() {
            let _ = __guard.pop_next();
        }
        __guard.discard();
    }};
}
pub use __drop_items as drop_items;

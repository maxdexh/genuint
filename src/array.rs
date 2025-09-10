//! Provides a drop-in replacement for builtin `[T; N]` arrays that uses a [`Uint`](crate::Uint)
//! for its length parameter

pub mod extra;

use crate::internals::ArraySealed;

// TODO: Document guarantees

/// # Safety
/// Currently cannot be implemented by downstream crates.
pub unsafe trait Array: Sized + ArraySealed {
    type Item;
    type Length: crate::Uint;
}

pub(crate) mod helper;

// SAFETY: By definition
unsafe impl<T, const N: usize> Array for [T; N]
where
    crate::consts::ConstUsize<N>: crate::ToUint,
{
    type Item = T;
    type Length = crate::uint::FromUsize<N>;
}
impl<T, const N: usize> ArraySealed for [T; N] where crate::consts::ConstUsize<N>: crate::ToUint {}

// SAFETY: MaybeUninit<[T; N]> is equivalent to [MaybeUninit<T>; N]
unsafe impl<A: Array> Array for core::mem::MaybeUninit<A> {
    type Item = core::mem::MaybeUninit<A::Item>;
    type Length = A::Length;
}
impl<A: Array> ArraySealed for core::mem::MaybeUninit<A> {}

// SAFETY: repr(transparent)
unsafe impl<A: Array> Array for ArrApi<A> {
    type Item = A::Item;
    type Length = A::Length;
}
impl<A: Array> ArraySealed for ArrApi<A> {}

// TODO: Move types into own module
pub use crate::internals::array_types::*;

mod impls;

/// A wrapper for an array implementor that provides all of the API relating to arrays.
///
/// The struct has a second generic parameter which is always the item of the array.
/// This gives better lifetime inferrence for the item type. Some methods, such as
/// [`Self::each_ref`] and the [`Index`](core::ops::Index) impl would not compile
/// the way they are written without it.
///
/// Once const traits become stabilized, the inherent methods may also be duplicated
/// as default methods in [`Array`].
#[repr(transparent)]
pub struct ArrApi<A: Array<Item = T>, T = <A as Array>::Item> {
    pub inner: A,
}

mod arr_vec;

/// A wrapper for a [`MaybeUninit`](core::mem::MaybeUninit) array that acts as a [`Vec`]
/// (with limited capacity), as well as a drop guard for the initialized items.
///
/// Note that unlike [`ArrApi`], all methods on this type may panic if the array length
/// exceeds [`usize::MAX`], without explicitly mentioning this in their docs.
///
/// # Drop implementation
/// This type currently has drop glue that does nothing except drop its elements, regardless
/// of whether the item type needs to be dropped.
/// This may be annoying in some `const` code as there is currently no way to make the `Drop`
/// implementation `const` for item types that can be dropped in `const`.
///
/// These workarounds exist:
/// - Using [`drop_items`]/[`assert_empty`](Self::assert_empty) if it's just a local variable
///   that needs to be dropped.
/// - Wrapping this type in [`ManuallyDrop`](core::mem::ManuallyDrop) if the item type is known
///   to have no drop glue. The contents of [`ManuallyDrop`](core::mem::ManuallyDrop) can be
///   accessed in `const` using [`const_util::mem::man_drop_mut`].
/// - Using [`Arr`]/[`CopyArr`] instead if the item type has a default value, or a layout niche
///   with [`Option`].
#[cfg_attr(not(doc), repr(transparent))]
pub struct ArrVecApi<A: Array<Item = T>, T = <A as Array>::Item>(
    arr_vec::ArrVecDrop<A>,
    core::marker::PhantomData<T>,
);

/// Alias for [`ArrVecApi`] around [`Arr`].
pub type ArrVec<T, N> = ArrVecApi<Arr<T, N>>;

mod arr_deq;

/// A wrapper for a [`MaybeUninit`](core::mem::MaybeUninit) array that acts as a
/// [`VecDeque`](std::collections::VecDeque) (with limited capacity), as well as
/// a drop guard for the initialized items.
///
/// Note that unlike [`ArrApi`], all methods on this type may panic if the array length
/// exceeds [`usize::MAX`], without explicitly mentioning this in their docs.
///
/// # Drop implementation
/// See [`ArrVecApi#drop-implementation`]
#[repr(transparent)]
pub struct ArrDeqApi<A: Array<Item = T>, T = <A as Array>::Item>(
    arr_deq::ArrDeqDrop<A>,
    core::marker::PhantomData<T>,
);

/// Alias for [`ArrDeqApi`] around [`Arr`].
pub type ArrDeq<T, N> = ArrDeqApi<Arr<T, N>>;

/// Helper macro that drops an [`ArrApi`], [`ArrVecApi`] or [`ArrDeqApi`], including in
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
        let mut __guard = $crate::__mac::arr::ArrDrop($arr).enter();
        while __guard.has_next() {
            let _ = __guard.pop_next();
        }
        __guard.discard();
    }};
}
pub use __drop_items as drop_items;

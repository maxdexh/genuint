//! Provides a drop-in replacement for builtin `[T; N]` arrays that uses a [`Uint`](crate::Uint)
//! for its length parameter

use crate::internals;

/// # Safety
/// Currently, this trait is sealed.
///
/// The guarantees made by types implementing [`Array`] with `Item = T` and `Length = N` include the following:
/// - Must have the same layout as an equivalent `[T; N]` builtin array. If `N > usize::MAX` (only
///   relevant for ZSTs), they still act as if there was such an array type.
/// - Arrays have no additional safety requirements over builtin arrays whatsoever. In particular:
///     - They have the same semantics as the equivalent builtin array with respect to arbitrary auto traits,
///       assuming there is no manual implementation from the crate declaring the trait.
///     - They also have the same semantics as the equivalent builtin array with respect to drop glue.
///       They never have a [`Drop`] implementation and only have the drop glue from their item type.
///       When the array is dropped, exactly `N` instances of `T` are dropped
///       ([in order](https://doc.rust-lang.org/reference/destructors.html)), even if `N > usize::MAX`.
///     - Note that together with the point about the layout, this is sufficient to perform arbitrary
///       casts and transmutes between equivalent array types. See the [`convert`] module.
/// - `MaybeUninit<[T; N]>` and `[MaybeUninit<T>; N]` are considered equivalent for the purposes of
///   this trait.
/// - Arrays of arrays are equivalent to their flattened versions, e.g. `[[i32; 4]; 3]` is
///   equivalent to `[i32; 12]`, which is equivalent to `[[i32; 3]; 4]`.
pub unsafe trait Array: Sized + internals::ArraySealed {
    /// The item type of the array.
    type Item;
    /// The length of the array as a type-level integer.
    type Length: crate::Uint;
}

pub use crate::internals::array_types::*;

/// A newtype adapter for an array implementor that the API relating to arrays.
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
    /// The array being wrapped.
    ///
    /// If you are getting errors trying to move out of this in `const` contexts, try using
    /// [`Self::into_inner`].
    pub inner: A,
}

/// Adapter that turns two arrays with the same item type into one long array.
///
/// This is just a `repr(C)` pair.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Concat<A, B>(pub A, pub B);

/// Adapter that turns an array of arrays into one long array of items.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Flatten<A>(pub A);

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
pub struct ArrVecApi<A: Array<Item = T>, T = <A as Array>::Item>(
    // SAFETY INVARIANT: See ArrVecRepr
    arr_vec::ArrVecDrop<A>,
    core::marker::PhantomData<T>,
);

/// Alias for [`ArrVecApi`] around [`Arr`].
pub type ArrVec<T, N> = ArrVecApi<Arr<T, N>>;

/// A wrapper for a [`MaybeUninit`](core::mem::MaybeUninit) array that acts as a
/// [`VecDeque`](std::collections::VecDeque) (with limited capacity), as well as
/// a drop guard for the initialized items.
///
/// Note that unlike [`ArrApi`], all methods on this type may panic if the array length
/// exceeds [`usize::MAX`], without explicitly mentioning this in their docs.
///
/// # Drop implementation
/// See [`ArrVecApi#drop-implementation`]
pub struct ArrDeqApi<A: Array<Item = T>, T = <A as Array>::Item>(
    // SAFETY INVARIANT: See ArrDeqRepr
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

pub(crate) mod helper;

mod arr_deq;
mod arr_vec;
mod impls;

pub mod convert;

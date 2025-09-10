use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{Uint, array::*, const_fmt, uint};

/// ```rust_analyzer_prefer_brackets
/// ImplArr![]
/// ```
macro_rules! ImplArr {
    [ $T:ty; $N:ty $(; $($extra_bounds:tt)*)? ] => {
        impl $crate::array::Array<Item = $T, Length = $N> $(+ $($extra_bounds)*)?
    };
}
pub(crate) use ImplArr;

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

pub(crate) const fn init_fill<T: Copy>(mut buf: &mut [MaybeUninit<T>], item: T) {
    while let [first, rest @ ..] = buf {
        *first = MaybeUninit::new(item);
        buf = rest;
    }
}
pub(crate) const fn init_fill_const<C: type_const::Const>(mut buf: &mut [MaybeUninit<C::Type>]) {
    while let [first, rest @ ..] = buf {
        *first = MaybeUninit::new(C::VALUE);
        buf = rest;
    }
}

/// `Src::Length == DST`
pub(crate) const unsafe fn arr_to_builtin_unchecked<Src: Array, const DST: usize>(
    src: Src,
) -> [Src::Item; DST] {
    // SAFETY: `Array` invariant
    unsafe { crate::utils::exact_transmute(src) }
}

/// # Panics
/// If `A::Length > usize::MAX`
///
/// # Safety
/// This operation is strictly the same as [`ptr::slice_from_raw_parts_mut`] with `ptr.cast()` as
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

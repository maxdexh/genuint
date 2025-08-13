//! Items related to implementation details of arrays.

use crate::array::*;

use crate::uint;

// The array type that is used internally (for consistency and to minimize monomorphizations)
pub(crate) type CanonArr<T, N> = Arr<T, N>;
pub(crate) type CanonVec<T, N> = ArrVec<CanonArr<T, N>>;
pub(crate) type CanonDeq<T, N> = ArrDeq<CanonArr<T, N>>;

#[repr(C)]
pub(crate) struct ArrConcat<A, B>(pub A, pub B);

unsafe impl<T, A: Array<Item = T>, B: Array<Item = T>> Array for ArrConcat<A, B> {
    type Item = T;
    type Length = crate::ops::Add<A::Length, B::Length>;
}

pub(crate) const fn phys_idx(logical: usize, cap: usize) -> usize {
    debug_assert!(logical == 0 || logical < 2 * cap);
    let phys = if logical >= cap {
        logical - cap
    } else {
        logical
    };
    debug_assert!(phys == 0 || phys < cap);
    phys
}

/// ```rust_analyzer_bracket_infer
/// ImplArr![]
/// ```
macro_rules! ImplArr {
    [ $T:ty; $N:ty $(; $($extra_bounds:tt)*)? ] => {
        impl $crate::array::Array<Item = $T, Length = $N> $(+ $($extra_bounds)*)?
    };
}
pub(crate) use ImplArr;

pub(crate) const fn arr_len<A: Array>() -> usize {
    match uint::to_usize::<A::Length>() {
        Some(n) => n,
        None => panic!("{}", uint::to_str::<A::Length>()),
    }
}

pub const fn from_slice<A: Array>(slice: &[A::Item]) -> Result<&A, TryFromSliceError> {
    if arr_len::<A>() == slice.len() {
        Ok(unsafe { &*slice.as_ptr().cast() })
    } else {
        Err(TryFromSliceError(()))
    }
}

pub const fn from_mut_slice<A: Array>(slice: &mut [A::Item]) -> Result<&mut A, TryFromSliceError> {
    if arr_len::<A>() == slice.len() {
        Ok(unsafe { &mut *slice.as_mut_ptr().cast() })
    } else {
        Err(TryFromSliceError(()))
    }
}

pub const fn retype<Src, Dst>(src: Src) -> Dst
where
    Src: Array,
    Dst: Array<Item = Src::Item, Length = Src::Length>,
{
    unsafe { crate::utils::exact_transmute(src) }
}

/// # Safety
/// `Src::Length == Dst::Length`
pub const unsafe fn retype_unchecked<Src, Dst>(src: Src) -> Dst
where
    Src: Array,
    Dst: Array<Item = Src::Item>,
{
    debug_assert!(
        crate::uint::cmp::<Src::Length, Dst::Length>().is_eq(),
        "Array length mismatch"
    );
    unsafe { crate::utils::exact_transmute(src) }
}

#[derive(Debug, Copy, Clone)]
pub struct TryFromSliceError(());

/// Maps `ArrApi<A>` to `A`.
///
/// This can be used to get the "internal" inner types of type alias wrappers for `ArrApi`,
/// such as [`Arr`]. Note that while these are not private as they can be accessed using
/// traits (see below), they implement nothing except [`Copy`], [`Clone`] and [`Array`].
///
/// This type alias is not magic; it is literally defined as
/// ```
/// # use generic_uint::array::{Array, ArrApi};
/// trait _ArrApi { type Inner; }
/// impl<A: Array> _ArrApi for ArrApi<A> { type Inner = A; }
/// type ArrApiInner<ArrApi> = <ArrApi as _ArrApi>::Inner;
/// ```
pub type ArrApiInner<ArrApi> = <ArrApi as crate::internals::_ArrApi>::Inner;

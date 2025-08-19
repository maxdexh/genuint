//! Items related to implementation details of arrays.

use crate::array::*;

use crate::uint;

// The array type that is used internally (for consistency and to minimize monomorphizations)
pub(crate) type CanonArr<T, N> = Arr<T, N>;
pub(crate) type CanonVec<T, N> = ArrVec<CanonArr<T, N>>;
pub(crate) type CanonDeq<T, N> = ArrDeq<CanonArr<T, N>>;

pub(crate) const fn into_canon<A: Array>(arr: A) -> CanonArr<A::Item, A::Length> {
    arr_convert(arr)
}

pub(crate) const fn wrapping_idx(logical: usize, cap: usize) -> usize {
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

pub(crate) const fn from_slice<A: Array>(slice: &[A::Item]) -> Result<&A, TryFromSliceError> {
    if arr_len::<A>() == slice.len() {
        Ok(unsafe { &*slice.as_ptr().cast() })
    } else {
        Err(TryFromSliceError(()))
    }
}

pub(crate) const fn from_mut_slice<A: Array>(
    slice: &mut [A::Item],
) -> Result<&mut A, TryFromSliceError> {
    if arr_len::<A>() == slice.len() {
        Ok(unsafe { &mut *slice.as_mut_ptr().cast() })
    } else {
        Err(TryFromSliceError(()))
    }
}

const fn debug_array_invariant_check<A: Array>() {
    debug_assert!(align_of::<A>() == align_of::<A::Item>());
    debug_assert!(match uint::to_usize::<A::Length>() {
        Some(len) => size_of::<A::Item>().checked_mul(len).unwrap() == size_of::<A>(),
        None => size_of::<A>() == 0 && size_of::<A::Item>() == 0,
    })
}

pub(crate) const fn arr_convert<Src, Dst>(src: Src) -> Dst
where
    Src: Array,
    Dst: Array<Item = Src::Item, Length = Src::Length>,
{
    debug_array_invariant_check::<Src>();
    debug_array_invariant_check::<Dst>();
    unsafe { arr_convert_unchecked(src) }
}

/// # Safety
/// `Src::Length == Dst::Length`
pub(crate) const unsafe fn arr_convert_unchecked<Src, Dst>(src: Src) -> Dst
where
    Src: Array,
    Dst: Array<Item = Src::Item>,
{
    unsafe { crate::utils::exact_transmute(src) }
}

#[track_caller]
pub(crate) const fn assert_same_arr_len<Src: Array, Dst: Array>() {
    debug_array_invariant_check::<Src>();
    debug_array_invariant_check::<Dst>();

    if let Some(err) = array_length_mismatch::<Src, Dst>() {
        panic!("{}", err)
    }
}
const fn array_length_mismatch<Src: Array, Dst: Array>() -> Option<&'static str> {
    struct Message<A, B>(A, B);
    impl<Src: Array, Dst: Array> type_const::Const for Message<Src, Dst> {
        type Type = &'static [&'static str];
        const VALUE: Self::Type = &[
            "Array length mismatch. Source array has length ",
            uint::to_str::<Src::Length>(),
            ", destination array type has length ",
            uint::to_str::<Dst::Length>(),
            ".",
        ];
    }
    const {
        if uint::cmp::<Src::Length, Dst::Length>().is_ne() {
            Some(if cfg!(feature = "uint-panic-values") {
                const_util::concat::concat_strs::<Message<Src, Dst>>()
            } else {
                "Array length mismatch"
            })
        } else {
            None
        }
    }
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

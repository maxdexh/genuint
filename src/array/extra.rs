//! Items related to implementation details of arrays.

use crate::{Uint, array::*, uint};

use crate::const_fmt::{concat_fmt, concat_fmt_if};

// The array type that is used internally (for consistency and to minimize monomorphizations)
pub(crate) type CanonArr<T, N> = Arr<T, N>;
pub(crate) type CanonVec<T, N> = ArrVecApi<CanonArr<T, N>>;
pub(crate) type CanonDeq<T, N> = ArrDeqApi<CanonArr<T, N>>;

pub(crate) const fn into_canon<A: Array>(arr: A) -> CanonArr<A::Item, A::Length> {
    arr_convert(arr)
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

#[track_caller]
pub(crate) const fn arr_len<A: Array>() -> usize {
    const fn doit<N: Uint>() -> usize {
        let precalc = const {
            if let Some(n) = uint::to_usize::<N>() {
                Ok(n)
            } else {
                Err(concat_fmt![
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

pub(crate) const fn from_slice<A: Array>(slice: &[A::Item]) -> Result<&A, TryFromSliceError> {
    if arr_len::<A>() == slice.len() {
        // SAFETY: `Array` implementors are valid for casts from arrays of correct size.
        // Since `slice` has the correct size, casting the pointer like this is valid.
        Ok(unsafe { &*slice.as_ptr().cast() })
    } else {
        Err(TryFromSliceError(()))
    }
}

pub(crate) const fn from_mut_slice<A: Array>(
    slice: &mut [A::Item],
) -> Result<&mut A, TryFromSliceError> {
    if arr_len::<A>() == slice.len() {
        // SAFETY: `Array` implementors are valid for casts from arrays of correct size.
        // Since `slice` has the correct size, casting the pointer like this is valid.
        Ok(unsafe { &mut *slice.as_mut_ptr().cast() })
    } else {
        Err(TryFromSliceError(()))
    }
}

/// Checks that `A` fulfills the required layout invariants wrt size and alignment
const fn debug_layout_invariant_check<A: Array>() {
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
    // SAFETY: By definition
    unsafe { arr_convert_no_len_check(src) }
}

/// # Safety
/// `Src::Length == Dst::Length`
pub(crate) const unsafe fn arr_convert_no_len_check<Src, Dst>(src: Src) -> Dst
where
    Src: Array,
    Dst: Array<Item = Src::Item>,
{
    debug_layout_invariant_check::<Src>();
    debug_layout_invariant_check::<Dst>();

    // SAFETY: `Array` invariant
    unsafe { crate::utils::exact_transmute(src) }
}

#[track_caller]
pub(crate) const fn assert_same_arr_len<Src: Array, Dst: Array>() {
    debug_layout_invariant_check::<Src>();
    debug_layout_invariant_check::<Dst>();

    #[track_caller]
    const fn doit<Src: Uint, Dst: Uint>() {
        let precalc = const {
            concat_fmt_if![
                uint::cmp::<Src, Dst>().is_ne(),
                "Array length mismatch. \n     Source length: ",
                PhantomData::<Src>,
                "\nDestination length: ",
                PhantomData::<Dst>,
            ]
        };
        if let Some(fmt) = precalc {
            fmt.panic();
        };
    }
    doit::<Src::Length, Dst::Length>();
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

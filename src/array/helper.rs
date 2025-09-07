use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{
    Uint,
    array::{extra::*, *},
    const_fmt, uint,
};

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

pub(crate) const fn arr_convert<Src, Dst>(src: Src) -> Dst
where
    Src: Array,
    Dst: Array<Item = Src::Item, Length = Src::Length>,
{
    // SAFETY: By definition
    unsafe { arr_convert_unchecked(src) }
}

/// # Safety
/// `Src::Length == Dst::Length`
pub(crate) const unsafe fn arr_convert_unchecked<Src, Dst>(src: Src) -> Dst
where
    Src: Array,
    Dst: Array<Item = Src::Item>,
{
    // SAFETY: `Array` invariant
    unsafe { crate::utils::exact_transmute(src) }
}

/// `Src::Length == DST`
pub(crate) const unsafe fn arr_to_builtin_unchecked<Src: Array, const DST: usize>(
    src: Src,
) -> [Src::Item; DST] {
    // SAFETY: `Array` invariant
    unsafe { crate::utils::exact_transmute(src) }
}

#[track_caller]
pub(crate) const fn assert_same_arr_len<Src: Array, Dst: Array>() {
    #[track_caller]
    const fn doit<Src: Uint, Dst: Uint>() {
        let precalc = const {
            if uint::cmp::<Src, Dst>().is_ne() {
                Some(const_fmt::fmt![
                    "Array length mismatch. \n     Source length: ",
                    PhantomData::<Src>,
                    "\nDestination length: ",
                    PhantomData::<Dst>
                ])
            } else {
                None
            }
        };
        if let Some(fmt) = precalc {
            fmt.panic();
        };
    }
    doit::<Src::Length, Dst::Length>();
}

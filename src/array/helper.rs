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

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

/// Checks that `A` fulfills the required layout invariants wrt size and alignment
macro_rules! check_layout {
    [ $($A:ty),* $(,)? ] => {
        const {
            $( crate::array::helper::_layout_check_intern::<$A>(); )+
        }
    };
}
pub(crate) use check_layout;

pub(crate) const fn _layout_check_intern<A: Array>() {
    struct Intern<A>(A);
    impl<A: Array> Intern<A> {
        const CHECK: &() = &{
            #[cfg(debug_assertions)]
            do_check_layout::<A>();
        };
    }
    *Intern::<A>::CHECK
}

#[cfg(debug_assertions)]
const fn do_check_layout<A: Array>() {
    use crate::const_fmt::ConstFmt;

    const fn __unwrap_one([one]: [ConstFmt; 1]) -> ConstFmt {
        one
    }
    macro_rules! fmt_one {
        ($one:expr) => {
            __unwrap_one(crate::const_fmt::ConstFmtWrap::new($one).fmt())
        };
    }

    macro_rules! fmt_one_each {
            ( $($ex:expr),* $(,)? ) => {
                [ $(fmt_one!($ex)),* ]
            };
        }
    macro_rules! fmt_one_slice {
            ( $($ex:expr),* $(,)? ) => {
                fmt_one!(&fmt_one_each![$($ex),*])
            };
        }
    macro_rules! fake_ty {
        ($ty:ty) => {{
            if false {
                #[allow(unused)]
                ::core::mem::size_of::<$ty>();
            };
            fmt_one!(stringify!($ty))
        }};
    }
    macro_rules! expr_and_val {
        ($val:expr) => {{
            let val = $val;
            (fmt_one!(stringify!($val)), val)
        }};
    }
    macro_rules! equals {
        ($expr:expr, $val:expr) => {
            fmt_one_slice!["\n", $expr, " = ", $val]
        };
    }

    const PREFIX: &str = "UB detected in `Array` implementor:\n";

    let (arr_align_expr, arr_align) = expr_and_val!(align_of::<A>());
    let (item_align_expr, item_align) = expr_and_val!(align_of::<A::Item>());
    if arr_align != item_align {
        const_fmt::do_panic_fmt(fmt_one_slice![
            PREFIX,
            "Array and item alignment must be the same.",
            equals!(arr_align_expr, arr_align),
            equals!(item_align_expr, item_align),
        ])
    }

    let (item_size_expr, item_size) = expr_and_val!(size_of::<A::Item>());
    let (arr_size_expr, arr_size) = expr_and_val!(size_of::<A>());

    let calc_size_result = match uint::to_usize::<A::Length>() {
        Some(len) => match item_size.checked_mul(len) {
            Some(calc_size) => {
                if calc_size == arr_size {
                    Ok(())
                } else {
                    Err(fmt_one!(calc_size))
                }
            }
            None => Err(fmt_one!("overflow")),
        },
        None => {
            if size_of::<A::Item>() != 0 {
                Err(fmt_one!("overflow"))
            } else if size_of::<A>() != 0 {
                Err(fmt_one!(0usize))
            } else {
                Ok(())
            }
        }
    };
    if let Err(calc_size) = calc_size_result {
        let len_expr = fake_ty!(A::Length);
        const_fmt::do_panic_fmt(fmt_one_slice![
            PREFIX,
            "Array size must equal array length times item size.",
            equals!(arr_size_expr, arr_size),
            equals!(fmt_one_slice![len_expr, " * ", item_size_expr], calc_size),
            equals!(len_expr, uint::to_str::<A::Length>()),
            equals!(item_size_expr, item_size),
        ])
    }
}

#[track_caller]
pub(crate) const fn arr_len<A: Array>() -> usize {
    check_layout!(A);
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
    check_layout!(A);

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
    check_layout!(A);

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
    check_layout!(Src, Dst);

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
    check_layout!(Src, Dst);

    // SAFETY: `Array` invariant
    unsafe { crate::utils::exact_transmute(src) }
}

/// `Src::Length == DST`
pub(crate) const unsafe fn arr_to_builtin_unchecked<Src: Array, const DST: usize>(
    src: Src,
) -> [Src::Item; DST] {
    check_layout!(Src);

    // SAFETY: `Array` invariant
    unsafe { crate::utils::exact_transmute(src) }
}

#[track_caller]
pub(crate) const fn assert_same_arr_len<Src: Array, Dst: Array>() {
    check_layout!(Src, Dst);

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

use super::*;

// SubIfGe(L, R) := if L >= R { L - R } else { L } = if L < R { L } else { L - R }
#[apply(lazy)]
pub type SubIfGeL<L, R> = If<
    //
    cmp::LtL<L, R>,
    L,
    sub::USubL<L, R>,
>;

// H := H(L), P := P(L)
//
// URemIn(L, R) := 2 * (H % R) + P
//
// L % R = (2 * H + P) % R
//       = (2 * (H % R) + P) % R
//       = URemIn(L, R) % R
pub type URemInSL<L, R> = AppendBit<
    //
    RemUncheckedL<H<L>, R>,
    P<L>,
>;

#[apply(lazy)]
// 0 % R = 0. We also get L % R = URem(L, R) % R
pub type URemL<L, R> = If<
    //
    L,
    URemInSL<L, R>,
    _0,
>;

// H % R <= R - 1
// thus URem(L, R) = 2 * (H % R) + P <= 2 * (H % R) + 1 <= 2 * R - 1
// thus L % R = URem(L, R) % R = SubIfGe(URem(L, R), R)
pub type RemUncheckedL<L, R> = SubIfGeL<
    //
    URemL<L, R>,
    R,
>;

#[apply(lazy)]
pub type DivUncheckedL<L, R> = If<
    //
    L,
    AppendBit<
        //
        DivUncheckedL<H<L>, R>,
        BitNot<cmp::LtL<URemL<L, R>, R>>,
    >,
    _0,
>;

#[apply(lazy)]
pub type RemL<L, R> = If<R, RemUncheckedL<L, R>, RemL<L, R>>;

#[apply(lazy)]
pub type DivL<L, R> = If<R, DivUncheckedL<L, R>, DivL<L, R>>;

/// Type-level remainder.
///
/// # Errors
/// Dividing by zero gives a "overflow while evaluating" error.
///
/// ```compile_fail
/// use generic_uint::{ops::Rem, uint, consts::*};
/// const _: fn(uint::From<Rem<_1, _0>>) = |_| {};
/// ```
#[apply(opaque)]
#[apply(test_op!
    test_rem,
    L % R,
    ..,
    1..
)]
pub type Rem<L, R> = RemL<L, R>;

/// Type-level division.
///
/// # Errors
/// Dividing by zero gives a "overflow while evaluating" error.
///
/// ```compile_fail
/// use generic_uint::{ops::Div, uint, consts::*};
/// const _: fn(uint::From<Div<_1, _0>>) = |_| {};
/// ```
#[apply(opaque)]
#[apply(test_op!
    test_div,
    L / R,
    ..,
    1..,
)]
pub type Div<L, R> = DivL<L, R>;

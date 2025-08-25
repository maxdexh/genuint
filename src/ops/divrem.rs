use super::*;

// SubIfGe(L, R) := if L >= R { L - R } else { L } = if L < R { L } else { L - R }
#[apply(lazy)]
pub type SubIfGeL<L, R> = Tern<
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
pub type URemInSL<L, R> = AppendL<
    //
    RemUncheckedL<H<L>, R>,
    P<L>,
>;

#[apply(lazy)]
// 0 % R = 0. We also get L % R = URem(L, R) % R
pub type URemL<L, R> = Tern<
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
pub type DivUncheckedL<L, R> = Tern<
    //
    L,
    AppendL<
        //
        DivUncheckedL<Half<L>, R>,
        BitNot<cmp::LtL<URemL<L, R>, R>>,
    >,
    _0,
>;

#[apply(lazy)]
pub type RemL<L, R> = Tern<R, RemUncheckedL<L, R>, RemL<L, R>>;

#[apply(lazy)]
pub type DivL<L, R> = Tern<R, DivUncheckedL<L, R>, DivL<L, R>>;

/// Dividing by zero gives a "overflow while evaluating" error.
///
/// ```compile_fail
/// use generic_uint::{ops::Rem, consts::*};
/// const _: fn(Rem<_1, _0>) = |_| {};
/// ```
#[apply(opaque)]
#[apply(test_op! test_rem: L.checked_rem(R).unwrap_or(0), .., 1..)]
pub type Rem<L, R> = RemL<L, R>;

/// Dividing by zero gives a "overflow while evaluating" error.
///
/// ```compile_fail
/// use generic_uint::{ops::Div, consts::*};
/// const _: fn(Div<_1, _0>) = |_| {};
/// ```
#[apply(opaque)]
#[apply(test_op! test_div: L.checked_div(R).unwrap_or(0), .., 1..)]
pub type Div<L, R> = DivL<L, R>;

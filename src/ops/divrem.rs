use super::*;

// SubIfGe(L, R) := if L >= R { L - R } else { L }
#[apply(lazy! sub_if_ge)]
type _SubIfGe<L, R> = If<
    _Lt<L, R>, // Ge is implemented on top of Lt, use Lt directly
    L,
    _SubUnchecked<L, R>,
>;

// H := H(L), P := P(L), R > 0
//
// NaiveRem(L, R) := 2 * (H % R) + P
//
// L % R = (2 * H + P) % R
//       = (H + H + P) % R
//       = ((H % R) + (H % R) + P) % R
//       = (2 * (H % R) + P) % R
//       = NaiveRem(L, R) % R
#[apply(lazy! naive)]
type _NaiveRem<L, R> = If<
    L,
    AppendBit<
        //
        _RemUnchecked<_H<L>, R>,
        _P<L>,
    >,
    // If L = 0, then NaiveRem(L, R) = 2 * (0 % R) + 0 = 0
    _0,
>;

// H := H(L), P := P(L), R > 0
//
// H % R <= R - 1
// thus NaiveRem(L, R) = 2 * (H % R) + P <= 2 * (H % R) + 1 <= 2 * R - 1
// thus normalizing NaiveRem is just a matter of subtracting R
//
// RemUnchecked(L, R) := L % R = NaiveRem(L, R) % R = SubIfGe(NaiveRem(L, R), R)
pub(crate) type _RemUnchecked<L, R> = _SubIfGe<_NaiveRem<L, R>, R>;

// H := H(L), P := P(L), R > 0
//
// DivUnchecked(L, R) := L / R
#[apply(lazy! unchecked_div)]
pub(crate) type _DivUnchecked<L, R> = If<
    //
    L,
    // Note that H = (H / R) * R + H % R, and
    // for any X, Y: (X * R + Y) / R = X + Y / R
    //
    // L / R = (2 * H + P) / R
    //       = (2 * ((H / R) * R + H % R) + P) / R
    //       = (2 * (H / R) * R + 2 * (H % R) + P) / R
    //       = (2 * (H / R) * R + 2 * (H % R) + P) / R
    //       = 2 * (H / R) + (2 * (H % R) + P) / R
    //       = 2 * (H / R) + NaiveRem(L, R) / R
    //
    // Since we still have NaiveRem(L, R) <= 2 * R - 1,
    // NaiveRem(L, R) / R = Ge(NaiveRem(L, R), R) <= 1.
    AppendBit<
        //
        _DivUnchecked<_H<L>, R>,
        // Use Lt here to match what we are doing above, since we already need to
        // go through this projection
        _IsZero<_Lt<_NaiveRem<L, R>, R>>,
    >,
    // 0 / R = 0
    _0,
>;

/// Type-level remainder operation.
///
/// # Errors
/// Dividing by zero gives a "overflow while evaluating" error.
///
/// ```compile_fail
/// use generic_uint::{ops::Rem, uint, consts::*};
/// const _: fn(uint::From<Rem<_1, _0>>) = |_| {};
/// ```
#[doc(alias = "%")]
#[doc(alias = "modulo")]
#[apply(opaque! rem_impl::_Rem)]
#[apply(test_op!
    test_rem,
    L % R,
    ..,
    1..
)]
pub type Rem<L, R> = If<
    R,
    _RemUnchecked<L, R>,
    // Recurse infinitely on div by 0
    _Rem<L, R>,
>;

/// Type-level division.
///
/// # Errors
/// Dividing by zero gives a "overflow while evaluating" error.
///
/// ```compile_fail
/// use generic_uint::{ops::Div, uint, consts::*};
/// const _: fn(uint::From<Div<_1, _0>>) = |_| {};
/// ```
#[doc(alias = "/")]
#[apply(opaque! div_impl::_Div)]
#[apply(test_op!
    test_div,
    L / R,
    ..,
    1..,
)]
pub type Div<L, R> = If<
    R,
    _DivUnchecked<L, R>,
    // Recurse infinitely on div by 0
    _Div<L, R>,
>;

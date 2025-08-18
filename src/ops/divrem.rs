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
    RemL<H<L>, R>,
    P<L>,
>;

#[apply(lazy)]
// 0 % R = 0. We also get L % R = URem(L, R) % R
pub type URemL<L, R> = Tern<
    //
    L,
    URemInSL<L, R>,
    U0,
>;

// H % R <= R - 1
// thus URem(L, R) = 2 * (H % R) + P <= 2 * (H % R) + 1 <= 2 * R - 1
// thus L % R = URem(L, R) % R = SubIfGe(URem(L, R), R)
pub type RemL<L, R> = SubIfGeL<
    //
    URemL<L, R>,
    R,
>;

#[apply(lazy)]
pub type DivL<L, R> = Tern<
    //
    L,
    AppendL<
        //
        DivL<Half<L>, R>,
        BitNot<cmp::LtL<URemL<L, R>, R>>,
    >,
    U0,
>;

#[apply(lazy)]
pub type CheckedRemL<L, R> = Tern<R, RemL<L, R>, CheckedRemL<L, R>>;

#[apply(lazy)]
pub type CheckedDivL<L, R> = Tern<R, DivL<L, R>, CheckedDivL<L, R>>;

#[apply(opaque)]
pub type Rem<L, R> = CheckedRemL<L, R>;

#[apply(opaque)]
pub type Div<L, R> = CheckedDivL<L, R>;

#[cfg(any(test, doctest))]
mod divrem_test {
    //! Dividing by zero gives a "overflow while evaluating" error.
    //!
    //! ```compile_fail
    //! use generic_uint::{ops::Div, consts::*};
    //! const _: fn(Div<U1, U0>) = |_| {};
    //! ```
    //!
    //! ```compile_fail
    //! use generic_uint::{ops::Rem, consts::*};
    //! const _: fn(Rem<U1, U0>) = |_| {};
    //! ```

    use super::*;

    #[apply(lazy)]
    pub type RemL<L, R> = Rem<L, R>;
    #[apply(test_op! test_rem: L.checked_rem(R).unwrap_or(0))]
    type RemOr0<L, R> = Tern<R, RemL<L, R>, U0>;

    #[apply(lazy)]
    pub type DivL<L, R> = Div<L, R>;
    #[apply(test_op! test_div: L.checked_div(R).unwrap_or(0))]
    type DivOr0<L, R> = Tern<R, DivL<L, R>, U0>;
}

use super::*;
// Notation: Cond!(C) := if C { 1 } else { 0 }
//           Cond!(C) = Cond!(D)  iff  (C iff D)
//           Cond!(C) = 1  iff  C

/// Type-level equality operator.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "==")]
#[apply(opaque! pub(self) eq_impl::_Eq)]
#[apply(test_op! test_eq, (L == R) as _)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// Eq(L, R) := Cond!(L == R)
pub type Eq<L, R> = If<
    L,
    //     L = R
    // iff 2 * HL + PL = 2 * HR + PR
    // iff PL = PR  and  HL = HR
    // iff Xnor(PL, PR) = 1  and  Eq(HL, HR) = 1
    // iff And(Xnor(PL, PR), Eq(HL, HR)) = 1
    _AndL<
        //
        _Xnor<_P<L>, _P<R>>,
        _Eq<_H<R>, _H<L>>,
    >,
    // case L = 0:  0 = R  iff  (if R { 0 } else { 1 }) = 1
    If<R, _0, _1>,
>;

/// Negated type-level equality operator.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "!=")]
#[apply(opaque! ne_impl::_Ne)]
pub type Ne<L, R> = _IsZero<Eq<L, R>>;

// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// LtByLast(L, R) := Cond!(HL = HR and PL = 0 and PR = 1)
type LtByLastL<L, R> = _AndSC<
    If<_P<L>, _0, _P<R>>, // Cond!(not PL and PR)
    _Eq<_H<L>, _H<R>>,    // Cond!(HL = HR)
>;

/// Type-level less-than operator.
///
/// This type will always be the same as [`Gt`] with swapped arguments.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<")]
#[apply(opaque! pub(crate) lt_impl::_Lt)]
#[apply(test_op! test_lt, (L < R) as _)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// Lt(L, R) := Cond!(L < R)
pub type Lt<L, R> = If<
    R,
    If<
        L,
        //     L < R
        // iff 2 * HL + PL < 2 * HR + PR
        // iff HL < HR or HL = HR and PL = 0 and PR = 1
        // iff Lt(HL, HR) = 1 or LtByLast(L, R) = 1
        If<_Lt<_H<L>, _H<R>>, _1, LtByLastL<L, R>>,
        // 0 < R is true because R = 0 was already checked
        _1,
    >,
    // L < 0 is false
    _0,
>;

/// Type-level greater-than operator.
///
/// This type will always be the same as [`Lt`] with swapped arguments.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = ">")]
pub type Gt<L, R> = Lt<R, L>;

/// Type-level greater-than-or-equal operator.
///
/// This type will always be the same as [`Le`] with swapped arguments.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = ">=")]
#[apply(opaque! ge_impl::_Ge)]
pub type Ge<L, R> = _IsZero<Lt<L, R>>;

/// Type-level less-than-or-equal operator.
///
/// This type will always be the same as [`Ge`] with swapped arguments.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<=")]
pub type Le<L, R> = Ge<L, R>;

/// Type-level [`min`](core::cmp::min) operator.
#[apply(opaque! min_impl::_Min)]
pub type Min<L, R> = If<_Lt<L, R>, R, L>;

/// Type-level [`max`](core::cmp::max) operator.
#[apply(opaque! max_impl::_Max)]
pub type Max<L, R> = If<_Lt<L, R>, L, R>;

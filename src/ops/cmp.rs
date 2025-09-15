use super::*;
// Notation: Cond!(C) := if C { 1 } else { 0 }
//           Cond!(C) = Cond!(D)  iff  (C iff D)
//           Cond!(C) = 1  iff  C

#[apply(lazy)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// Eq(L, R) := Cond!(L == R)
pub type EqL<L, R> = If<
    L,
    //     L = R
    // iff 2 * HL + PL = 2 * HR + PR
    // iff PL = PR  and  HL = HR
    // iff Xnor(PL, PR) = 1  and  Eq(HL, HR) = 1
    // iff And(Xnor(PL, PR), Eq(HL, HR)) = 1
    AndL<Xnor<P<L>, P<R>>, EqL<H<R>, H<L>>>,
    // case L = 0:  0 = R  iff  (if R { 0 } else { 1 }) = 1
    If<R, _0, _1>,
>;

/// Type-level equality operator.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "==")]
#[apply(opaque)]
#[apply(test_op! test_eq, (L == R) as _)]
pub type Eq<L, R> = EqL<L, R>;

/// Negated type-level equality operator.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "!=")]
#[apply(opaque)]
pub type Ne<L, R> = IsZero<Eq<L, R>>;

#[apply(lazy)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// LtByLast(L, R) := Cond!(HL = HR and PL = 0 and PR = 1)
pub type LtByLastL<L, R> = AndSC<
    If<P<L>, _0, P<R>>, // Cond!(not PL and PR)
    EqL<H<L>, H<R>>,    // Cond!(HL = HR)
>;

#[apply(lazy)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
// Lt(L, R) := Cond!(L < R)
pub type LtL<L, R> = If<
    R,
    If<
        L,
        //     L < R
        // iff 2 * HL + PL < 2 * HR + PR
        // iff HL < HR or HL = HR and PL = 0 and PR = 1
        // iff Lt(HL, HR) = 1 or LtByLast(L, R) = 1
        If<LtL<H<L>, H<R>>, _1, LtByLastL<L, R>>,
        // 0 < R is true because R = 0 was already checked
        _1,
    >,
    // L < 0 is false
    _0,
>;

/// Type-level less-than operator.
///
/// This type will always be the same as [`Gt`] with swapped arguments.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<")]
#[apply(opaque)]
#[apply(test_op! test_lt, (L < R) as _)]
pub type Lt<L, R> = LtL<L, R>;

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
#[apply(opaque! GeL)]
pub type Ge<L, R> = IsZero<Lt<L, R>>;

/// Type-level less-than-or-equal operator.
///
/// This type will always be the same as [`Ge`] with swapped arguments.
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<=")]
pub type Le<L, R> = Ge<L, R>;

/// Type-level `min` operator.
#[apply(opaque! MinL)]
pub type Min<L, R> = If<LtL<L, R>, R, L>;

/// Type-level `max` operator.
#[apply(opaque! MaxL)]
pub type Max<L, R> = If<LtL<L, R>, L, R>;

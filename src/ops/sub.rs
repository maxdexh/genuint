use super::*;

#[apply(lazy! unchecked)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R), C <= 1, L <= R + C.
// Result is unspecified for malformed input.
//
// SubUnchecked(L, R, C) := L - R - C
pub(crate) type _SubUnchecked<L, R, C = _0> = If<
    R,
    // This is a variation of binary addition.
    //
    //   L - R - C
    // = 2 * HL + PL - (2 * HR + PR) - C
    // = 2 * (HL - HR) + PL - PR - C
    //
    // X := PL - PR - C, so -2 <= X <= 1.
    //
    // Using euclidian or floor divmod, which are identical for positive divisors,
    // - X = 2 * (X / 2) + X % 2
    // - CC := -(X / 2), so 0 <= CC <= 1 and CC is either 0 or 1
    // - X % 2 is either 0 or 1 because of euclidian mod
    //
    // Hence, X = 2 * (X / 2) + X % 2 = - 2 * CC + X % 2 and
    //   L - R - C
    // = 2 * (HL - HR) - 2 * CC + X % 2
    // = 2 * (HL - HR - CC) + X % 2
    // = Append(SubUnchecked(HL, HR, CC), X % 2)
    PushBit<
        _SubUnchecked<
            _H<L>,
            _H<R>,
            // Normalize recursive argument
            uint::From<
                // Because CC is -(X / 2) using floor division, we have X / 2 < 0  iff  X < 0.
                // Thus, CC = 1  iff  CC > 0  iff  X / 2 < 0  iff  X < 0  iff  PL < PR + C
                If<
                    _P<L>,
                    // PL = 1, so CC = 1  iff  1 < PR + C  iff  PR = 1 and C = 1  iff  And(PR, C) = 1
                    _And<_P<R>, C>,
                    // PL = 0, so CC = 1  iff  0 < PR + C  iff  PR = 1  or C = 1  iff   Or(PR, C) = 1
                    _Or<_P<R>, C>,
                >,
            >,
        >,
        //   X % 2
        // = (PL - PR - C) % 2
        // = (PL + PR + C) % 2   (euclidian mod gives either 0 or 1)
        // = Xor(PL, PR, C)
        _Xor3<_P<L>, _P<R>, C>,
    >,
    // L - 0 - C = L - C = if C { DecUnchecked(L) } else { L }
    If<C, _DecUnchecked<L>, L>,
>;

/// Type-level [`abs_diff`](u128::abs_diff).
#[apply(test_op! test_abs_diff, L.abs_diff(R))]
#[apply(opaque! abs_diff::_AbsDiff)]
// AbsDiff(L, R) := |L - R| = if L < R { R - L } else { L - R }
pub type AbsDiff<L, R> = If<
    //
    _Lt<L, R>,
    _SubUnchecked<R, L>,
    _SubUnchecked<L, R>,
>;

/// Type-level [`saturating_sub`](u128::saturating_sub).
#[apply(opaque! sat_sub::_SatSub)]
#[apply(test_op! test_sat_sub, L.saturating_sub(R))]
pub type SatSub<L, R> = If<
    //
    _Lt<R, L>,
    _SubUnchecked<L, R>,
    _0,
>;

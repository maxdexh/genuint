use super::*;

#[apply(lazy)]
// This is a variation of binary addition.
//
// HL := H(L), PL := P(L), HR := H(R), PR := P(R), C <= 1.
//
// We also assume L <= R + C. For all other inputs, we do not care about the result.
//
// USub(L, R, C) := L - R - C
pub type USubL<L, R, C = _0> = Tern<
    R,
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
    // = Append(USub(HL, HR, CC), X % 2)
    AppendBit<
        USubL<
            H<L>,
            H<R>,
            // Because CC is -(X / 2) using floor division, we have X / 2 < 0  iff  X < 0.
            // Thus, CC = 1  iff  CC > 0  iff  X / 2 < 0  iff  X < 0  iff  PL < PR + C
            Tern<
                P<L>,
                // PL = 1, so CC = 1  iff  1 < PR + C  iff  PR = 1 and C = 1  iff  And(PR, C) = 1
                AndSC<P<R>, C>,
                // PL = 0, so CC = 1  iff  0 < PR + C  iff  PR = 1  or C = 1  iff   Or(PR, C) = 1
                OrSC<P<R>, C>,
            >,
        >,
        //   X % 2
        // = (PL - PR - C) % 2
        // = (PL + PR + C) % 2   (euclidian mod gives either 0 or 1)
        // = Xor(PL, PR, C)
        Xor3<P<L>, P<R>, C>,
    >,
    // R = 0, so L - 0 - C = L - C = SatDecIf(L, C)
    satdec::SatDecIfL<L, C>,
>;

/// Type-level [`u128::abs_diff`].
#[apply(opaque)]
#[apply(test_op! test_abs_diff, L.abs_diff(R))]
// AbsDiff(L, R) := |L - R| = if L < R { R - L } else { L - R }
pub type AbsDiff<L, R> = Tern<
    //
    cmp::LtL<L, R>,
    USubL<R, L>,
    USubL<L, R>,
>;

/// Type-level [`u128::saturating_sub`].
#[apply(opaque)]
#[apply(test_op! test_sat_sub, L.saturating_sub(R))]
pub type SatSub<L, R> = Tern<
    //
    cmp::LtL<R, L>,
    USubL<L, R>,
    _0,
>;

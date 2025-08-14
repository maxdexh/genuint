use super::*;

#[apply(lazy)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R), C <= 1, L <= R + C
//
// USub(L, R, C) := L - R - C
pub type USubL<L, R, C = U0> = Tern<
    R, // don't bother short-circuiting on L, since L <= R
    // L - R - C = 2 * HL + PL - (2 * HR + PR) - C
    //           = 2 * (HL - HR) + PL - PR - C
    //           = 2 * (HL - HR) + X
    //
    // where X := PL - PR - C, so -2 <= X <= 1.
    //
    // Using euclidian/floor divmod (identical for positive divisor):
    // X % 2 = (PL - PR - C) % 2 = (PL + PR + C) % 2 = Xor(PL, PR, C).
    // CC := -(X / 2), so 0 <= CC <= 1 because -1 <= X / 2 <= 0
    //
    // Hence, X = 2 * (X / 2) + X % 2 = - 2 * CC + Xor(PL, PR, C)
    //
    // L - R - C = 2 * (HL - HR) - 2 * CC + Xor(PL, PR, C)
    //           = 2 * (HL - HR - CC) + Xor(PL, PR, C)
    //           = Append(USub(HL, HR, CC), Xor(PL, PR, C))
    //
    AppendL<
        USubL<
            H<L>,
            H<R>,
            // Because CC is -(X / 2) using floor division, we have X / 2 < 0  iff  X < 0.
            // Thus, CC = 1  iff  CC > 0  iff  X / 2 < 0  iff  X < 0  iff  PL < PR + C
            Tern<
                P<L>,
                // case PL = 1:
                // CC = 1  iff  1 < PR + C  iff  PR = 1 and C = 1  iff  And(PR, C) = 1
                AndSC<P<R>, C>,
                // case PL = 0:
                // CC = 1  iff  0 < PR + C  iff  PR = 1  or C = 1  iff   Or(PR, C) = 1
                OrSC<P<R>, C>,
            >,
        >,
        Xor3<P<L>, P<R>, C>,
    >,
    // case R = 0: L - 0 - C = L - C = SatDecIf(L, C)
    satdec::SatDecIfL<L, C>,
>;

#[apply(opaque)]
// AbsDiff(L, R) := |L - R| = if L < R { R - L } else { L - R }
pub type AbsDiff<L, R> = Tern<
    //
    Lt<L, R>,
    USubL<R, L>,
    USubL<L, R>,
>;

test_op! { test_abs_diff: L R, AbsDiff<L, R>, L.abs_diff(R) }

#[apply(opaque)]
pub type SatSub<L, R> = Tern<
    //
    Gt<L, R>,
    USubL<L, R>,
    U0,
>;

test_op! { test_sat_sub: L R, SatSub<L, R>, L.saturating_sub(R) }

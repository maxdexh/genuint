use super::*;

#[apply(lazy! inc)]
// Inc(N) := N + 1
// Binary increment: flip bits at the end up to and including the last zero bit
pub(crate) type _Inc<N> = If<
    _P<N>, //
    AppendBit<_Inc<_H<N>>, _0>,
    AppendBit<_H<N>, _1>,
>;

#[apply(lazy! plus_bit)]
pub(crate) type _PlusBit<N, C> = If<C, _Inc<N>, N>;

#[apply(lazy! add_impl)]
// This is just binary addition.
//
// HL := H(L), PL := P(L), HR := H(R), PR := P(R), C <= 1.
//
// Add(L, R, C) := L + R + C
pub(crate) type _Add<L, R, C = _0> = If<
    L,
    //   L + R + C
    // = (2 * LH + LP) + (2 * RH + RP) + C
    // = 2 * (LH + RH) + (LP + RP + C)
    //
    // X := LP + RP + C. Since X = 2 * (X / 2) + X % 2, we get
    //   L + R + C
    // = 2 * (LH + RH + X / 2) + X % 2
    // = Append(LH + RH + X / 2, X % 2)
    AppendBit<
        // LH + RH + X / 2
        _Add<
            _H<L>,
            _H<R>,
            // Since X = LP + RP + C <= 3, we have X / 2 being either 0 or 1,
            // and therefore X / 2 = 1 iff LP + RP + C >= 2, else X / 2 = 0.
            If<
                //
                _P<L>,
                // LP = 1, so LP + RP + C >= 2 iff RP + C >= 1 iff RP = 1 or  C = 1
                _OrSC<_P<R>, C>,
                // LP = 0, so LP + RP + C >= 2 iff RP + C >= 2 iff RP = 1 and C = 1
                _AndSC<_P<R>, C>,
            >,
        >,
        // X % 2 is 1 iff X is odd, i.e. if an odd number of LP, RP, C are 1.
        // Hence X % 2 = Xor(LP, RP, C).
        _Xor3<_P<L>, _P<R>, C>,
    >,
    _PlusBit<R, C>,
>;

/// Type-level addition.
#[apply(opaque)]
#[apply(test_op! test_add, L + R)]
pub type Add<L, R> = _Add<L, R>;

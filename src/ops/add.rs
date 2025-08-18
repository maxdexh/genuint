use super::*;

#[apply(lazy)]
pub type IncIfL<N, C> = Tern<C, AppendL<IncIfL<H<N>, P<N>>, BitNot<P<N>>>, N>;

#[apply(lazy)]
// This is just binary addition.
//
// HL := H(L), PL := P(L), HR := H(R), PR := P(R), C <= 1.
//
// Add(L, R, C) := L + R + C
pub type AddL<L, R, C = U0> = Tern<
    L,
    //   L + R + C
    // = (2 * LH + LP) + (2 * RH + RP) + C
    // = 2 * (LH + RH) + (LP + RP + C)
    //
    // X := LP + RP + C. Since X = 2 * (X / 2) + X % 2, we get
    //   L + R + C
    // = 2 * (LH + RH + X / 2) + X % 2
    // = Append(LH + RH + X / 2, X % 2)
    AppendL<
        // LH + RH + X / 2
        AddL<
            H<L>,
            H<R>,
            // Since X = LP + RP + C <= 3, we have X / 2 being either 0 or 1,
            // and therefore X / 2 = 1 iff LP + RP + C >= 2, else X / 2 = 0.
            Tern<
                //
                P<L>,
                // LP = 1, so LP + RP + C >= 2 iff RP + C >= 1 iff RP = 1 or  C = 1
                OrSC<P<R>, C>,
                // LP = 0, so LP + RP + C >= 2 iff RP + C >= 2 iff RP = 1 and C = 1
                AndSC<P<R>, C>,
            >,
        >,
        // X % 2 is 1 iff X is odd, i.e. if an odd number of LP, RP, C are 1.
        // Hence X % 2 = Xor(LP, RP, C).
        Xor3<P<L>, P<R>, C>,
    >,
    IncIfL<R, C>,
>;

#[apply(opaque)]
#[apply(test_op! test_add: L + R)]
pub type Add<L, R> = AddL<L, R>;

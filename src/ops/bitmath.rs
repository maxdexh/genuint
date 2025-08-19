use super::*;

#[apply(lazy)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
pub type BitAndL<L, R> = Tern<
    L,
    // Because L is the result of appending LP to LH (and the same thing for R), and
    // LP and RP are suffixes of equal bit length (1), we have
    //
    // L & R = (2 * LH + LP) & (2 * RH + RP) = 2 * (LH & RH) + (RH & RP)
    AppendL<
        //
        BitAndL<H<R>, H<L>>, // LH & RH = RH & LH, switching will terminate faster
        AndSC<P<L>, P<R>>,
    >,
    // 0 & R = 0
    U0,
>;

#[apply(opaque)]
#[apply(test_op! test_bit_and: L & R)]
pub type BitAnd<L, R> = BitAndL<L, R>;

#[apply(lazy)]
pub type BitOrL<L, R> = Tern<
    L,
    // This works by analogy with BitAnd
    AppendL<
        //
        BitOrL<H<R>, H<L>>,
        OrSC<P<L>, P<R>>,
    >,
    // 0 | R = R
    R,
>;

#[apply(opaque)]
#[apply(test_op! test_bit_or: L | R)]
pub type BitOr<L, R> = BitOrL<L, R>;

#[apply(lazy)]
pub type BitXorL<L, R> = Tern<
    L,
    // This works by analogy with BitAnd
    AppendL<
        //
        BitXorL<H<R>, H<L>>,
        Xor<P<L>, P<R>>,
    >,
    // 0 ^ R = R
    R,
>;

#[apply(opaque)]
#[apply(test_op! test_bit_xor: L ^ R)]
pub type BitXor<L, R> = BitXorL<L, R>;

#[apply(lazy)]
pub type CountOnesL<N> = Tern<
    //
    N,
    add::IncIfL<
        //
        CountOnesL<H<N>>,
        P<N>,
    >,
    U0,
>;

#[apply(opaque)]
#[apply(test_op! test_count_ones: N.count_ones().into())]
pub type CountOnes<N> = CountOnesL<N>;

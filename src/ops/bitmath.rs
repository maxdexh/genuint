use super::*;

/// Type-level bitwise AND.
#[doc(alias = "&")]
#[apply(opaque! bit_and::_BitAnd)]
#[apply(test_op! test_bit_and, L & R)]
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
pub type BitAnd<L, R> = If<
    L,
    // Because L is the result of appending LP to LH (and the same thing for R), and
    // LP and RP are suffixes of equal bit length (1), we have
    //
    // L & R = (2 * LH + LP) & (2 * RH + RP) = 2 * (LH & RH) + (RH & RP)
    AppendBit<
        //
        _BitAnd<_H<R>, _H<L>>, // LH & RH = RH & LH, switching will terminate faster
        _AndSC<_P<L>, _P<R>>,
    >,
    // 0 & R = 0
    _0,
>;

/// Type-level bitwise OR.
#[doc(alias = "|")]
#[apply(opaque! bit_or::_BitOr)]
#[apply(test_op! test_bit_or, L | R)]
pub type BitOr<L, R> = If<
    L,
    // This works by analogy with BitAnd
    AppendBit<
        //
        _BitOr<_H<R>, _H<L>>,
        _OrSC<_P<L>, _P<R>>,
    >,
    // 0 | R = R
    R,
>;

/// Type-level bitwise XOR.
#[doc(alias = "^")]
#[apply(opaque! bit_xor::_BitXor)]
#[apply(test_op! test_bit_xor, L ^ R)]
pub type BitXor<L, R> = If<
    L,
    // This works by analogy with BitAnd
    AppendBit<
        //
        _BitXor<_H<R>, _H<L>>,
        _Xor<_P<L>, _P<R>>,
    >,
    // 0 ^ R = R
    R,
>;

/// Type-level [`count_ones`](u128::count_ones).
#[apply(opaque! count_ones::_CountOnes)]
#[apply(test_op! test_count_ones, N.count_ones().into())]
pub type CountOnes<N> = If<
    //
    N,
    add::_PlusBit<
        //
        _CountOnes<_H<N>>,
        _P<N>,
    >,
    _0,
>;

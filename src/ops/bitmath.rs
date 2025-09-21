use super::*;

// All bitwise operations are implemented the same way:
//
// Let BitWise(Op, L, R) be the result of all Op(L[i], L[i]) appended to each other.
//
// Then we have
// BitWise(Op, L, R) = BitWise(Op, Append(H(L), P(L)), Append(H(R), P(R)))
//                   = Append(BitWise(Op, H(L), H(R)), Op(P(L), P(R))
//
// Because H and P split the number into the part before and after the last bit.

/// Type-level bitwise AND.
#[doc(alias = "&")]
#[apply(opaque! bit_and::_BitAnd)]
#[apply(test_op! test_bit_and, L & R)]
// BitAnd(L, R) = BitWise(And, L, R), see above
pub type BitAnd<L, R> = If<
    L,
    PushBit<
        _BitAnd<_H<R>, _H<L>>, // A & B = B & A, switching will terminate faster
        _And<_P<L>, _P<R>>,
    >,
    // 0 & R = 0
    _0,
>;

/// Type-level bitwise OR.
#[doc(alias = "|")]
#[apply(opaque! bit_or::_BitOr)]
#[apply(test_op! test_bit_or, L | R)]
// BitOr(L, R) = BitWise(Or, L, R), see above
pub type BitOr<L, R> = If<
    L,
    PushBit<
        _BitOr<_H<R>, _H<L>>, // A | B = B | A
        _Or<_P<L>, _P<R>>,
    >,
    // 0 | R = R
    R,
>;

/// Type-level bitwise XOR.
#[doc(alias = "^")]
#[apply(opaque! bit_xor::_BitXor)]
#[apply(test_op! test_bit_xor, L ^ R)]
// BitXor(L, R) = BitWise(Xor, L, R), see above
pub type BitXor<L, R> = If<
    L,
    PushBit<
        //
        _BitXor<_H<R>, _H<L>>, // A ^ B = B ^ A
        _Xor<_P<L>, _P<R>>,
    >,
    // 0 ^ R = R
    R,
>;

/// Type-level [`count_ones`](u128::count_ones).
#[apply(opaque! count_ones::_CountOnes)]
#[apply(test_op! test_count_ones, N.count_ones().into())]
// CountOnes(N) := Number of one-bits in N = Sum of bits in N
pub type CountOnes<N> = If<
    N,
    // CountOnes(N) = CountOnes(Append(H(N), P(N)))
    //              = CountOnes(H(N)) + P(N)
    add::_PlusBit<
        //
        _CountOnes<_H<N>>,
        _P<N>,
    >,
    // CountOnes(0) = 0
    _0,
>;

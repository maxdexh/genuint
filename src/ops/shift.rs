use super::*;

// DoubleIf(N, C) := if C { 2 * N } else { N }
type _DoubleIf<N, C> = If<C, AppendBit<N, _0>, N>;

/// Type-level left bitshift.
#[doc(alias = "<<")]
#[apply(opaque! shl::_Shl)]
#[apply(test_op!
    test_shl,
    L << R,
    ..,
    ..=15,
)]
// Shl(L, R) := L << R = L * Pow(2, R)
//
// R = 2 * H + P, H = H(R), P = P(R)
pub type Shl<L, R> = If<
    R,
    //   Shl(L, R)
    // = Shl(L, 2 * H + P)
    // = Shl(L, H + H + P)
    // = L * Pow(2, H + H + P)
    // = L * Pow(2, H) * Pow(2, H) * Pow(2, P)
    // = if P { 2 * Shl(Shl(L, H), H) } else { Shl(Shl(L, H), H) }
    // = DoubleIf(Shl(Shl(L, H), H), P)
    _DoubleIf<
        // NOTE: From testing, this is the fastest known way to write this recursion
        _Shl<
            _Shl<
                //
                uint::From<L>,
                _H<R>,
            >,
            _H<R>,
        >,
        _P<R>,
    >,
    // L << 0 = L
    L,
>;

// HalfIf(N, C) := if C { H(N) } else { N }
type HalfIfL<N, C> = If<C, Half<N>, N>;

/// Type-level right bitshift.
#[doc(alias = ">>")]
#[apply(opaque! shr::_Shr)]
#[apply(test_op!
    test_shr,
    L >> R,
    ..,
    ..=15,
)]
// Shr(L, R) := L >> R = L / Pow(2, R)
//
// R = 2 * H + P, H = H(R), P = P(R)
pub type Shr<L, R> = If<
    // Also exit for L = 0, since we will reach that way more often than for Shl.
    // This also makes it much less likely for this to hit the recursion limit,
    // because that requires a large R, which might just make the result 0.
    _And<L, R>,
    //   Shr(L, R)
    // = Shr(L, 2 * H + P)
    // = Shr(L, H + H + P)
    // = L / Pow(2, H + H + P)
    // = L / Pow(2, H) / Pow(2, H) / Pow(2, P)
    // = if P { H(Shr(Shr(L, H), H)) } else { Shr(Shr(L, H), H) }
    // = HalfIf(Shr(Shr(L, H), H), P)
    HalfIfL<
        // NOTE: From testing, this is the fastest known way to write this recursion
        _Shr<
            _Shr<
                //
                uint::From<L>,
                _H<R>,
            >,
            _H<R>,
        >,
        _P<R>,
    >,
    // R = 0: L >> 0 = L
    // L = 0: L >> R = 0 >> R = 0 = L
    L,
>;

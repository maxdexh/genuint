use super::*;

// Quad(N) := 4 * N
type _Quad<N> = AppendBit<AppendBit<N, _0>, _0>;

// N = 2 * H + P, H = H(N), P = P(N)
//
// Square(N) := Pow(N, 2) = N * N
#[apply(lazy! square)]
type _Square<N> = If<
    N,
    If<
        _P<N>,
        // P = 1
        // Pow(N, 2) = Pow(2 * H + 1, 2) = 4 * Pow(H, 2) + 4 * H + 1
        add::_CarryAdd<
            //
            _Quad<_Square<_H<N>>>,
            _Quad<_H<N>>,
            _1,
        >,
        // P = 0
        // Pow(N, 2) = Pow(2 * H, 2) = 4 * Pow(H, 2)
        _Quad<_Square<_H<N>>>,
    >,
    // Pow(0, 2) = 0
    _0,
>;

// MulIf(N, F, C) := if C { N * F } else { N }
type _MulIf<N, F, C> = If<C, _Mul<F, N>, N>;

/// Type-level exponentiation
#[apply(opaque! pow_impl::_Pow)]
#[apply(test_op!
    test_pow,
    B.pow(E.try_into().unwrap()),
    ..,
    // Cap the exponent at 10 for tests
    ..=10,
)]
// E = 2 * H + P, H = H(E), P = P(E)
pub type Pow<B, E> = If<
    E,
    //   Pow(B, E)
    // = Pow(B, 2 * H + P)
    // = Pow(Pow(B, H), 2) * Pow(B, P)
    // = Square(Pow(B, H)) * if P { B } else { 1 }
    // = if P { Square(Pow(B, H)) * B } else { Square(Pow(B, H)) }
    // = MulIf(Square(Pow(B, H)), B, P)
    _MulIf<
        //
        _Pow<_Square<B>, _H<E>>,
        B,
        _P<E>,
    >,
    // Pow(B, 0) = 1 (including if B = 0)
    _1,
>;

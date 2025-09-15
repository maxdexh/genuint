use super::*;

// Quad(N) := 4 * N
type QuadL<N> = AppendBit<AppendBit<N, _0>, _0>;

// N = 2 * H + P, H = H(N), P = P(N)
//
// Square(N) := Pow(N, 2) = N * N
#[apply(lazy)]
pub type SquareL<N> = If<
    N,
    If<
        P<N>,
        // P = 1
        // Pow(N, 2) = Pow(2 * H + 1, 2) = 4 * Pow(H, 2) + 4 * H + 1
        add::AddL<
            //
            QuadL<SquareL<H<N>>>,
            QuadL<H<N>>,
            _1,
        >,
        // P = 0
        // Pow(N, 2) = Pow(2 * H, 2) = 4 * Pow(H, 2)
        QuadL<SquareL<H<N>>>,
    >,
    // Pow(0, 2) = 0
    _0,
>;

// MulIf(N, F, C) := if C { N * F } else { N }
type MulIfL<N, F, C> = If<C, mul::MulL<F, N>, N>;

// E = 2 * H + P, H = H(E), P = P(E)
#[apply(lazy)]
pub type PowL<B, E> = If<
    E,
    //   Pow(B, E)
    // = Pow(B, 2 * H + P)
    // = Pow(Pow(B, H), 2) * Pow(B, P)
    // = Square(Pow(B, H)) * if P { B } else { 1 }
    // = if P { Square(Pow(B, H)) * B } else { Square(Pow(B, H)) }
    // = MulIf(Square(Pow(B, H)), B, P)
    MulIfL<
        //
        PowL<SquareL<B>, H<E>>,
        B,
        P<E>,
    >,
    // Pow(B, 0) = 1 (including if B = 0)
    _1,
>;

/// Type-level exponentiation
#[apply(opaque)]
#[apply(test_op!
    test_pow,
    B.pow(E.try_into().unwrap()),
    ..,
    ..=10,
)]
pub type Pow<B, E> = PowL<B, E>;

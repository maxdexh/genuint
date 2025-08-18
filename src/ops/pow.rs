use super::*;

// Quad(N) := 4 * N
type QuadL<N> = AppendL<AppendL<N, U0>, U0>;

// N = 2 * H + P, H = H(N), P = P(N)
//
// Square(N) := Pow(N, 2) = N * N
#[apply(lazy)]
pub type SquareL<N> = Tern<
    N,
    TernL<
        P<N>,
        // P = 1
        // Pow(N, 2) = Pow(2 * H + 1, 2) = 4 * Pow(H, 2) + 4 * H + 1
        add::AddL<
            //
            QuadL<SquareL<H<N>>>,
            QuadL<H<N>>,
            U1,
        >,
        // P = 0
        // Pow(N, 2) = Pow(2 * H, 2) = 4 * Pow(H, 2)
        QuadL<SquareL<H<N>>>,
    >,
    // Pow(0, 2) = 0
    U0,
>;

// MulIf(N, F, C) := if C { N * F } else { N }
type MulIfL<N, F, C> = TernL<C, mul::MulL<N, F>, N>;

// E = 2 * H + P, H = H(E), P = P(E)
#[apply(lazy)]
pub type PowL<B, E> = Tern<
    E,
    //   Pow(B, E)
    // = Pow(B, 2 * H + P)
    // = Pow(Pow(B, H), 2) * Pow(B, P)
    // = Square(Pow(B, H)) * if P { B } else { 1 }
    // = if P { Square(Pow(B, H)) * B } else { Square(Pow(B, H)) }
    // = MulIf(Square(Pow(B, H)), B, P)
    MulIfL<
        //
        PowL<SquareL<B>, Half<E>>,
        B,
        P<E>,
    >,
    // Pow(B, 0) = 1 (including if B = 0)
    U1,
>;

#[apply(opaque)]
#[apply(test_op! test_pow: B.pow(E.try_into().unwrap()))]
pub type Pow<B, E> = PowL<B, E>;

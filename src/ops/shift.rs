use super::*;

// DoubleIf(N, C) := if C { 2 * N } else { N }
type DoubleIfL<N, C> = TernL<C, AppendL<N, _0>, N>;

// Shl(L, R) := L << R = L * Pow(2, R)
//
// R = 2 * H + P, H = H(R), P = P(R)
#[apply(lazy)]
pub type ShlL<L, R> = Tern<
    R,
    //   Shl(L, R)
    // = Shl(L, 2 * H + P)
    // = Shl(L, H + H + P)
    // = L * Pow(2, H + H + P)
    // = L * Pow(2, H) * Pow(2, H) * Pow(2, P)
    // = if P { 2 * Shl(Shl(L, H), H) } else { Shl(Shl(L, H), H) }
    // = DoubleIf(Shl(Shl(L, H), H), P)
    DoubleIfL<
        //
        ShlL<ShlL<L, H<R>>, H<R>>,
        P<R>,
    >,
    // Shl(L, 0) = L
    L,
>;

#[apply(opaque)]
#[apply(test_op! test_shl: L << R)]
pub type Shl<L, R> = ShlL<L, R>;

// HalfIf(N, C) := if C { H(N) } else { N }
type HalfIfL<N, C> = TernL<C, HalfL<N>, N>;

// Shr(L, R) := L >> R = L / Pow(2, R)
//
// R = 2 * H + P, H = H(R), P = P(R)
#[apply(lazy)]
pub type ShrL<L, R> = Tern<
    R,
    //   Shr(L, R)
    // = Shr(L, 2 * H + P)
    // = Shr(L, H + H + P)
    // = L / Pow(2, H + H + P)
    // = L / Pow(2, H) / Pow(2, H) / Pow(2, P)
    // = if P { H(Shr(Shr(L, H), H)) } else { Shr(Shr(L, H), H) }
    // = HalfIf(Shr(Shr(L, H), H), P)
    HalfIfL<
        //
        ShrL<ShrL<L, H<R>>, H<R>>,
        P<R>,
    >,
    L,
>;
#[apply(opaque)]
#[apply(test_op! test_shr: L >> R)]
pub type Shr<L, R> = ShrL<L, R>;

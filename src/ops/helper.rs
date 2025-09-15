use super::*;

// Short-circuiting And
pub type _AndSC<L, R> = uint::From<If<L, R, _0>>;
// Short-circuiting Or
pub type _OrSC<L, R> = uint::From<If<L, _1, R>>;

pub type _AndL<L, R> = If<L, R, _0>;
pub type _Xor<L, R> = uint::From<If<L, _IsZero<R>, R>>;
pub type _Xnor<L, R> = uint::From<If<L, R, _IsZero<R>>>;
pub type _Xor3<A, B, C> = uint::From<If<A, _Xnor<B, C>, _Xor<B, C>>>;

/// Eager version of `Half`.
pub type _H<N> = uint::From<Half<N>>;
/// Eager version of `Parity`.
pub type _P<N> = uint::From<Parity<N>>;

#[apply(lazy)]
// H := H(N), P := P(N)
//
// SatSubBit(N, C) := if C { if N { N - 1 } else { 0 } } else { N }
//                 = if N { N - C } else { N }
//                 = max(N - C, 0)
pub type _SatSubBit<N, C> = If<
    _AndSC<C, N>,
    // case C = 1, N > 0:
    //
    // SatSubBit(N, C) = N - 1
    // H > 0 or P != 0, hence if H = 0 then P = 1
    //
    // N - 1 = 2 * H + P - 1
    //       = 2 * H - (1 - P)
    //       = 2 * H - Not(P)
    //       = 2 * H - Not(P)
    //       = 2 * H - 2 * Not(P) + Not(P)
    //       = 2 * (H - Not(P)) + Not(P)
    //
    // if H > 0,
    //   then 2 * (H - Not(P)) + Not(P)
    //      = 2 * SatSubBit(H, Not(P)) + Not(P)
    //
    // if H = 0,
    //   then P = 1,
    //   thus 2 * (H - Not(P)) + Not(P) = 0
    //      = 2 * SatSubBit(H, Not(P)) + Not(P)
    //
    // Thus:
    // N - 1 = 2 * SatSubBit(H, Not(P)) + Not(P)
    //       = Append(SatSubBit(H, Not(P)), Not(P))
    AppendBit<
        _SatSubBit<
            //
            _H<N>,
            _IsZero<_P<N>>,
        >,
        _IsZero<_P<N>>,
    >,
    // case C = 0: SatSubBit(N, 0) = N
    // case N = 0: SatSubBit(0, C) = 0 = N
    N,
>;

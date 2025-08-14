use super::*;

#[apply(lazy)]
// H := H(N), P := P(N)
//
// SatDecIf(N, C) := if C { if N { N - 1 } else { 0 } } else { N }
//                 = if N { N - C } else { N }
pub type SatDecIfL<N, C = U1> = Tern<
    AndSC<C, N>,
    // case C = 1, N > 0:
    //
    // SatDecIf(N, C) = N - 1
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
    //      = 2 * SatDecIf(H, Not(P)) + Not(P)
    //
    // if H = 0,
    //   then P = 1,
    //   thus 2 * (H - Not(P)) + Not(P) = 0
    //      = 2 * SatDecIf(H, Not(P)) + Not(P)
    //
    // Thus:
    // N - 1 = 2 * SatDecIf(H, Not(P)) + Not(P)
    //       = Append(SatDecIf(H, Not(P)), Not(P))
    AppendL<SatDecIfL<Half<N>, BitNot<P<N>>>, BitNot<P<N>>>,
    // case C = 0: SatDecIf(N, 0) = N
    // case N = 0: SatDecIf(0, C) = 0 = N
    N,
>;

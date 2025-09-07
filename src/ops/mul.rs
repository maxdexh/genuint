use super::*;

// AddIf(C, L, R) := if C { L + R } else { R }
//                 = L + if C { R } else { 0 }
type AddIfL<C, L, R> = TernL<C, add::AddL<L, R>, L>;

// Double(N) := 2 * N
pub type DoubleL<N> = AppendL<N, _0>;

#[apply(lazy)]
// H := H(L), P := P(L)
//
// Mul(L, R) := L * R
pub type MulL<L, R> = Tern<
    L,
    // L * R = (2 * H + P) * R
    //       = 2 * (H * R) + P * R
    //       = 2 * (H * R) + if P { R } else { 0 }
    //       = AddIf(P, Double(H * R), R)
    AddIfL<P<L>, DoubleL<MulL<H<L>, R>>, R>,
    // 0 * R = 0
    _0,
>;

#[apply(opaque)]
#[apply(test_op! test_mul, L * R)]
pub type Mul<L, R> = MulL<L, R>;

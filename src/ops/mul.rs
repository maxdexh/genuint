use super::*;

// AddIf(C, L, R) := if C { L + R } else { R }
//                 = L + if C { R } else { 0 }
type _AddIf<C, L, R> = If<C, add::_CarryAdd<L, R>, L>;

// Double(N) := 2 * N
type _Double<N> = PushBit<N, _0>;

/// Type-level multiplication.
#[apply(opaque! pub(crate) mul_impl::_Mul)]
#[apply(test_op! test_mul, L * R)]
// H := H(L), P := P(L)
//
// Mul(L, R) := L * R
pub type Mul<L, R> = If<
    L,
    // L * R = (2 * H + P) * R
    //       = 2 * (H * R) + P * R
    //       = 2 * (H * R) + if P { R } else { 0 }
    //       = AddIf(P, Double(H * R), R)
    _AddIf<
        _P<L>,
        _Double<
            //
            _Mul<_H<L>, R>,
        >,
        R,
    >,
    // 0 * R = 0
    _0,
>;

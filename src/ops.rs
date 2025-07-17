use crate::{consts::*, internals, uint::From as UintFrom, utils::private_pub};

/// ```rust_analyzer_brace_infer
/// l! {}
/// ```
macro_rules! l {
    (
        $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty
    ) => {
        pub struct $name<$($param $(= $def)?),*>($($param),*);
        impl<$($param: $crate::ToUint),*> $crate::ToUint for $name<$($param),*> {
            type ToUint = $val;
        }
    };
}
/// ```rust_analyzer_brace_infer
/// a! {}
/// ```
macro_rules! a {
    (
        $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty
    ) => {
        pub type $name<$($param $(= $def)?),*> = $val;
    };
}

pub(crate) type TernSL<C, T, F> = internals::Prim!(UintFrom<C>, Ternary<T, F>);
a! {
    // H(N) := N / 2
    Half<N> = internals::Prim!(UintFrom<N>, Half)
}
a! {
    // P(N) := N % 2
    Parity<N> = internals::Prim!(UintFrom<N>, Parity)
}
a! {
    // Append(N, P) := 2 * N + if P { 1 } else { 0 }
    AppendAsBit<N, P> = internals::Prim!(UintFrom<N>, AppendAsBit<UintFrom<P>>)
}

a! {
    // Tern(C, T, F) := if C { T } else { F }
    Tern<C, T, F> = UintFrom<TernSL<C, T, F>>
}

private_pub! {
    mod helper;
    l! { AppendL<N, P> = AppendAsBit<N, P> }
    a! { BitNot<N> = Tern<N, U0, U1> }
    // Short-circuiting And
    a! { AndSC<L, R> = Tern<L, R, U0> }
    // Short-circuiting Or
    a! { OrSC<L, R> = Tern<L, U1, R> }
    a! { Xor<L, R> = Tern<L, BitNot<R>, R> }
    a! { Xnor<L, R> = Tern<L, R, BitNot<R>> }
    a! { Xor3<A, B, C> = Tern<A, Xnor<B, C>, Xor<B, C>> }
    l! { TernL<C, T, F> = Tern<C, T, F> }
    pub use super::{Half as H, Parity as P};
}

private_pub! {
    mod satdec;
    l! {
        // H := H(N), P := P(N)
        //
        // SatDecIf(N, C) := if C { if N { N - 1 } else { 0 } } else { N }
        //                 = if N { N - C } else { N }
        SatDecIfL<N, C = U1> = Tern<
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
            AppendL<
                SatDecIfL<
                    Half<N>,
                    BitNot<P<N>>
                >,
                BitNot<P<N>>
            >,
            // case C = 0: SatDecIf(N, 0) = N
            // case N = 0: SatDecIf(0, C) = 0 = N
            N,
        >
    }
}
#[cfg(test)]
pub(crate) type SatDec<N> = UintFrom<SatDecIfL<N>>;

#[cfg(test)]
mod testing;
#[cfg(test)]
use testing::test_op;
#[cfg(not(test))]
macro_rules! test_op {
    ($($input:tt)*) => {};
}

// TODO: Opaque
private_pub! {
    mod bitwise;
    l! {
        BitAndL<L, R> = Tern<
            L,
            AppendL<BitAndL<Half<R>, H<L>>, AndSC<P<L>, P<R>>>,
            U0,
        >
    }
}
a! { BitAnd<L, R> = UintFrom<BitAndL<L, R>> }
test_op! { test_bit_and: L R, BitAnd<L, R>, L & R }

private_pub! {
    mod add;
    l! {
        IncIfL<N, C> = Tern<
            C,
            AppendL<IncIfL<H<N>, P<N>>, BitNot<P<N>>>,
            N,
        >
    }
    l! {
        AddL<L, R, C = U0> = Tern<
            L,
            AppendL<
                AddL<
                    H<L>,
                    H<R>,
                    Tern<
                        P<L>,
                        OrSC<P<R>, C>,
                        AndSC<P<R>, C>,
                    >,
                >,
                Xor3<P<L>, P<R>, C>,
            >,
            IncIfL<R, C>,
        >
    }
}
a! { Add<L, R> = UintFrom<AddL<L, R>> }
test_op! { test_add: L R, Add<L, R>, L + R }

private_pub! {
    mod mul;

    a! {
        // AddIf(C, L, R) := if C { L + R } else { R }
        //                 = L + if C { R } else { 0 }
        AddIfSL<C, L, R> = TernSL<C, AddL<L, R>, L>
    }

    a! {
        // Double(N) := 2 * N
        DoubleL<N> = AppendL<N, U0>
    }

    l! {
        // H := H(L), P := P(L)
        //
        // Mul(L, R) := L * R
        MulL<L, R> = Tern<
            L,
            // L * R = (2 * H + P) * R
            //       = 2 * (H * R) + P * R
            //       = 2 * (H * R) + if P { R } else { 0 }
            //       = AddIf(P, Double(H * R), R)
            AddIfSL<
                P<L>,
                DoubleL<MulL<
                    H<L>,
                    R,
                >>,
                R,
            >,
            // 0 * R = 0
            U0,
        >
    }
}
pub type Mul<L, R> = UintFrom<MulL<L, R>>;
test_op! { test_mul: L R, Mul<L, R>, L * R }

private_pub! {
    mod cmp;
    // Notation: Cond!(C) := if C { 1 } else { 0 }
    //           Cond!(C) = Cond!(D)  iff  (C iff D)
    //           Cond!(C) = 1  iff  C

    l! {
        // HL := H(L), PL := P(L), HR := H(R), PR := P(R)
        //
        //     L = R
        // iff 2 * HL + PL = 2 * HR + PR
        // iff PL = PR  and  HL = HR
        // iff Xnor(PL, PR) = 1  and  Eq(HL, HR) = 1
        // iff And(Xnor(PL, PR), Eq(HL, HR)) = 1
        //
        // Eq(L, R) := Cond!(L == R) = And(Xnor(PL, PR), Eq(HL, HR))
        EqL<L, R> = AndSC<
            Xnor<P<L>, P<R>>,
            EqL<H<L>, H<R>>,
        >
    }

    l! {
        // HL := H(L), PL := P(L), HR := H(R), PR := P(R)
        //
        // LtByLast(L, R) := Cond!(HL = HR and PL = 0 and PR = 1)
        LtByLastL<L, R> = AndSC<
            Tern<P<L>, U0, P<R>>, // Cond!(not PL and PR)
            EqL<H<L>, H<R>>,  // Cond!(HL = HR)
        >
    }
    l! {
        // HL := H(L), PL := P(L), HR := H(R), PR := P(R)
        //
        // Lt(L, R) := Cond!(L < R)
        LtL<L, R> = AndSC<
            R,
            TernSL<
                L,
                //     L < R
                // iff 2 * HL + PL < 2 * HR + PR
                // iff HL < HR or HL = HR and PL = 0 and PR = 1
                // iff Lt(HL, HR) = 1 or LtByLastL(L, R) = 1
                TernL<
                    LtL<H<L>, H<R>>,
                    U1,
                    LtByLastL<L, R>,
                >,
                // 0 < R  because R = 0 is handled by the initial And
                U1
            >,
        >
    }
}
a! { Lt<L, R> = UintFrom<LtL<L, R>> }
a! { Gt<L, R> = Lt<R, L> }
a! { Eq<L, R> = UintFrom<EqL<L, R>> }

private_pub! {
    mod sub;

    l! {
        // HL := H(L), PL := P(L), HR := H(R), PR := P(R), C <= 1, L <= R + C
        //
        // USub(L, R, C) := L - R - C
        USubL<L, R, C = U0> = Tern<
            R, // don't bother short-circuiting on L, since L <= R

            // L - R - C = 2 * HL + PL - (2 * HR + PR) - C
            //           = 2 * (HL - HR) + PL - PR - C
            //           = 2 * (HL - HR) + X
            //
            // where X := PL - PR - C, so -2 <= X <= 1.
            //
            // Using euclidian/floor divmod (identical for positive divisor):
            // X % 2 = (PL - PR - C) % 2 = (PL + PR + C) % 2 = Xor(PL, PR, C).
            // CC := -(X / 2), so 0 <= CC <= 1 because -1 <= X / 2 <= 0
            //
            // Hence, X = 2 * (X / 2) + X % 2 = - 2 * CC + Xor(PL, PR, C)
            //
            // L - R - C = 2 * (HL - HR) - 2 * CC + Xor(PL, PR, C)
            //           = 2 * (HL - HR - CC) + Xor(PL, PR, C)
            //           = Append(USub(HL, HR, CC), Xor(PL, PR, C))
            //
            AppendL<
                USubL<
                    H<L>,
                    H<R>,
                    // Because CC is -(X / 2) using floor division, we have X / 2 < 0  iff  X < 0.
                    // Thus, CC = 1  iff  CC > 0  iff  X / 2 < 0  iff  X < 0  iff  PL < PR + C
                    Tern<
                        P<L>,
                        // case PL = 1:
                        // CC = 1  iff  1 < PR + C  iff  PR = 1 and C = 1  iff  And(PR, C) = 1
                        AndSC<P<R>, C>,
                        // case PL = 0:
                        // CC = 1  iff  0 < PR + C  iff  PR = 1  or C = 1  iff   Or(PR, C) = 1
                        OrSC<P<R>, C>,
                    >,
                >,
                Xor3<P<L>, P<R>, C>,
            >,

            // case R = 0: L - 0 - C = L - C = SatDecIf(L, C)
            SatDecIfL<L, C>,
        >
    }
}
a! { AbsDiff<L, R> = Tern<Lt<L, R>, USubL<R, L>, USubL<L, R>> }
a! { SatSub<L, R> = Tern<Gt<L, R>, USubL<L, R>, U0> }

private_pub! {
    mod div;
}

use crate::{consts::*, uint::From as UintFrom, utils::private_pub};

/// ```rust_analyzer_brace_infer
/// l! {}
/// ```
macro_rules! l {
    (
        $v:vis $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty
    ) => {
        $v struct $name<$($param $(= $def)?),*>($($param),*);
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
        $v:vis $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty
    ) => {
        $v type $name<$($param $(= $def)?),*> = $val;
    };
}

pub(crate) type TernSL<C, T, F> = crate::internals::Prim!(UintFrom<C>, Ternary<T, F>);
pub type Half<N> = crate::internals::Prim!(UintFrom<N>, Half);
pub type Parity<N> = crate::internals::Prim!(UintFrom<N>, Parity);
pub type AppendAsBit<N, P> = crate::internals::Prim!(UintFrom<N>, AppendAsBit<UintFrom<P>>);
pub type Tern<C, T, F> = UintFrom<TernSL<C, T, F>>;

private_pub! {
    mod helper;
    l! { pub AppendL<N, P> = AppendAsBit<N, P> }
    a! { pub BitNot<N> = Tern<N, U0, U1> }
    a! { pub ToBit<N> = Tern<N, U1, U0> }
    // Short-circuiting And
    a! { pub AndSC<L, R> = Tern<L, R, U0> }
    // Short-circuiting Or
    a! { pub OrSC<L, R> = Tern<L, U1, R> }
    a! { pub Xor<L, R> = Tern<L, BitNot<R>, R> }
    a! { pub Xnor<L, R> = Tern<L, R, BitNot<R>> }
    a! { pub Xor3<A, B, C> = Tern<A, Xnor<B, C>, Xor<B, C>> }
    l! { pub TernL<C, T, F> = Tern<C, T, F> }
    a! { pub H<N> = Half<N> }
    a! { pub P<N> = Parity<N> }
}

mod satdec {
    use super::*;
    l! {
        pub SatDecL<N, If = U1> = Tern<
            AndSC<If, N>,
            AppendL<SatDecL<Half<N>, BitNot<P<N>>>, BitNot<P<N>>>,
            N,
        >
    }
}
#[cfg(test)]
pub(crate) type SatDec<N> = UintFrom<satdec::SatDecL<N>>;

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
        pub BitAndL<L, R> = Tern<
            L,
            AppendL<BitAndL<Half<R>, H<L>>, AndSC<P<L>, P<R>>>,
            U0,
        >
    }
}
pub type BitAnd<L, R> = UintFrom<BitAndL<L, R>>;
test_op! { test_bit_and: L R, BitAnd<L, R>, L & R }

private_pub! {
    mod add;
    l! {
        pub IncL<N, If> = Tern<
            If,
            AppendL<IncL<H<N>, P<N>>, BitNot<P<N>>>,
            N,
        >
    }
    l! {
        pub AddL<L, R, C = U0> = Tern<
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
            IncL<R, C>,
        >
    }
}
pub type Add<L, R> = UintFrom<AddL<L, R>>;
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
        pub MulL<L, R> = Tern<
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
        //
        LtL<L, R> = Tern<
            AndSC<R, L>,
            //     L < R
            // iff 2 * HL + PL < 2 * HR + PR
            // iff HL < HR or HL = HR and PL = 0 and PR = 1
            // iff Lt(HL, HR) = 1 or LtByLastL(L, R) = 1
            TernL<
                LtL<H<L>, H<R>>,
                U1,
                LtByLastL<L, R>,
            >,
            // if L = 0:  0 < R  iff  R != 0
            // if R = 0:  L < 0  iff  false  iff  R != 0
            //
            // Cond!(R != 0) = ToBit(R)
            ToBit<R>,
        >
    }
}

private_pub! {
    mod sub;
}

private_pub! {
    mod div;
}

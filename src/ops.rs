use crate::{consts::*, internals, uint, utils::private_pub};
use macro_rules_attribute::apply;

macro_rules! lazy {
    (
        $(#[$($attr:tt)*])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        $(#[$($attr)*])*
        pub struct $name<$($param $(= $def)?),*>($($param),*);
        impl<$($param: $crate::ToUint),*> $crate::ToUint for $name<$($param),*> {
            type ToUint = $val;
        }
    };
}

macro_rules! opaque {
    (
        $(#[$($attr:tt)*])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        $(#[$($attr)*])*
        pub type $name<$($param $(= $def)?),*> = $crate::uint::From<$val>;
    };
}

pub(crate) type TernSL<C, T, F> = internals::Prim!(uint::From<C>, Ternary<T, F>);

// H(N) := N / 2
pub type Half<N> = internals::Prim!(uint::From<N>, Half);

// P(N) := N % 2
pub type Parity<N> = internals::Prim!(uint::From<N>, Parity);

// Append(N, P) := 2 * N + if P { 1 } else { 0 }
pub type AppendAsBit<N, P> = internals::Prim!(uint::From<N>, AppendAsBit<uint::From<P>>);

// Tern(C, T, F) := if C { T } else { F }
pub type Tern<C, T, F> = uint::From<TernSL<C, T, F>>;

mod helper {
    use super::*;

    #[apply(lazy)]
    pub type TernL<C, T, F> = Tern<C, T, F>;

    #[apply(lazy)]
    pub type AppendL<N, P> = AppendAsBit<N, P>;

    // Short-circuiting And
    pub type AndSC<L, R> = Tern<L, R, U0>;
    // Short-circuiting Or
    pub type OrSC<L, R> = Tern<L, U1, R>;

    pub type BitNot<N> = Tern<N, U0, U1>;
    pub type AndL<L, R> = TernL<L, R, U0>;
    pub type Xor<L, R> = Tern<L, BitNot<R>, R>;
    pub type Xnor<L, R> = Tern<L, R, BitNot<R>>;
    pub type Xor3<A, B, C> = Tern<A, Xnor<B, C>, Xor<B, C>>;

    pub use super::{Half as H, Parity as P};
}
pub(crate) use helper::*;

mod satdec {
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
}
pub(crate) use satdec::*;

#[cfg(test)]
pub(crate) type SatDec<N> = uint::From<SatDecIfL<N>>;

#[cfg(test)]
mod testing;
#[cfg(test)]
use testing::test_op;
#[cfg(not(test))]
macro_rules! test_op {
    ($($input:tt)*) => {};
}

mod bitwise {
    use super::*;
    // TODO: Missing stuff
    #[apply(lazy)]
    pub type BitAndL<L, R> = Tern<L, AppendL<BitAndL<Half<R>, H<L>>, AndSC<P<L>, P<R>>>, U0>;
}
pub(crate) use bitwise::*;
#[apply(opaque)]
pub type BitAnd<L, R> = BitAndL<L, R>;
test_op! { test_bit_and: L R, BitAnd<L, R>, L & R }

mod add {
    #[allow(unused_imports)]
    use super::*;

    #[apply(lazy)]
    pub type IncIfL<N, C> = Tern<C, AppendL<IncIfL<H<N>, P<N>>, BitNot<P<N>>>, N>;

    #[apply(lazy)]
    pub type AddL<L, R, C = U0> = Tern<
        L,
        AppendL<AddL<H<L>, H<R>, Tern<P<L>, OrSC<P<R>, C>, AndSC<P<R>, C>>>, Xor3<P<L>, P<R>, C>>,
        IncIfL<R, C>,
    >;
}
#[allow(unused_imports)]
pub(crate) use add::*;

#[apply(opaque)]
pub type Add<L, R> = AddL<L, R>;
test_op! { test_add: L R, Add<L, R>, L + R }

mod mul {
    use super::*;

    // AddIf(C, L, R) := if C { L + R } else { R }
    //                 = L + if C { R } else { 0 }
    type AddIfSL<C, L, R> = TernSL<C, AddL<L, R>, L>;

    // Double(N) := 2 * N
    pub type DoubleL<N> = AppendL<N, U0>;

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
        AddIfSL<P<L>, DoubleL<MulL<H<L>, R>>, R>,
        // 0 * R = 0
        U0,
    >;
    test_op! { test_mul: L R, MulL<L, R>, L * R }
}
pub(crate) use mul::*;

#[apply(opaque)]
pub type Mul<L, R> = MulL<L, R>;

mod cmp {
    use super::*;
    // Notation: Cond!(C) := if C { 1 } else { 0 }
    //           Cond!(C) = Cond!(D)  iff  (C iff D)
    //           Cond!(C) = 1  iff  C

    #[apply(lazy)]
    // HL := H(L), PL := P(L), HR := H(R), PR := P(R)
    //
    // Eq(L, R) := Cond!(L == R)
    pub type EqL<L, R> = Tern<
        L,
        //     L = R
        // iff 2 * HL + PL = 2 * HR + PR
        // iff PL = PR  and  HL = HR
        // iff Xnor(PL, PR) = 1  and  Eq(HL, HR) = 1
        // iff And(Xnor(PL, PR), Eq(HL, HR)) = 1
        AndL<Xnor<P<L>, P<R>>, EqL<H<R>, H<L>>>,
        // case L = 0:  0 = R  iff  (if R { 0 } else { 1 }) = 1
        TernL<R, U0, U1>,
    >;

    #[apply(lazy)]
    // HL := H(L), PL := P(L), HR := H(R), PR := P(R)
    //
    // LtByLast(L, R) := Cond!(HL = HR and PL = 0 and PR = 1)
    pub type LtByLastL<L, R> = AndSC<
        Tern<P<L>, U0, P<R>>, // Cond!(not PL and PR)
        EqL<H<L>, H<R>>,      // Cond!(HL = HR)
    >;

    #[apply(lazy)]
    // HL := H(L), PL := P(L), HR := H(R), PR := P(R)
    //
    // Lt(L, R) := Cond!(L < R)
    pub type LtL<L, R> = AndSC<
        R,
        TernSL<
            L,
            //     L < R
            // iff 2 * HL + PL < 2 * HR + PR
            // iff HL < HR or HL = HR and PL = 0 and PR = 1
            // iff Lt(HL, HR) = 1 or LtByLastL(L, R) = 1
            TernL<LtL<H<L>, H<R>>, U1, LtByLastL<L, R>>,
            // 0 < R  because R = 0 is handled by the initial And
            U1,
        >,
    >;
}
pub(crate) use cmp::*;

#[apply(opaque)]
pub type Lt<L, R> = LtL<L, R>;
test_op! { test_cmp: L R, Lt<L, R>, (L < R) as _ }

#[apply(opaque)]
pub type Gt<L, R> = Lt<R, L>;

// TODO: Ge, Le

#[apply(opaque)]
pub type Eq<L, R> = EqL<L, R>;
test_op! { test_eq: L R, Eq<L, R>, (L == R) as _ }

#[apply(opaque)]
pub type Min<L, R> = Tern<Lt<L, R>, R, L>;

#[apply(opaque)]
pub type Max<L, R> = Tern<Lt<L, R>, L, R>;

mod sub {
    use super::*;

    #[apply(lazy)]
    // HL := H(L), PL := P(L), HR := H(R), PR := P(R), C <= 1, L <= R + C
    //
    // USub(L, R, C) := L - R - C
    pub type USubL<L, R, C = U0> = Tern<
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
    >;
}
pub(crate) use sub::*;

#[apply(opaque)]
pub type AbsDiff<L, R> = Tern<Lt<L, R>, USubL<R, L>, USubL<L, R>>;
test_op! { test_abs_diff: L R, AbsDiff<L, R>, L.abs_diff(R) }

#[apply(opaque)]
pub type SatSub<L, R> = Tern<Gt<L, R>, USubL<L, R>, U0>;
test_op! { test_sat_sub: L R, SatSub<L, R>, L.saturating_sub(R) }

mod divrem {
    use super::*;

    // SubIfGe(L, R) := if L >= R { L - R } else { L } = if L < R { L } else { L - R }
    #[apply(lazy)]
    pub type SubIfGeL<L, R> = Tern<
        //
        Lt<L, R>,
        L,
        USubL<L, R>,
    >;

    // H := H(L), P := P(L)
    //
    // URemIn(L, R) := 2 * (H % R) + P
    //
    // L % R = (2 * H + P) % R
    //       = (2 * (H % R) + P) % R
    //       = URemIn(L, R) % R
    pub type URemInSL<L, R> = AppendL<
        //
        RemL<H<L>, R>,
        P<L>,
    >;

    #[apply(lazy)]
    // 0 % R = 0. We also get L % R = URem(L, R) % R
    pub type URemL<L, R> = Tern<
        //
        L,
        URemInSL<L, R>,
        U0,
    >;

    // H % R <= R - 1
    // thus URem(L, R) = 2 * (H % R) + P <= 2 * (H % R) + 1 <= 2 * R - 1
    // thus L % R = URem(L, R) % R = SubIfGe(URem(L, R), R)
    pub type RemL<L, R> = SubIfGeL<
        //
        URemL<L, R>,
        R,
    >;

    #[apply(lazy)]
    pub type DivL<L, R> = Tern<
        //
        L,
        AppendL<
            //
            DivL<Half<L>, R>,
            BitNot<LtL<URemL<L, R>, R>>,
        >,
        U0,
    >;

    #[apply(lazy)]
    pub type CheckedRemL<L, R> = Tern<R, RemL<L, R>, CheckedRemL<L, R>>;
    #[apply(lazy)]
    pub type CheckedDivL<L, R> = Tern<R, DivL<L, R>, CheckedDivL<L, R>>;
}
pub(crate) use divrem::*;

/// ```compile_fail
/// use generic_uint::{ops::Rem, consts::*};
/// const _: fn(Rem<U1, U0>) = |_| {};
/// ```
#[apply(opaque)]
pub type Rem<L, R> = CheckedRemL<L, R>;

/// ```compile_fail
/// use generic_uint::{ops::Div, consts::*};
/// const _: fn(Div<U1, U0>) = |_| {};
/// ```
#[apply(opaque)]
pub type Div<L, R> = CheckedDivL<L, R>;

#[cfg(test)]
mod divrem_test {
    use super::*;

    #[apply(lazy)]
    pub type RemL<L, R> = Rem<L, R>;
    type DefaultRem<L, R> = Tern<R, RemL<L, R>, U0>;
    test_op! { test_rem: L R, DefaultRem<L, R>, L.checked_rem(R).unwrap_or(0) }

    #[apply(lazy)]
    pub type DivL<L, R> = Div<L, R>;
    type DefaultDiv<L, R> = Tern<R, DivL<L, R>, U0>;
    test_op! { test_div: L R, DefaultDiv<L, R>, L.checked_div(R).unwrap_or(0) }
}

private_pub! {
    mod pow;
}

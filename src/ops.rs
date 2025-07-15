use crate::{UintFrom, consts::*};

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

pub type LazyTernary<C, T, F> = crate::internals::Prim!(UintFrom<C>, Ternary<T, F>);
pub type Half<N> = crate::internals::Prim!(UintFrom<N>, Half);
pub type Parity<N> = crate::internals::Prim!(UintFrom<N>, Parity);
pub type AppendAsBit<N, P> = crate::internals::Prim!(UintFrom<N>, AppendAsBit<UintFrom<P>>);

pub type Ternary<C, T, F> = UintFrom<LazyTernary<C, T, F>>;

mod helper {
    use super::*;
    l! { AppendL<N, P> = AppendAsBit<N, P> }
    pub type BitNot<N> = Ternary<N, U0, U1>;
    pub type And<L, R> = Ternary<L, R, U0>;
}
use helper::*;

mod satdec {
    use super::*;
    l! {
        SatDecL<N, If = U1> = Ternary<
            And<If, N>,
            AppendL<SatDecL<Half<N>, BitNot<Parity<N>>>, BitNot<Parity<N>>>,
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

mod bitwise {
    use super::*;
    l! {
        BitAndL<L, R> = Ternary<
            L,
            AppendL<BitAndL<Half<R>, Half<L>>, And<Parity<L>, Parity<R>>>,
            U0,
        >
    }
}
pub type BitAnd<L, R> = UintFrom<bitwise::BitAndL<L, R>>;
test_op! { test_bit_and: L R, BitAnd<L, R>, L & R }

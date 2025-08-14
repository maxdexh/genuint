use crate::{consts::*, internals, uint};
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

// H(N) := N / 2
pub type Half<N> = internals::PrimitiveOp!(uint::From<N>, ::Half);

// P(N) := N % 2
pub type Parity<N> = internals::PrimitiveOp!(uint::From<N>, ::Parity);

// Append(N, P) := 2 * N + if P { 1 } else { 0 }
pub type AppendAsBit<N, P> = internals::PrimitiveOp!(uint::From<N>, ::AppendAsBit<uint::From<P>>);

// Tern(C, T, F) := if C { T } else { F }
pub type Tern<C, T, F> = uint::From<internals::PrimitiveOp!(uint::From<C>, ::Ternary<T, F>)>;

mod helper;
pub(crate) use helper::*;

mod satdec;

// We need this to iterate over ranges of uints in tests
#[cfg(test)]
pub type SatDecForTest<N> = uint::From<satdec::SatDecIfL<N>>;

#[cfg(test)]
mod testing;
macro_rules! test_op {
    ($($input:tt)*) => {
        #[cfg(test)]
        $crate::ops::testing::test_op! { $($input)* }
    };
}

mod bitwise;
pub use bitwise::BitAnd;

mod add;
pub use add::Add;

mod mul;
pub use mul::Mul;

mod cmp;
pub use cmp::{Eq, Ge, Gt, Le, Lt, Max, Min, Ne};

mod sub;
pub use sub::{AbsDiff, SatSub};

mod divrem;
pub use divrem::{Div, Rem};

// TODO: pow

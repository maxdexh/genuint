use crate::{consts::*, internals, uint};
use macro_rules_attribute::apply;

// TODO: Warning for custom op: Having associated type projections in the result can affect type
// inference

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

macro_rules! __make_opaque {
    (, $out:ty) => {
        $out
    };
    ($pop:ident $($param:ident)*, $out:ty) => {
        $crate::internals::PrimitiveOp!(
            $crate::uint::From<$pop>,
            ::Opaque<
                $crate::ops::__make_opaque!($($param)*, $out)
            >
        )
    };
}
pub(crate) use __make_opaque;

macro_rules! opaque {
    (
        $(#[$($attr:tt)*])*
        pub type $name:ident<$($param:ident $(= $def:ty)?),* $(,)?> = $val:ty;
    ) => {
        $(#[$($attr)*])*
        pub type $name<$($param $(= $def)?),*> = $crate::uint::From<
            $crate::ops::__make_opaque!($($param)*, $val)
        >;
    };
}

/// More efficient implementation of [`Div<N, U2>`].
///
/// Under the current implementation, this is a primitive operation.
// H(N) := N / 2
pub type Half<N> = internals::PrimitiveOp!(uint::From<N>, ::Half);

/// More efficient implementation of [`Rem<N, U2>`].
///
/// Under the current implementation, this is a primitive operation.
// P(N) := N % 2
pub type Parity<N> = internals::PrimitiveOp!(uint::From<N>, ::Parity);

/// More efficient implementation of `Add<Mul<N, U2>, Tern<P, U1, U0>>`.
///
/// Under the current implementation, this is a primitive operation.
// Append(N, P) := 2 * N + if P { 1 } else { 0 }
pub type AppendBit<N, P> = internals::PrimitiveOp!(uint::From<N>, ::AppendAsBit<uint::From<P>>);

/// If-else/Ternary function.
///
/// If the first argument is nonzero, evaluates to the second argument, otherwise to the third.
///
/// This will only access ("evaluate") [`ToUint::ToUint`](crate::ToUint::ToUint) for the required
/// argument. This means that this operation can be used for the exit condition of a recursive
/// operation (see examples below).
///
/// Under the current implementation, this is a primitive operation.
///
/// # Examples
/// Exiting from a recursive operation
/// ```
/// use generic_uint::{ToUint, ops, uint};
/// struct CountOnesL<N, Acc>(N, Acc);
/// impl<N: ToUint, Acc: ToUint> ToUint for CountOnesL<N, Acc> {
///     type ToUint = ops::Tern<
///         N,
///         CountOnesL<
///             ops::Half<N>,
///             ops::Add<Acc, ops::Parity<N>>,
///         >,
///         Acc,
///     >;
/// }
/// type CountOnes<N> = uint::From<CountOnesL<N, uint::FromU128<0>>>;
/// assert_eq!(
///     uint::to_u128::<CountOnes<uint::FromU128<0b1011101>>>(),
///     Some(5),
/// );
/// ```
//
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

// TODO: shift

// TODO: pow
